use log::{debug, info, error, warn, trace};
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
    pub async fn from_torrent_file(path: &str) -> Self {
        info!("");
        info!("-->      Reading File         <--");

        let Ok(mut file) = TokioFile::open(path).await else {
            error!("Unable to read file at {path}");
            panic!("Unable to read file at {path}");
        };
        info!("Found\t\t > {path}");

        let mut buf: Vec<u8> = Vec::new();
        let Ok(_) = file.read_to_end(&mut buf).await else {
            error!("Error reading file > {path}");
            panic!("Error reading file > {path}");
        };
        info!("Read\t\t > {path}");
        
        let Ok(torrent) = serde_bencode::from_bytes(&buf) else {
            error!("Error deserializing file > {path}");
            panic!("Error deserializing file > {path}");
        };
        info!("Parsed\t > {path}");

        torrent
    }
}
    
impl Torrent {
    /// Logs info about the *.torrent file
    pub fn log_useful_information(&self) {
        info!("");
        info!("-->    Torrent Information    <--");
        info!("Name:\t\t{}", self.info.name);
        info!("Trackers");
        if let Some(trackers) = &self.announce_list {
            for tracker in trackers {
                info!("  |>  {}", tracker[0])
            }
        }
        info!("InfoHash:\t{:X?}", self.get_info_hash());
        info!("Length:\t{:?}", self.info.length);
        
        info!("Files:");
        let Some(mut files) = self.info.files.clone() else {
            info!("./{}", self.info.name);
            return
        };

        files.sort_by(|a, b| a.path.len().cmp(&b.path.len()) );
        info!("./");
        for file in files {
            if file.path.len() == 1 {
                info!(" |--> {:?}", file.path);
            } else {
                let mut path = String::new();
                file.path.iter().for_each(|s| { path.push_str(s); path.push('/') } );
                path.pop();

                info!(" |--> {}: {}B", path, file.length)
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
            info!("Piece {}/{} Correct!", index + 1, self.info.pieces.len() / 20);
            true
        } else {
            error!("Piece {}/{} incorrect :(",  index + 1, self.info.pieces.len() / 20);
            debug!("{:?}", &result[..]);
            debug!("{:?}", piece_hash);
            debug!("{:?}", &result[..].len());
            debug!("{:?}", piece_hash.len());
            debug!("{}", piece.len());
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
    
    pub fn get_trackers(&self) -> Option<Vec<SocketAddrV4>> {
        info!("");
        info!("-->      Locating Trackers    <--");

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
            } else {
                warn!("{url} does not match the expected url pattern");
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
                        info!("Sucessfully found tracker {}", url[0]);
                    }
                } else {
                    warn!("{} does not match the expected url pattern", url[0]);
                }
            }
        }
        
        if addresses.len() > 0 {
            Some(addresses)
        } else {
            None
        }
    }
}