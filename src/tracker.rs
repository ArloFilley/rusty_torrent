use std::{net::{SocketAddr, Ipv4Addr}, vec};

use tokio::net::UdpSocket;
use log::{debug, error};

pub struct Tracker {
    /// A UdpSocket used for communication.
    connection_stream: UdpSocket,
    /// The local socket address requests are made from
    pub socket_addr: SocketAddr,
    /// The remote socket address of the tracker.
    pub remote_addr: SocketAddr
}

impl Tracker {
    /// Creates a new `Tracker` instance asynchronously.
    ///
    /// # Arguments
    ///
    /// * `socket_address` - Local socket address for binding.
    /// * `remote_hostname` - Remote host for connection.
    /// * `remote_port` - Remote port for connection.
    ///
    /// # Panics
    ///
    /// Panics if there is an error parsing the given address or creating the UDP socket.
    pub async fn new(socket_address: &str, remote_hostname: &str, remote_port: u16) -> Self {
        let socket_addr = match socket_address.parse::<SocketAddr>() {
            Err(err) => {
                error!("error parsing address {}, err: {}", socket_address, err);
                panic!("error parsing given address, {}", err);
            }
            Ok(addr) => { addr }
        };

        let remote_address = dns_lookup::lookup_host(remote_hostname).unwrap()[0];

        let remote_addr = SocketAddr::new(remote_address, remote_port);

        let connection_stream = match UdpSocket::bind(socket_addr).await {
            Err(err) => {
                error!("unable to bind to {}, err: {}", socket_address, err);
                panic!("error creating udpsocket, {}", err);
            },
            Ok(stream) => {
                debug!("bound udpsocket successfully to: {socket_addr}");
                stream
            }
        };

        match connection_stream.connect(remote_addr).await {
            Err(err) => {
                error!("unable to connect to {}, err: {}", remote_addr, err);
                panic!("error creating udpsocket, {}", err);
            },
            Ok(()) => {
                debug!("successfully connected to: {remote_addr}");
            }
        };


        Self {
            connection_stream,
            socket_addr,
            remote_addr
        }
    }

    /// Sends a message to the tracker and receives a response asynchronously.
    ///
    /// # Arguments
    ///
    /// * `message` - A type that implements the `ToBuffer` trait, representing the message to send.
    ///
    /// # Returns
    ///
    /// A byte vector containing the received response.
    pub async fn send_message<T: ToBuffer>(&mut self, message: &T) -> Vec<u8> {
        let mut buf: Vec<u8> = vec![ 0; 16_384 ];

        self.connection_stream.send(&message.to_buffer()).await.unwrap();
        self.connection_stream.recv(&mut buf).await.unwrap();

        buf
    }
}

/// A trait for converting a type into a byte buffer.
pub trait ToBuffer {
    /// Converts the implementing type into a byte buffer.
    fn to_buffer(&self) -> Vec<u8>;
}

/// A trait for converting a type from a byte buffer.
pub trait FromBuffer {
    /// Converts a byte buffer into the implementing type.
    fn from_buffer(buf: &Vec<u8>) -> Self;
}

#[derive(Debug)]
/// Represents a basic connection message.
pub struct ConnectionMessage {
    pub connection_id: i64,
    action: i32,
    transaction_id: i32,
}

impl ConnectionMessage {
    /// Creates a new basic connection message
    pub fn create_basic_connection() -> Self {
        Self { 
            connection_id: 4497486125440,
            action: 0, 
            transaction_id: 123 
        }
    }
}

impl ToBuffer for ConnectionMessage {
    fn to_buffer(&self) -> Vec<u8> {
        let mut buf: Vec<u8> = vec![];

        buf.extend(self.connection_id.to_be_bytes());
        buf.extend(self.action.to_be_bytes());
        buf.extend(self.transaction_id.to_be_bytes());

        buf
    }
}

impl FromBuffer for ConnectionMessage {
    fn from_buffer(buf: &Vec<u8>) -> Self {
        let mut action: [u8; 4] = [0; 4];
        for i in 0..4 {
            action[i] = buf[i];
        }
        let action = i32::from_be_bytes(action);

        let mut transaction_id: [u8; 4] = [0; 4];
        for i in 4..8 {
            transaction_id[i - 4] = buf[i];
        }
        let transaction_id = i32::from_be_bytes(transaction_id);

        let mut connection_id: [u8; 8] = [0; 8];
        for i in 8..16 {
            connection_id[i - 8] = buf[i];
        }
        let connection_id = i64::from_be_bytes(connection_id);

        

        Self {
            connection_id,
            action,
            transaction_id
        }
    }
}

