use log::{ info, error };

/// Represents the handshake message that will be sent to a client.
#[derive(Debug)]
pub struct Handshake {
    /// The length of the protocol name, must be 19 for "BitTorrent protocol".
    p_str_len: u8,
    /// The protocol name, should always be "BitTorrent protocol".
    p_str: String,
    /// Reserved for extensions, currently unused.
    reserved: [u8; 8],
    /// The infohash for the torrent.
    info_hash: Vec<u8>,
    /// The identifier for the client.
    pub peer_id: String,
}

impl Handshake {
    /// Creates a new handshake.
    ///
    /// # Arguments
    ///
    /// * `info_hash` - The infohash for the torrent.
    ///
    /// # Returns
    ///
    /// A new `Handshake` instance on success, or an empty `Result` indicating an error.
    pub fn new(info_hash: &[u8]) ->  Option<Self> {
        if info_hash.len() != 20 {
            error!("Incorrect infohash length, consider using the helper function in torrent");
            return None;
        }

        Some(Self {
            p_str_len: 19,
            p_str: String::from("BitTorrent protocol"),
            reserved: [0; 8],
            info_hash: info_hash.to_vec(),
            peer_id: String::from("-MY0001-123456654322")
        })
    }

    /// Converts the `Handshake` instance to a byte buffer for sending to a peer.
    ///
    /// # Returns
    ///
    /// A byte vector containing the serialized handshake.
    pub fn to_buffer(&self) -> Vec<u8> {
        let mut buf: Vec<u8> = vec![0; 68];

        buf[0] = self.p_str_len;
        buf[1..20].copy_from_slice(&self.p_str.as_bytes()[..19]);
        buf[21..28].copy_from_slice(&self.reserved[..7]);
        buf[28..48].copy_from_slice(&self.info_hash[..20]);
        buf[48..68].copy_from_slice(&self.peer_id.as_bytes()[..20]);

        buf
    }

    /// Converts a byte buffer to a `Handshake` instance.
    ///
    /// # Arguments
    ///
    /// * `buf` - A byte vector containing the serialized handshake.
    ///
    /// # Returns
    ///
    /// A new `Handshake` instance on success, or an empty `Result` indicating an error.
    ///
    /// # Errors
    ///
    /// Returns an error if the provided buffer is not long enough (at least 68 bytes).
    pub fn from_buffer(buf: &Vec<u8>) -> Option<Self> {
        // Verify that buffer is at least the correct size, if not error
        if buf.len() < 68 {
            error!("buffer provided to handshake was too short");
            return None;
        }

        let mut p_str = String::new();
        for byte in buf.iter().take(20).skip(1) {
            p_str.push(*byte as char)
        }

        let mut info_hash: Vec<u8> = vec![0; 20];
        info_hash[..20].copy_from_slice(&buf[28..48]);

        let mut peer_id = String::new();
        for byte in buf.iter().take(68).skip(48) {
            peer_id.push(*byte as char)
        }

        Some(Self { 
            p_str_len: buf[0], 
            p_str, 
            reserved: [0; 8], 
            info_hash, 
            peer_id 
        })
    }

    pub fn log_useful_information(&self) {
        info!("Connected - PeerId: {:?}", self.peer_id);
    }
}