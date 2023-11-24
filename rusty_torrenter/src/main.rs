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

use core::panic;
use std::net::SocketAddr;

// Crate Imports
use lib_rusty_torrent::{
    files::Files,
    peer::*,
    torrent::Torrent,
    tracker::Tracker,
};

// External Ipmorts
use clap::Parser;
use log::{ debug, info, LevelFilter, error };

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
    let torrent = Torrent::from_torrent_file(&args.torrent_file_path).await.unwrap();
    
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
    
    

    println!("{}", peers.len());
    
    let mut len = 0;
    let mut i = 0;
    let t = torrent.clone();



    let _ = sender.send(peer::ControlMessage::DownloadPiece(i, t.info.piece_length as u32, len, t.get_total_length() as u32));

    let a = reciever.recv().await.unwrap();
        
    println!("2 {a:?}");

    let peer::ControlMessage::DownloadedPiece(b) = a else {
        continue;
    };

    if t.check_piece(&b, i) {
        files.write_piece(b).await;
    } else {
        break
    }
    
    peer.disconnect().await;

    
    info!("Successfully completed download");
}
