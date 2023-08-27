use log::error;
use std::vec;

/// Represents a message in the BitTorrent protocol.
#[derive(Debug, PartialEq)]
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

    /// Decodes a message from a given buffer.
    ///
    /// # Arguments
    ///
    /// * `buf` - The byte buffer containing the serialized message.
    ///
    /// # Returns
    ///
    /// A new `Message` instance on success, or an empty `Result` indicating an error.
    pub fn from_buffer(buf: &Vec<u8>) -> Result<Self, ()> {
        let mut message_length: [u8; 4] = [0; 4];
        for i in 0..4 {
            message_length[i] = buf[i];
        };

        let message_length = u32::from_be_bytes(message_length); 
        
        let payload: Option<Vec<u8>>;
        let message_type: MessageType;

        if message_length == 0 {
            message_type = MessageType::KeepAlive;
            payload = None;
        } else {
            message_type = match buf[4] {
                0 => MessageType::Choke,
                1 => MessageType::Unchoke,
                2 => MessageType::Interested,
                3 => MessageType::NotInterested,
                4 => MessageType::Have,
                5 => MessageType::Bitfield,
                6 => MessageType::Request,
                7 => MessageType::Piece,
                8 => MessageType::Cancel,
                9 => MessageType::Port,
                _ => {
                    error!("Invalid Message Type: {} | Message: {:?}", buf[4], buf);
                    return Err(())
                }
            };

            // if message_type == MessageType::Piece && 5 + message_length - 1 != 16397 {
            //     error!("{:?}", 5..5 + message_length as usize - 1);
            // }
            
            payload = Some(buf[5..5 + message_length as usize - 1].to_vec());
        }

        Ok(Self {
            message_length,
            message_type,
            payload
        })
    }

    /// Converts the `Message` instance to a byte buffer for sending.
    ///
    /// # Returns
    ///
    /// A byte vector containing the serialized message.
    pub fn to_buffer(&mut self) -> Vec<u8> {
        let mut buf: Vec<u8> = vec![];

        for byte in self.message_length.to_be_bytes() {
            buf.push(byte);
        }

        match self.message_type {
            MessageType::KeepAlive => { 
                return buf 
            },
            MessageType::Choke => { 
                buf.push(0);
                return buf;
            },
            MessageType::Unchoke => { 
                buf.push(1);
                return buf;
            },
            MessageType::Interested => { 
                buf.push(2);
                return buf;
            },
            MessageType::NotInterested => { 
                buf.push(3);
                return buf;
            },
            MessageType::Have => { 
                buf.push(4);
            },
            MessageType::Bitfield => { 
                buf.push(5);
            },
            MessageType::Request => { 
                buf.push(6);
            },
            MessageType::Piece => { 
                buf.push(7);
            },
            MessageType::Cancel => { 
                buf.push(8);
            },
            MessageType::Port => { 
                buf.push(9);
            },
        }

        match &self.payload {
            None => { panic!("Error you are trying to create a message that needs a payload with no payload") }
            Some(payload) => {
                for byte in payload {
                    buf.push(*byte)
                }

                buf
            }
        }
    }

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
    pub fn create_request(piece_index: u32, offset: u32, length: u32) -> Self {
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

        Self { message_length: 13, message_type: MessageType::Request, payload: Some(payload) }
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
    pub fn number_of_messages(buf: &Vec<u8>) -> (Vec<Vec<u8>>, u32) {
        let mut message_num = 0;
        let mut messages: Vec<Vec<u8>> = vec![];

        // Find the length of message one
        // put that into an array and increment counter by one
        let mut i = 0; // points to the front
        let mut j; // points to the back

        loop {
            j = u32::from_be_bytes([buf[i], buf[i + 1], buf[i + 2], buf[i + 3]]) as usize + 4;
            
            messages.push(buf[i..i+j].to_vec());
            i = i+j;
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