#[derive(Debug)]
/// Represents an announcement message in the bittorrent udp tracker protocol.
pub struct AnnounceMessage {
    connection_id: i64,
    action: i32,
    transaction_id: i32,
    info_hash: [u8; 20],
    peer_id: [u8; 20],
    downloaded: i64,
    left: i64,
    uploaded: i64,
    event: i32, 
    ip: u32,
    key: u32,
    num_want: i32,
    port: u16,
    extensions: u16
}

impl AnnounceMessage {
    /// Creates a new announce message.
    pub fn new(connection_id: i64, infohash: &Vec<u8>, peerid: &str, total_length: i64) -> Self {
        let mut info_hash: [u8; 20] = [0; 20];
        for i in 0..20 {
            info_hash[i] = infohash[i];
        }

        let mut peer_id: [u8; 20] = [0; 20];
        for i in 0..20 {
            peer_id[i] = peerid.chars().nth(i).unwrap() as u8;
        }

        Self { 
            connection_id, 
            action: 1, 
            transaction_id: 132,
            info_hash, 
            peer_id, 
            downloaded: 0, 
            left: total_length, 
            uploaded: 0, 
            event: 1, 
            ip: 0, 
            key: 234, 
            num_want: -1, 
            port: 61389, 
            extensions: 0
        }
    }
}

impl ToBuffer for AnnounceMessage {
    fn to_buffer(&self) -> Vec<u8> {
        let mut buf: Vec<u8> = vec![];

        buf.extend(self.connection_id.to_be_bytes());
        buf.extend(self.action.to_be_bytes());
        buf.extend(self.transaction_id.to_be_bytes());
        buf.extend(self.info_hash);
        buf.extend(self.peer_id);
        buf.extend(self.downloaded.to_be_bytes());
        buf.extend(self.left.to_be_bytes());
        buf.extend(self.uploaded.to_be_bytes());
        buf.extend(self.event.to_be_bytes());
        buf.extend(self.ip.to_be_bytes());
        buf.extend(self.key.to_be_bytes());
        buf.extend(self.num_want.to_be_bytes());
        buf.extend(self.port.to_be_bytes());
        buf.extend(self.extensions.to_be_bytes());

        buf
    }
}

#[derive(Debug)]
/// Represents a response to an announcement message.
pub struct AnnounceMessageResponse {
    pub action: i32,
    pub transaction_id: i32,
    pub interval: i32,
    pub leechers: i32,
    pub seeders: i32,
    pub ips: Vec<Ipv4Addr>,
    pub ports: Vec<u16>
}

impl FromBuffer for AnnounceMessageResponse {
    /// Converts a byte buffer into an `AnnounceMessageResponse` instance.
    fn from_buffer(buf: &Vec<u8>) -> Self {
        let mut action: [u8; 4] = [0; 4];
        for i in 0..4 {
            action[i] = buf[i];
        }
        let action = i32::from_be_bytes(action);

        let mut transaction_id: [u8; 4] = [0; 4];
        for i in 4..8 {
            transaction_id[i - 4] = buf[i];
        }
        let transaction_id = i32::from_be_bytes(transaction_id);

        let mut interval: [u8; 4] = [0; 4];
        for i in 8..12 {
            interval[i - 8] = buf[i];
        }
        let interval = i32::from_be_bytes(interval);

        let mut leechers: [u8; 4] = [0; 4];
        for i in 12..16 {
            leechers[i - 12] = buf[i];
        }
        let leechers = i32::from_be_bytes(leechers);

        let mut seeders: [u8; 4] = [0; 4];
        for i in 16..20 {
            seeders[i - 16] = buf[i];
        }
        let seeders = i32::from_be_bytes(seeders);

        let mut ips: Vec<Ipv4Addr> = vec![];
        let mut ports: Vec<u16> = vec![];

        for i in (20..buf.len()-6).step_by(6) {
            let ip = Ipv4Addr::new(buf[i], buf[i+1], buf[i+2], buf[i+3]);
            let port = u16::from_be_bytes([buf[i+4], buf[i+5]]);

            if ip.to_string() == "0.0.0.0" && port == 0 {
                break;
            }

            ips.push(ip);
            ports.push(port)
        }

        Self { action, transaction_id, interval, leechers, seeders, ips: ips[1..].to_vec(), ports: ports[1..].to_vec() }
    }
}