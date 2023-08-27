//! Contains Structures and associated methods to abstract interaction with the peer

// Crate Imports
use crate::{
    handshake::Handshake,
    message::{ Message, MessageType }, 
    torrent::Torrent
};

// External imports
use log::{ debug, error };
use std::net::SocketAddr;
use tokio::{
    io::{ AsyncReadExt, AsyncWriteExt },
    net::TcpStream
};

/// Structure to abstract interaction with a peer.
pub struct Peer {
    /// The `TcpStream` that is used to communicate with the peeer
    connection_stream: TcpStream,
    /// The `SocketAddr` of the peer
    pub socket_addr: SocketAddr,
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
    pub async fn create_connection(socket_address: &str) -> Option<Self> {
        let socket_addr = match socket_address.parse::<SocketAddr>() {
            Err(err) => {
                error!("error parsing address {}, err: {}", socket_address, err);
                return None;
            }
            Ok(addr) => { addr }
        };

        let connection_stream = match TcpStream::connect(socket_addr).await {
            Err(err) => {
                error!("unable to connect to {}, err: {}", socket_address, err);
                return None
            },
            Ok(stream) => {
                debug!("created tcpstream successfully to: {socket_addr}");
                stream
            }
        };

        Some(Self {
            connection_stream,
            socket_addr,
            peer_id: String::new(),
            choking: true,
        })
    }

    /// Sends a handshake message to the peer, the first step in the peer wire messaging protocol.
    ///
    /// # Arguments
    ///
    /// * `torrent` - The `Torrent` instance associated with the peer.
    pub async fn handshake(&mut self, torrent: &Torrent) {
        let mut buf = vec![0; 1024];

        let handshake_message = Handshake::new(torrent.get_info_hash()).unwrap();

        self.connection_stream.writable().await.unwrap();
        self.connection_stream.write_all(&handshake_message.to_buffer()).await.unwrap();

        self.connection_stream.readable().await.unwrap();
        let _ = self.connection_stream.read(&mut buf).await.unwrap();

        let handshake = Handshake::from_buffer(&buf[..68].to_vec()).unwrap();
        handshake.log_useful_information();

        for message_buf in Message::number_of_messages(&buf[68..].to_vec()).0 {
            let message = match Message::from_buffer(&message_buf) {
                Err(()) => { 
                    error!("error decoding message");
                    self.disconnect().await;
                    return;
                },
                Ok(message) => { message }
            };

            match message.message_type {
                MessageType::Unchoke => {
                    self.choking = false;
                }
                _ => {}
            }
        }

        self.peer_id = handshake.peer_id;
    }

    /// Keeps the connection alive and sends interested messages until the peer unchokes
    pub async fn keep_alive_until_unchoke(&mut self) {
        loop {
            let message = match self.read_message().await {
                None => return,
                Some(message) => message
            };

            debug!("{message:?}");
            match message.message_type {
                MessageType::Unchoke => {
                    self.choking = false;
                    break
                }
                MessageType::KeepAlive => {
                    self.send_message_no_response(Message::new(0, MessageType::KeepAlive, None)).await;
                    self.send_message_no_response(Message::new(1, MessageType::Interested, None)).await;
                }
                MessageType::Choke => {
                    self.choking = true;
                }
                _ => { continue }
            }
        }
    }

    /// Sends a message to the peer and waits for a response, which it returns
    pub async fn send_message(&mut self, mut message: Message) -> Message {
        let mut buf = vec![0; 16_397];

        self.connection_stream.writable().await.unwrap();
        self.connection_stream.write_all(&message.to_buffer()).await.unwrap();

        self.connection_stream.readable().await.unwrap();
        let _ = self.connection_stream.read_exact(&mut buf).await.unwrap();

        Message::from_buffer(&buf).unwrap()
    }

    /// Sends a message to the peer and waits for a response, which it returns
    pub async fn send_message_exact_size_response(&mut self, mut message: Message, size: usize) -> Message {
        let mut buf = vec![0; size];

        self.connection_stream.writable().await.unwrap();
        self.connection_stream.write_all(&message.to_buffer()).await.unwrap();

        self.connection_stream.readable().await.unwrap();
        let _ = self.connection_stream.read_exact(&mut buf).await.unwrap();

        Message::from_buffer(&buf).unwrap()
    }

    /// Sends a message but doesn't wait for a response
    pub async fn send_message_no_response(&mut self, mut message: Message) {
        self.connection_stream.writable().await.unwrap();
        self.connection_stream.write_all(&message.to_buffer()).await.unwrap();
    }
    
    /// reads a message from the peer
    pub async fn read_message(&mut self) -> Option<Message> {
        let mut buf = vec![0; 16_397];

        self.connection_stream.readable().await.unwrap();
        let _ = self.connection_stream.read(&mut buf).await.unwrap();

        match Message::from_buffer(&buf) {
            Err(()) => {
                error!("Unable to decode message");
                self.disconnect().await;
                return None;
            },
            Ok(message) => {
                Some(message)
            }
        }
    }

    /// Shutsdown the connection stream
    pub async fn disconnect(&mut self) {
        match self.connection_stream.shutdown().await {
            Err(err) => {
                error!("Error disconnecting from {}: {}", self.socket_addr, err);
                panic!("Error disconnecting from {}: {}", self.socket_addr, err);
            },
            Ok(_) => {
                debug!("Successfully disconnected from {}", self.socket_addr)
            }
        }
    }
}

impl Peer {
    // Sends the requests and reads responses to put a piece together
    pub async fn request_piece(&mut self, index: u32, piece_length: u32, len: &mut u32, total_len: u32) -> Vec<u8> {
        let mut buf = vec![];
        // Sequentially requests piece from the peer
        for offset in (0..piece_length).step_by(16_384 as usize) {
            
            let mut length = 16_384;

            let response: Message;

            if *len + 16_384 >= total_len {
                debug!("Final Request {}", total_len - *len);
                length = total_len - *len;

                response = self.send_message_exact_size_response(
                    Message::create_request(index, offset, length),
                    length as usize + 13
                ).await;
            } else {
                response = self.send_message(Message::create_request(index, offset, length)).await;
            };
            
            match response.message_type {
                MessageType::Piece => {
                    let data = response.payload.unwrap();
                    *len += data.len() as u32;
                    *len -= 8;
                    
                    for i in 8..data.len() {
                        buf.push(data[i])
                    }
                },
                _ => { debug!("didn't recieve expected piece request | Recieved: {:?}", response.message_type); }
            };

            if *len >= total_len - 1 {
                return buf;
            }
        }

        buf
    }
}