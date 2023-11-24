//! Contains Structures and associated methods to abstract interaction with the peer

// Crate Imports
use crate::{
    peer_wire_protocol::{ Handshake, Message, MessageType }, 
    torrent::Torrent
};

// External imports
use std::net::SocketAddrV4;
use tokio::{
    io::{ AsyncReadExt, AsyncWriteExt },
    net::TcpStream
};

/// Structure to abstract interaction with a peer.
pub struct Peer {
    /// The `TcpStream` that is used to communicate with the peeer
    connection_stream: TcpStream,
    /// The `SocketAddr` of the peer
    pub socket_addr: SocketAddrV4,
    /// The id of the peer
    pub peer_id: String,
    /// Whether the peer is choking the client
    pub choking: bool,
}

impl Peer {
    /// Creates a connection to the peer.
    ///
    /// # Arguments
    ///
    /// * `socket_address` - The socket address of the peer.
    pub async fn create_connection(socket_address: SocketAddrV4) -> Result<Self, String> {
        let connection_stream = match TcpStream::connect(socket_address).await {
            Err(err) => {
                return Err(format!("unable to connect to {}, err: {}", socket_address, err))
            },
            Ok(stream) => {
                stream
            }
        };
        
        Ok(Self {
            connection_stream,
            socket_addr: socket_address,
            peer_id: String::new(),
            choking: true,
        })
    }
}

impl Peer {
    /// Sends a handshake message to the peer, the first step in the peer wire messaging protocol.
    ///
    /// # Arguments
    ///
    /// * `torrent` - The `Torrent` instance associated with the peer.
    pub async fn handshake(&mut self, torrent: &Torrent) -> Result<(), String>{
        let mut buf = vec![0; 1024];
        
        let handshake_message = Handshake::new(&torrent.get_info_hash(), String::from("-RT0001-123456012345")).unwrap();
        
        self.connection_stream.writable().await.unwrap();
        self.connection_stream.write_all(&handshake_message.to_buffer()).await.unwrap();
        
        self.connection_stream.readable().await.unwrap();
        let _ = self.connection_stream.read(&mut buf).await.unwrap();
        
        let handshake = Handshake::from_buffer(&buf[..68].to_vec()).unwrap();
        
        for message_buf in Message::number_of_messages(&buf[68..]).0 {
            let message: Message = (&*message_buf).try_into()?;
            
            if message.message_type == MessageType::Unchoke {
                self.choking = false;
            }
        }
        
        self.peer_id = handshake.peer_id;

        Ok(())
    }
    
    /// Keeps the connection alive and sends interested messages until the peer unchokes
    pub async fn keep_alive_until_unchoke(&mut self) -> Result<(), String> {
        loop {
            let message = self.read_message().await?;
            
            match message.message_type {
                MessageType::Unchoke => {
                    self.choking = false;
                    break
                }
                MessageType::KeepAlive => {
                    self.send_message_no_response(Message::new(0, MessageType::KeepAlive, None)).await?;
                    self.send_message_no_response(Message::new(1, MessageType::Interested, None)).await?;
                }
                MessageType::Choke => {
                    self.choking = true;
                }
                _ => { continue }
            }
        }

        Ok(())
    }
    
    /// Sends a message to the peer and waits for a response, which it returns
    pub async fn send_message(&mut self, message: Message) -> Result<Message, String> {
        let mut response = vec![0; 16_397];

        let message: Vec<u8> = message.try_into()?;
        
        self.connection_stream.writable().await.unwrap();
        self.connection_stream.write_all(&message).await.unwrap();
        
        self.connection_stream.readable().await.unwrap();
        let _ = self.connection_stream.read_exact(&mut response).await.unwrap();
        
        Ok((*response).try_into()?)
    }
    
    /// Sends a message to the peer and waits for a response, which it returns
    pub async fn send_message_exact_size_response(&mut self, message: Message, size: usize) -> Result<Message, String> {
        let mut response = vec![0; size];

        let message: Vec<u8> = message.try_into()?;
        
        self.connection_stream.writable().await.unwrap();
        self.connection_stream.write_all(&message).await.unwrap();
        
        self.connection_stream.readable().await.unwrap();
        let _ = self.connection_stream.read_exact(&mut response).await.unwrap();
        
        Ok((*response).try_into()?)
    }
    
    /// Sends a message but doesn't wait for a response
    pub async fn send_message_no_response(&mut self, message: Message) -> Result<(), String> {

        let message: Vec<u8> = message.try_into()?;
        self.connection_stream.writable().await.unwrap();
        self.connection_stream.write_all(&message).await.unwrap();

        Ok(())
    }
    
    /// reads a message from the peer
    pub async fn read_message(&mut self) -> Result<Message, String> {
        let mut response = vec![0; 16_397];
        
        self.connection_stream.readable().await.unwrap();
        let _ = self.connection_stream.read(&mut response).await.unwrap();
        
        Ok((*response).try_into()?)
    }
    
    /// Shutsdown the connection stream
    pub async fn disconnect(&mut self) -> Result<(), String>{
        match self.connection_stream.shutdown().await {
            Err(err) => {
                return Err(format!("Error disconnecting from {}: {}", self.socket_addr, err));
            },
            Ok(_) => {
                Ok(())
            }
        }
    }
}

impl Peer {
    // Sends the requests and reads responses to put a piece together
    pub async fn request_piece(&mut self, index: u32, piece_length: u32, len: &mut u32, total_len: u32) -> Result<Vec<u8>, String> {
        let mut buf = vec![];
        // Sequentially requests piece from the peer
        for offset in (0..piece_length).step_by(16_384) {
            let mut length = 16_384;
            
            let response: Message;
            
            if *len + 16_384 >= total_len {
                length = total_len - *len;
                
                response = self.send_message_exact_size_response(
                    Message::create_piece_request(index, offset, length),
                    length as usize + 13
                ).await?;
            } else {
                response = self.send_message(Message::create_piece_request(index, offset, length)).await?;
            };
            
            match response.message_type {
                MessageType::Piece => {
                    let mut data = response.payload.unwrap();
                    *len += data.len() as u32;
                    *len -= 8;
                    
                    for byte in data.drain(..).skip(8) {
                        buf.push(byte)
                    }
                },
                _ => { }
            };
            
            if *len >= total_len - 1 {
                return Ok(buf);
            }
        }
        
        Ok(buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::torrent::Torrent;
    use std::net::Ipv4Addr;

    #[tokio::test]
    async fn peer_create_connection() {
        // Replace the IP and port with the actual values
        let socket_address = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 6881);

        match Peer::create_connection(socket_address).await {
            Ok(peer) => {
                assert_eq!(peer.socket_addr, socket_address);
                // Add more assertions if needed
            }
            Err(err) => panic!("Unexpected error: {}", err),
        }
    }

    #[tokio::test]
    async fn peer_handshake() {
        let socket_address = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 6881);
        let mut peer = Peer::create_connection(socket_address.clone()).await.unwrap();
        let torrent = Torrent::from_torrent_file("test.torrent").await.unwrap();

        assert!(peer.handshake(&torrent).await.is_ok());
    }

    // Add more tests for other methods in the Peer structure
}
