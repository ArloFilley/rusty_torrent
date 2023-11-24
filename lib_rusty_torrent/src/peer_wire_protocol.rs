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
  pub fn new(info_hash: &[u8], peer_id: String) ->  Result<Self, String> {
    if info_hash.len() != 20 {
      return Err(String::from("Incorrect infohash length"));
    }
    
    if peer_id.len() != 20 {
        return Err(String::from("Incorrect Peer_Id Length"))
    }
    
    Ok(Self {
      p_str_len: 19,
      p_str: String::from("BitTorrent protocol"),
      reserved: [0; 8],
      info_hash: info_hash.to_vec(),
      peer_id: String::from("-MY0001-123456654321")
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
  pub fn from_buffer(buf: &Vec<u8>) -> Result<Self, String> {
    // Verify that buffer is at least the correct size, if not error
    if buf.len() < 68 {
      return Err(String::from("buffer provided to handshake was too short"));
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
    
    Ok(Self { 
      p_str_len: buf[0], 
      p_str, 
      reserved: [0; 8], 
      info_hash, 
      peer_id 
    })
  }
}

/// Represents a message in the BitTorrent protocol.
#[derive(Clone, Debug, PartialEq)]
pub struct Message {
    /// The length of the message, including the type and payload.
    pub message_length: u32,
    /// The type of message.
    pub message_type: MessageType,
    /// The payload of the message, if any.
    pub payload: Option<Vec<u8>>,
}

impl Message {
    /// Creates a new message.
    ///
    /// # Arguments
    ///
    /// * `message_length` - The length of the message.
    /// * `message_type` - The type of message.
    /// * `payload` - The payload of the message, if any.
    pub fn new(message_length: u32, message_type: MessageType, payload: Option<Vec<u8>>) -> Self {
        Self { message_length, message_type, payload }
    }
}

impl TryFrom<&[u8]> for Message {
    type Error = String;
    /// Decodes a message from a given buffer.
    ///
    /// # Arguments
    ///
    /// * `buf` - The byte buffer containing the serialized message.
    ///
    /// # Returns
    ///
    /// A new `Message` instance on success, or an empty `Result` indicating an error.
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let mut message_length: [u8; 4] = [0; 4];

        if value.len() < 5 {
            return Err(format!("Buffer not long enough to be a message: Length {}, should be at least 4 bytes", value.len()));
        }

        message_length[..4].copy_from_slice(&value[..4]);
        
        let message_length = u32::from_be_bytes(message_length); 
        
        let payload: Option<Vec<u8>>;
        let message_type: MessageType;
        
        if message_length == 0 {
            message_type = MessageType::KeepAlive;
            payload = None;
        } else if message_length == 5 {
            message_type = value[4].try_into()?;
            payload = None;
        } else {
            message_type = value[4].try_into()?;
            
            let end_of_message = 4 + message_length as usize;
            
            if end_of_message > value.len() {
                return Err(format!("Invalid message length {} expected {}", value.len(), end_of_message))
            } else {
                payload = Some(value[5..end_of_message].to_vec());
            } 
        }
        
        Ok(Self {
            message_length,
            message_type,
            payload
        })
    }
}


impl TryFrom<Message> for Vec<u8> {
    type Error = String;
    /// Converts the `Message` instance to a byte buffer for sending.
    ///
    /// # Returns
    ///
    /// A byte vector containing the serialized message.
    fn try_from(value: Message) -> Result<Self, Self::Error> {
        let mut buf: Vec<u8> = vec![];
        
        for byte in value.message_length.to_be_bytes() {
            buf.push(byte);
        }
        
        match value.message_type {
            MessageType::KeepAlive => { 
                return Ok(buf)
            },
            MessageType::Choke | MessageType::Unchoke | MessageType::Interested | MessageType::NotInterested => { 
                buf.push(value.message_type.try_into()?);
                return Ok(buf);
            },
            MessageType::Have | MessageType::Bitfield | MessageType::Request | MessageType::Piece | MessageType::Cancel | MessageType::Port => { 
                buf.push(value.message_type.try_into()?);
            },
        }
        
        match value.payload {
            None => { 
                return Err(String::from("Error you are trying to create a message that needs a payload with no payload")) 
            }
            Some(payload) => {
                buf.extend(payload);
            }
        }
        
        Ok(buf)
    }
}

impl Message {
    /// Create a request message from a given piece_index, offset, and length
    /// 
    /// # Arguments
    /// 
    /// * `piece_index` - The index of the piece in the torrent
    /// * `offset` - The offset within the piece, because requests should be no more than 16KiB
    /// * `length` - The length of the piece request, should be 16KiB
    /// 
    /// # Returns 
    /// 
    /// A piece request message
    pub fn create_piece_request(piece_index: u32, offset: u32, length: u32) -> Self {
        let mut payload: Vec<u8> = vec![];
        
        for byte in piece_index.to_be_bytes() {
            payload.push(byte);
        }
        
        for byte in offset.to_be_bytes() {
            payload.push(byte)
        }
        
        for byte in length.to_be_bytes() {
            payload.push(byte)
        }
        
        Self { 
            message_length: 13, 
            message_type: MessageType::Request, 
            payload: Some(payload) 
        }
    }
    
    /// Returns the number of messages in the given buffer and their contents.
    ///
    /// # Arguments
    ///
    /// * `buf` - The byte buffer containing multiple serialized messages.
    ///
    /// # Returns
    ///
    /// A tuple containing a vector of message byte buffers and the number of messages.
    pub fn number_of_messages(buf: &[u8]) -> (Vec<Vec<u8>>, u32) {
        let mut message_num = 0;
        let mut messages: Vec<Vec<u8>> = vec![];
        
        // Find the length of message one
        // put that into an array and increment counter by one
        let mut i = 0; // points to the front
        let mut j; // points to the back
        
        loop {
            j = u32::from_be_bytes([buf[i], buf[i + 1], buf[i + 2], buf[i + 3]]) as usize + 4;
            
            messages.push(buf[i..i+j].to_vec());
            i += j;
            message_num += 1;
            
            if buf[i] == 0 && buf[i + 1] == 0 && buf[i + 2] == 0 && buf[i + 3] == 0 {
                break;
            }
        }
        
        (messages, message_num)
    }
}

/// An enum representing all possible message types in the BitTorrent peer wire protocol.
#[derive(Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum MessageType {
    /// Keepalive message, 0 length.
    /// Potential Errors if trying to handle a keepalive message like another message.
    /// Due to length being 0, should always be explicitly handled.
    KeepAlive = u8::MAX,
    /// Message telling the client not to send any requests until the peer has unchoked, 1 length.
    Choke = 0,
    /// Message telling the client that it can send requests, 1 length.
    Unchoke = 1,
    /// Message indicating that the peer is still interested in downloading, 1 length.
    Interested = 2,
    /// Message indicating that the peer is not interested in downloading, 1 length.
    NotInterested = 3,
    /// Message indicating that the peer has a given piece, fixed length.
    Have = 4,
    /// Message sent after a handshake, represents the pieces that the peer has.
    Bitfield = 5,
    /// Request a given part of a piece based on index, offset, and length, 13 length.
    Request = 6,
    /// A response to a request with the accompanying data, varying length.
    Piece = 7,
    /// Cancels a request, 13 length.
    Cancel = 8,
    /// Placeholder for unimplemented message type.
    Port = 9,
}

impl TryFrom<MessageType> for u8 {
    type Error = String;
    fn try_from(value: MessageType) -> Result<Self, Self::Error> {
        match value {
            MessageType::Choke => Ok(0),
            MessageType::Unchoke => Ok(1),
            MessageType::Interested => Ok(2),
            MessageType::NotInterested => Ok(3),
            MessageType::Have => Ok(4),
            MessageType::Bitfield => Ok(5),
            MessageType::Request => Ok(6),
            MessageType::Piece => Ok(7),
            MessageType::Cancel => Ok(8),
            MessageType::Port => Ok(9),
            _ => {
                Err(format!("Invalid Message Type {:?}", value))
            }
        }
    }
}

impl TryFrom<u8> for MessageType {
    type Error = String;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(MessageType::Choke),
            1 => Ok(MessageType::Unchoke),
            2 => Ok(MessageType::Interested),
            3 => Ok(MessageType::NotInterested),
            4 => Ok(MessageType::Have),
            5 => Ok(MessageType::Bitfield),
            6 => Ok(MessageType::Request),
            7 => Ok(MessageType::Piece),
            8 => Ok(MessageType::Cancel),
            9 => Ok(MessageType::Port),
            _ => {
                Err(format!("Invalid Message Type {}", value))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handshake_creation() {
        let info_hash: [u8; 20] = [1; 20];
        let peer_id = String::from("-MY0001-123456654321");

        match Handshake::new(&info_hash, peer_id.clone()) {
            Ok(handshake) => {
                assert_eq!(handshake.p_str_len, 19);
                assert_eq!(handshake.p_str, "BitTorrent protocol");
                assert_eq!(handshake.reserved, [0; 8]);
                assert_eq!(handshake.info_hash, info_hash.to_vec());
                assert_eq!(handshake.peer_id, peer_id);
            }
            Err(_) => panic!("Unexpected error creating handshake"),
        }
    }

    #[test]
    fn handshake_creation_invalid_infohash() {
        let invalid_info_hash: [u8; 19] = [1; 19];
        let peer_id = String::from("-MY0001-123456654321");

        match Handshake::new(&invalid_info_hash, peer_id.clone()) {
            Err(err) => assert_eq!(err, "Incorrect infohash length"),
            Ok(_) => panic!("Expected an error creating handshake, but got Ok"),
        }
    }

    #[test]
    fn handshake_creation_invalid_peer_id() {
        let info_hash: [u8; 20] = [1; 20];
        let invalid_peer_id = String::from("-INVALID");

        match Handshake::new(&info_hash, invalid_peer_id) {
            Err(err) => assert_eq!(err, "Incorrect Peer_Id Length"),
            Ok(_) => panic!("Expected an error creating handshake, but got Ok"),
        }
    }

    #[test]
    fn handshake_to_buffer() {
        let info_hash: [u8; 20] = [1; 20];
        let peer_id = String::from("-MY0001-123456654321");
        let handshake = Handshake::new(&info_hash, peer_id).unwrap();
        let buffer = handshake.to_buffer();

        assert_eq!(buffer.len(), 68);
        // Add more assertions based on the expected structure of the buffer if needed
    }

    #[test]
    fn handshake_from_buffer() {
        let info_hash: [u8; 20] = [1; 20];
        let peer_id = String::from("-MY0001-123456654321");
        let original_handshake = Handshake::new(&info_hash, peer_id.clone()).unwrap();
        let buffer = original_handshake.to_buffer();

        match Handshake::from_buffer(&buffer) {
            Ok(handshake) => assert_eq!(handshake.peer_id, peer_id),
            Err(err) => panic!("Unexpected error: {}", err),
        }
    }

    #[test]
    fn handshake_from_buffer_invalid_size() {
        let short_buffer: Vec<u8> = vec![0; 67]; // Invalid size
        match Handshake::from_buffer(&short_buffer) {
            Err(err) => assert_eq!(err, "buffer provided to handshake was too short"),
            Ok(_) => panic!("Expected an error, but got Ok"),
        }
    }

    #[test]
    fn u8_to_message_type() {
        assert_eq!(TryInto::<MessageType>::try_into(0 as u8), Ok(MessageType::Choke));
        assert_eq!(TryInto::<MessageType>::try_into(1 as u8), Ok(MessageType::Unchoke));
        assert_eq!(TryInto::<MessageType>::try_into(2 as u8), Ok(MessageType::Interested));
        assert_eq!(TryInto::<MessageType>::try_into(3 as u8), Ok(MessageType::NotInterested));
        assert_eq!(TryInto::<MessageType>::try_into(4 as u8), Ok(MessageType::Have));
        assert_eq!(TryInto::<MessageType>::try_into(5 as u8), Ok(MessageType::Bitfield));
        assert_eq!(TryInto::<MessageType>::try_into(6 as u8), Ok(MessageType::Request));
        assert_eq!(TryInto::<MessageType>::try_into(7 as u8), Ok(MessageType::Piece));
        assert_eq!(TryInto::<MessageType>::try_into(8 as u8), Ok(MessageType::Cancel));
        assert_eq!(TryInto::<MessageType>::try_into(9 as u8), Ok(MessageType::Port));
        assert_eq!(TryInto::<MessageType>::try_into(10 as u8), Err(String::from("Invalid Message Type 10")));
    }

    #[test]
    fn message_type_to_u8() {
        assert_eq!(TryInto::<u8>::try_into(MessageType::Choke),         Ok(0 as u8));
        assert_eq!(TryInto::<u8>::try_into(MessageType::Unchoke),       Ok(1 as u8));
        assert_eq!(TryInto::<u8>::try_into(MessageType::Interested),    Ok(2 as u8));
        assert_eq!(TryInto::<u8>::try_into(MessageType::NotInterested), Ok(3 as u8));
        assert_eq!(TryInto::<u8>::try_into(MessageType::Have),          Ok(4 as u8));
        assert_eq!(TryInto::<u8>::try_into(MessageType::Bitfield),      Ok(5 as u8));
        assert_eq!(TryInto::<u8>::try_into(MessageType::Request),       Ok(6 as u8));
        assert_eq!(TryInto::<u8>::try_into(MessageType::Piece),         Ok(7 as u8));
        assert_eq!(TryInto::<u8>::try_into(MessageType::Cancel),        Ok(8 as u8));
        assert_eq!(TryInto::<u8>::try_into(MessageType::Port),          Ok(9 as u8));
        assert_eq!(TryInto::<u8>::try_into(MessageType::KeepAlive),     Err(String::from("Invalid Message Type KeepAlive")));
    }

    #[test]
    fn create_piece_request() {
        let piece_index = 42;
        let offset = 1024;
        let length = 16384;

        let request_message = Message::create_piece_request(piece_index, offset, length);

        assert_eq!(request_message.message_length, 13);
        assert_eq!(request_message.message_type, MessageType::Request);

        if let Some(payload) = request_message.payload {
            assert_eq!(payload.len(), 12); // 4 bytes for piece_index + 4 bytes for offset + 4 bytes for length

            let mut expected_payload = vec![];
            expected_payload.extend_from_slice(&piece_index.to_be_bytes());
            expected_payload.extend_from_slice(&offset.to_be_bytes());
            expected_payload.extend_from_slice(&length.to_be_bytes());

            assert_eq!(payload, expected_payload);
        } else {
            panic!("Expected payload, but found None");
        }
    }

    #[test]
    fn try_from_valid_message() {
        let message_bytes = vec![0, 0, 0, 5, 1]; // Unchoke message

        match Message::try_from(&message_bytes[..]) {
            Ok(message) => {
                assert_eq!(message.message_length, 5);
                assert_eq!(message.message_type, MessageType::Unchoke);
                assert!(message.payload.is_none());
            }
            Err(err) => panic!("Unexpected error: {}", err),
        }
    }

    #[test]
    fn try_from_invalid_message() {
        let invalid_message_bytes = vec![0, 0, 0, 2]; // Message length indicates 2 bytes, but no payload provided

        match Message::try_from(&invalid_message_bytes[..]) {
            Ok(_) => panic!("Expected an error but got Ok"),
            Err(err) => {
                assert_eq!(
                    err,
                    "Buffer not long enough to be a message: Length 4, should be at least 4 bytes"
                );
            }
        }
    }

    #[test]
    fn try_into_valid_message() {
        let message = Message {
            message_length: 5,
            message_type: MessageType::Unchoke,
            payload: None,
        };

        match Vec::<u8>::try_from(message) {
            Ok(serialized_message) => {
                assert_eq!(serialized_message, vec![0, 0, 0, 5, 1]); // Unchoke message
            }
            Err(err) => panic!("Unexpected error: {}", err),
        }
    }

    #[test]
    fn try_into_message_with_payload() {
        let payload_data = vec![65, 66, 67]; // Arbitrary payload
        let message = Message {
            message_length: 7,
            message_type: MessageType::Piece,
            payload: Some(payload_data.clone()),
        };

        match Vec::<u8>::try_from(message) {
            Ok(serialized_message) => {
                let mut expected_serialized_message = vec![0, 0, 0, 7, 7]; // Piece message
                expected_serialized_message.extend_from_slice(&payload_data);

                assert_eq!(serialized_message, expected_serialized_message);
            }
            Err(err) => panic!("Unexpected error: {}", err),
        }
    }
}