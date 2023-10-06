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

// Modules
mod files;
mod handshake;
mod peer;
mod message;
mod torrent;
mod tracker;

use core::panic;
use std::net::SocketAddr;

// Crate Imports
use crate::{
    files::Files,
    peer::Peer, 
    torrent::Torrent,
    tracker::tracker::Tracker
};

use tokio::sync::mpsc;
// External Ipmorts
use clap::Parser;
use log::{ debug, info, LevelFilter, error };
use tokio::spawn;

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
    
    #[arg(short, long)]
    peer_id: String,
}

/// The root function
#[tokio::main]
async fn main() {
    let args = Args::parse();
    
    // Creates a log file to handle large amounts of data
    let log_path = args.log_file_path.unwrap_or(String::from("./log/rustytorrent.log"));
    //simple_logging::log_to_file(&log_path, LevelFilter::Debug).unwrap();
    simple_logging::log_to_stderr(LevelFilter::Debug);
    
    info!("==> WELCOME TO RUSTY-TORRENT  <==");
    
    // Read the Torrent File
    let torrent = Torrent::from_torrent_file(&args.torrent_file_path).await;
    torrent.log_useful_information();
    
    // Create the files that will be written to
    let mut files = Files::new();
    files.create_files(&torrent, &args.download_path).await;
    
    // Gets peers from the given tracker
    
    let Some(socketaddrs) = torrent.get_trackers() else {
        error!("couldn't find trackers");
        panic!("couldn't find trackers")
    };
    let (remote_hostname, remote_port) = ("tracker.opentrackr.org", 1337);
    debug!("{}:{}", remote_hostname, remote_port);
    
    info!("");
    info!("-->       Finding Peers       <--");
    let listen_address = "0.0.0.0:61389".parse::<SocketAddr>().unwrap();
    let Ok(mut tracker) = Tracker::new(listen_address, std::net::SocketAddr::V4(socketaddrs[0])).await else {
        panic!("tracker couldn't be created")
    };
    info!("Successfully connected to tracker {}:{}", remote_hostname, remote_port);
    
    let peers = tracker.find_peers(&torrent, &args.peer_id).await;
    
    info!("Found Peers");
    
    let num_pieces = torrent.info.pieces.len() / 20;
    
    let mut peer = match Peer::create_connection(peers[0]).await {
        None => { return },
        Some(peer) => peer
    };
            
    peer.handshake(&torrent).await;
    peer.keep_alive_until_unchoke().await;
    info!("Successfully Created Connection with peer: {}", peer.peer_id);

    println!("{}", peers.len());
    
    let mut len = 0;
    
    for index in 0..num_pieces {
        let piece= peer.request_piece(
            index as u32, torrent.info.piece_length as u32, 
            &mut len, torrent.get_total_length() as u32
        ).await;
        
        if torrent.check_piece(&piece, index as u32) {
            files.write_piece(piece).await;
        } else {
            break
        }
    }
    
    peer.disconnect().await;

    
    info!("Successfully completed download");
}
