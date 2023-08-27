use log::{debug, info, error};
use regex::Regex;
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use tokio::{fs::File as TokioFile, io::AsyncReadExt};

/// Represents a node in a DHT network.
#[derive(Clone, Debug, Deserialize, Serialize)]
struct Node(String, i64);

/// Represents a file described in a torrent.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct File {
    pub path: Vec<String>,
    pub length: u64,
    #[serde(default)]
    md5sum: Option<String>,
}

/// Represents the metadata of a torrent.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Info {
    pub name: String,
    #[serde(with = "serde_bytes")]
    pub pieces: Vec<u8>,
    #[serde(rename = "piece length")]
    pub piece_length: u64,
    #[serde(default)]
    md5sum: Option<String>,
    #[serde(default)]
    pub length: Option<i64>,
    #[serde(default)]
    pub files: Option<Vec<File>>,
    #[serde(default)]
    private: Option<u8>,
    #[serde(default)]
    path: Option<Vec<String>>,
    #[serde(default)]
    #[serde(rename = "root hash")]
    root_hash: Option<String>,
}

/// Represents a torrent.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Torrent {
    pub info: Info,
    #[serde(default)]
    pub announce: Option<String>,
    #[serde(default)]
    nodes: Option<Vec<Node>>,
    #[serde(default)]
    encoding: Option<String>,
    #[serde(default)]
    httpseeds: Option<Vec<String>>,
    #[serde(default)]
    #[serde(rename = "announce-list")]
    announce_list: Option<Vec<Vec<String>>>,
    #[serde(default)]
    #[serde(rename = "creation date")]
    creation_date: Option<i64>,
    #[serde(rename = "comment")]
    comment: Option<String>,
    #[serde(default)]
    #[serde(rename = "created by")]
    created_by: Option<String>
}

impl Torrent {
    /// Reads a `.torrent` file and converts it into a `Torrent` struct.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the `.torrent` file.
    pub async fn from_torrent_file(path: &str) -> Self {
        // Read .torrent File
        let mut file = TokioFile::open(path).await.unwrap();
        let mut buf: Vec<u8> = Vec::new();

        match file.read_to_end(&mut buf).await {
            Err(err) => { 
                error!("Error reading file till end: {err}");
                panic!("Error reading file till end: {err}");
            }
            Ok(_) => { info!("Succesfully read {path}") }
        }

        match serde_bencode::from_bytes(&buf) {
            Err(err) => { 
                error!("Error deserializing file: {err}");
                panic!("Error deserializing file: {err}");
            }
            Ok(torrent) => { torrent }
        }
    }

    /// Logs info about the *.torrent file
    pub fn log_useful_information(&self) {
        info!(" -> Torrent Information <- ");
        info!("Name: {}", self.info.name);
        info!("Tracker: {:?}", self.announce);
        info!("Tracker List: {:?}", self.announce_list);
        info!("Info Hash: {:X?}", self.get_info_hash());
        info!("Length: {:?}", self.info.length);
        
        info!("Files:");

        match &self.info.files {
            None => { 
                info!("> {}", self.info.name);
            }
            Some(files) => {
                for file in files {
                    let mut path = String::new();
                    for section in &file.path {
                        path.push_str(&format!("{section}/"));
                    }
                    path.pop();
                    info!("> {}: {}B", path, file.length)
                }
            }
        }
    } 

    /// Calculates the info hash of the torrent.
    pub fn get_info_hash(&self) -> Vec<u8> {
        let buf = serde_bencode::to_bytes(&self.info).unwrap();

        let mut hasher = Sha1::new();
        hasher.update(buf);
        let res = hasher.finalize();
        res[..].to_vec()
    }

    /// Checks if a downloaded piece matches its hash.
    ///
    /// # Arguments
    ///
    /// * `piece` - The downloaded piece.
    /// * `index` - The index of the piece.
    /// 
    /// # Returns
    ///
    /// * `true` if the piece is correct, `false` otherwise.
    pub fn check_piece(&self, piece: &[u8], index: u32) -> bool {
        let mut hasher = Sha1::new();
        hasher.update(piece);
        let result = hasher.finalize();  

        let piece_hash = &self.info.pieces[(index * 20) as usize..(index * 20 + 20) as usize];

        if &result[..] == piece_hash {
            info!("Downloaded Piece {}/{} Correct!, Piece Was: {}B long", index + 1, self.info.pieces.len() / 20,  piece.len(),);
            true
        } else {
            debug!("{:?}", &result[..]);
            debug!("{:?}", piece_hash);
            debug!("{:?}", &result[..].len());
            debug!("{:?}", piece_hash.len());
            debug!("{}", piece.len());
            error!("Piece downloaded incorrectly");
            false
        }
    }

    pub fn get_total_length(&self) -> u64 {
        match self.info.length {
            None => {},
            Some(n) => { return n as u64 }
        };

        match &self.info.files {
            None => { 0 },
            Some(files) => {
                let mut n = 0;

                for file in files {
                    n += file.length;
                };

                n
            }
        }
    }

    pub fn get_tracker(&self) -> (&str, u16) {
        let re = Regex::new(r"^udp://([^:/]+):(\d+)/announce$").unwrap();

        if let Some(url) = &self.announce {
            if let Some(captures) = re.captures(url) {
                let hostname = captures.get(1).unwrap().as_str();
                let port = captures.get(2).unwrap().as_str();
    
                return (hostname, port.parse().unwrap());
            } else {
                println!("URL does not match the expected pattern | {}", url);
            }
        }

        for (i, url) in self.announce_list.as_ref().unwrap().iter().enumerate() {
            debug!("{:?}", url);
            if let Some(captures) = re.captures(&url[i]) {
                let hostname = captures.get(1).unwrap().as_str();
                let port = captures.get(2).unwrap().as_str();
    
                return (hostname, port.parse().unwrap());
            } else {
                println!("URL does not match the expected pattern | {}", url[i]);
            }
        }

        ("", 0)
    }
}