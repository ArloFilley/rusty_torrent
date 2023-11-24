use regex::Regex;
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use tokio::{fs::File as TokioFile, io::AsyncReadExt};
use std::net::{IpAddr, SocketAddrV4};

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
    pub async fn from_torrent_file(path: &str) -> Result<Self, String> {
        let Ok(mut file) = TokioFile::open(path).await else {
            return Err(format!("Unable to read file at {path}"));
        };

        let mut buf: Vec<u8> = Vec::new();
        let Ok(_) = file.read_to_end(&mut buf).await else {
            return Err(format!("Error reading file > {path}"));
        };

        let torrent: Torrent = match serde_bencode::from_bytes(&buf) {
            Err(_) => return Err(format!("Error deserializing file > {path}")),
            Ok(torrent) => torrent,
        };

        Ok(torrent)
    }
}
    
impl Torrent {
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
            true
        } else {
            false
        }
    }
    
    pub fn get_total_length(&self) -> u64 {
        if let Some(n) = self.info.length {
            return n as u64
        };
        
        if let Some(files) = &self.info.files {
            let mut n = 0;
                
            for file in files {
                n += file.length;
            };
                
            return n
        };

        0
    }
    
    pub fn get_trackers(&self) -> Result<Vec<SocketAddrV4>, String> {
        let mut addresses = vec![];

        // This is the current regex as I haven't implemented support for http trackers yet
        let re = Regex::new(r"^udp://([^:/]+):(\d+)/announce$").unwrap();
        
        if let Some(url) = &self.announce {
            if let Some(captures) = re.captures(url) {
                let hostname = captures.get(1).unwrap().as_str();
                let port = captures.get(2).unwrap().as_str();

                if let Ok(ip) = dns_lookup::lookup_host(hostname) {
                    for i in ip { 
                        if let IpAddr::V4(j) = i {
                            addresses.push(SocketAddrV4::new(j, port.parse().unwrap()))
                        }
                    }
                }
            }
        }
        
        if let Some(urls) = &self.announce_list {
            for url in urls.iter() {
                if let Some(captures) = re.captures(&url[0]) {
                    let hostname = captures.get(1).unwrap().as_str();
                    let port = captures.get(2).unwrap().as_str();
                    
                    if let Ok(ip) = dns_lookup::lookup_host(hostname) {
                        for i in ip { 
                            if let IpAddr::V4(j) = i {
                                addresses.push(SocketAddrV4::new(j, port.parse().unwrap()));
                            }
                        }
                    }
                }
            }
        }
        
        if addresses.len() > 0 {
            Ok(addresses)
        } else {
            Err(String::from("Unable to find trackers"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::runtime::Runtime;

    #[test]
    fn from_torrent_file_success() {
        let runtime = Runtime::new().unwrap();
        let path = "test.torrent";

        let result = runtime.block_on(Torrent::from_torrent_file(path));
        println!("{result:?}");

        assert!(result.is_ok());
    }

    #[test]
    fn from_torrent_file_failure() {
        let runtime = Runtime::new().unwrap();
        let path = "nonexistent/file.torrent";

        let result = runtime.block_on(Torrent::from_torrent_file(path));

        assert!(result.is_err());
    }

    #[test]
    fn get_info_hash() {
        // Create a mock Torrent instance
        let torrent = Torrent {
            info: Info {
                name: String::from("test_torrent"),
                pieces: vec![],
                piece_length: 1024,
                length: Some(2048),
                files: None,
                md5sum: None,
                private: None,
                path: None,
                root_hash: None,
            },
            announce: Some(String::from("http://tracker.example.com/announce")),
            nodes: None,
            encoding: None,
            httpseeds: None,
            announce_list: None,
            creation_date: None,
            comment: None,
            created_by: None,
        };

        let result = torrent.get_info_hash();

        assert!(!result.is_empty());
    }

    #[test]
    fn check_piece_valid() {
        let mut hasher = Sha1::new();
        hasher.update(vec![0; 1024]);
        let piece_hash: &[u8] = &hasher.finalize();  

        // Create a mock Torrent instance
        let torrent = Torrent {
            info: Info {
                name: String::from("test_torrent"),
                pieces: piece_hash.into(), // Mock piece hashes
                piece_length: 1024,
                length: Some(2048),
                files: None,
                md5sum: None,
                private: None,
                path: None,
                root_hash: None,
            },
            announce: Some(String::from("http://tracker.example.com/announce")),
            nodes: None,
            encoding: None,
            httpseeds: None,
            announce_list: None,
            creation_date: None,
            comment: None,
            created_by: None,
        };

        // Mock a valid piece
        let piece = vec![0; 1024];

        let result = torrent.check_piece(&piece, 0);

        assert!(result);
    }

    #[test]
    fn check_piece_invalid() {
        // Create a mock Torrent instance
        let torrent = Torrent {
            info: Info {
                name: String::from("test_torrent"),
                pieces: vec![0; 20], // Mock piece hashes
                piece_length: 1024,
                length: Some(2048),
                files: None,
                md5sum: None,
                private: None,
                path: None,
                root_hash: None,
            },
            announce: Some(String::from("http://tracker.example.com/announce")),
            nodes: None,
            encoding: None,
            httpseeds: None,
            announce_list: None,
            creation_date: None,
            comment: None,
            created_by: None,
        };

        // Mock an invalid piece
        let piece = vec![1; 1024];

        let result = torrent.check_piece(&piece, 0);

        assert!(!result);
    }

    #[test]
    fn get_total_length_single_file() {
        // Create a mock Torrent instance with a single file
        let torrent = Torrent {
            info: Info {
                name: String::from("test_torrent"),
                pieces: vec![],
                piece_length: 1024,
                length: Some(2048),
                files: Some(vec![File {
                    path: vec![String::from("test_file.txt")],
                    length: 2048,
                    md5sum: None,
                }]),
                md5sum: None,
                private: None,
                path: None,
                root_hash: None,
            },
            announce: Some(String::from("http://tracker.example.com/announce")),
            nodes: None,
            encoding: None,
            httpseeds: None,
            announce_list: None,
            creation_date: None,
            comment: None,
            created_by: None,
        };

        let result = torrent.get_total_length();

        assert_eq!(result, 2048);
    }

    #[test]
    fn get_total_length_multiple_files() {
        // Create a mock Torrent instance with multiple files
        let torrent = Torrent {
            info: Info {
                name: String::from("test_torrent"),
                pieces: vec![],
                piece_length: 1024,
                length: None,
                files: Some(vec![
                    File {
                        path: vec![String::from("file1.txt")],
                        length: 1024,
                        md5sum: None,
                    },
                    File {
                        path: vec![String::from("file2.txt")],
                        length: 2048,
                        md5sum: None,
                    },
                ]),
                md5sum: None,
                private: None,
                path: None,
                root_hash: None,
            },
            announce: Some(String::from("http://tracker.example.com/announce")),
            nodes: None,
            encoding: None,
            httpseeds: None,
            announce_list: None,
            creation_date: None,
            comment: None,
            created_by: None,
        };

        let result = torrent.get_total_length();

        assert_eq!(result, 3072);
    }

    // Add more tests for other methods and edge cases as needed
}