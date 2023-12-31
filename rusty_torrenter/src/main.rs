//! The root of the crate
//! 
//! Currently:
//! Creates the logger
//! Handles peer connection
//! Handles torrent information
//! Creates Files
//! Requests pieces
//! Checks piece hashes
//! Writes to torrent file

use std::net::SocketAddr;

// Crate Imports
use lib_rusty_torrent::{
    files::Files,
    peer::*,
    torrent::Torrent,
    tracker::Tracker,
    tracker::ConnectionMessage,
    tracker::AnnounceMessage,
    tracker::AnnounceMessageResponse,
    tracker::FromBuffer
};

// External Ipmorts
use clap::Parser;
use log::{ debug, info, LevelFilter };

/// Struct Respresenting needed arguments
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
  #[arg(short, long)]
  log_file_path: Option<String>,
  
  #[arg(short, long)]
  torrent_file_path: String,
  
  #[arg(short, long)]
  download_path: String,
}

/// The root function
#[tokio::main]
async fn main() {
  let args = Args::parse();
  
  // Creates a log file to handle large amounts of data
  let log_path = args.log_file_path.unwrap_or(String::from("./log/rustytorrent.log"));
  simple_logging::log_to_file(&log_path, LevelFilter::Info).unwrap();
  
  // Read the Torrent File
  let torrent = Torrent::from_torrent_file(&args.torrent_file_path).await.unwrap();
  info!("Sucessfully read torrent file");
  
  // Create the files that will be written to
  let mut files = Files::new();
  files.create_files(&torrent, &args.download_path).await;
  
  // Gets peers from the given tracker
  let addresses = torrent.get_trackers().unwrap();
  let (remote_hostname, remote_port) = ("tracker.opentrackr.org", 1337);
  debug!("{}:{}", remote_hostname, remote_port);
  
  let mut tracker = Tracker::new("0.0.0.0:61389".parse().unwrap(), SocketAddr::V4(addresses[0])).await.unwrap();
  info!("Successfully connected to tracker {}:{}", remote_hostname, remote_port);
  let connection_message = ConnectionMessage::from_buffer(
    &tracker.send_message(&ConnectionMessage::create_basic_connection()).await
  );
  
  debug!("{:?}", connection_message);
  
  let announce_message_response = AnnounceMessageResponse::from_buffer(
    &tracker.send_message(&AnnounceMessage::new(
      connection_message.connection_id, 
      &torrent.get_info_hash(), 
      "-MY0001-123456654321", 
      torrent.get_total_length() as i64
    )).await
  );
  
  debug!("{:?}", announce_message_response);
  info!("Found Peers");
  
  // Creates an assumed peer connection to the `SocketAddr` given
  let mut peer = match Peer::create_connection(format!("{}:{}", announce_message_response.ips[0], announce_message_response.ports[0]).parse().unwrap()).await {
    Err(_) => { return },
    Ok(peer) => peer
  }; 
  
  let num_pieces = torrent.info.pieces.len() / 20;
  peer.handshake(&torrent).await.unwrap();
  peer.keep_alive_until_unchoke().await.unwrap();
  
  info!("Successfully Created Connection with peer: {}", peer.peer_id);
  
  let mut len = 0;
  
  for index in 0..num_pieces {
    let piece= peer.request_piece(
      index as u32, torrent.info.piece_length as u32, 
      &mut len, torrent.get_total_length() as u32
    ).await.unwrap();
    
    if torrent.check_piece(&piece, index as u32) {
      files.write_piece(piece).await;
    } else {
      break
    }
  }
  
  peer.disconnect().await.unwrap();
  info!("Successfully completed download");
}