use log::debug;
use tokio::{
    fs::try_exists as dir_exists,
    fs::create_dir as create_dir,
    fs::File, 
    io::AsyncWriteExt
};

use crate::torrent::Torrent;

/// Represents information about a file being downloaded.
#[derive(Debug)]
struct FileInfo {
    file: File,
    length: u64,
    current_length: u64,
    name: String,
    complete: bool
} 

/// Represents a collection of files being downloaded.
#[derive(Debug)]
pub struct Files(Vec<FileInfo>);

impl Files {
    /// Creates a new `Files` instance.
    pub fn new() -> Self {
        Self(vec![])
    }

    /// Creates the files in the local system for downloading.
    ///
    /// # Arguments
    ///
    /// * `torrent` - The `Torrent` instance describing the torrent.
    /// * `download_path` - The path where the files will be downloaded.
    pub async fn create_files(&mut self, torrent: &Torrent, download_path: &str) {
        match &torrent.info.files {
            // Single File Mode
            None => {
                let path = &format!("{download_path}/{}", torrent.info.name);
                let file = File::create(&path).await.unwrap();

                let length = torrent.info.length.unwrap_or(0) as u64;

                self.0.push(FileInfo { file, length, current_length: 0, name: path.to_string(), complete: false })
            }

            // Multi File Mode
            Some(files) => {
                for t_file in files {
                    let mut path = download_path.to_string();

                    for dir in &t_file.path[..t_file.path.len() - 1] {
                        path.push('/');
                        path.push_str(dir);
                        
                        if !dir_exists(&path).await.unwrap() {
                            debug!("Creating: {path}");
                            create_dir(&path).await.unwrap();
                        }
                    }

                    path.push('/');
                    path.push_str(&t_file.path[t_file.path.len() - 1]);

                    
                    debug!("Creating: {path}");
                    let file = File::create(&path).await.unwrap();
                    let length = t_file.length;

                    self.0.push(FileInfo { file, length, current_length: 0, name: path.to_string(), complete: false });
                }
            }
        }
    }

    /// Writes a piece of data to the appropriate files.
    ///
    /// # Arguments
    ///
    /// * `piece` - The piece of data to write.
    pub async fn write_piece(&mut self, piece: Vec<u8>) {
        let mut j = 0;

        let mut piece_len = piece.len() as u64;
        let file_iterator = self.0.iter_mut();

        for file in file_iterator {

            if file.complete { continue }

            if file.current_length + piece_len > file.length {
                let n = file.file.write(&piece[j..(file.length - file.current_length) as usize]).await.unwrap();
                debug!("Wrote {n}B > {}", file.name);
                j = (file.length - file.current_length) as usize;
                file.current_length += j as u64;
                piece_len -= j as u64;
                file.complete = true;
            } else {
                let n = file.file.write(&piece[j..]).await.unwrap();
                debug!("Wrote {n}B > {}", file.name);
                file.current_length += piece_len;
                return
            }
        }
    }
}