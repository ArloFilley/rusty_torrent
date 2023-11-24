# Lib Rusty Torrent

A Bittorrent V1 Protocol a Rust library for handling torrent files, downloading files from torrents, and communicating with BitTorrent trackers.

## Features

- **Torrent Parsing:** Parse and deserialize torrent files into a structured representation.
- **BitTorrent Tracker Communication:** Communicate with BitTorrent trackers using the UDP protocol.
- **File Management:** Manage the creation and writing of files associated with a torrent download.
- **Peer Management:** Easy management of peers through abstracted methods

## Installation

To use this library in your Rust project, add the following line to your `Cargo.toml` file:

```toml
[dependencies]
your_library_name = "0.1.0"
```

## Usage
### Parsing a Torrent File
```rust
use your_library_name::torrent::Torrent;

#[tokio::main]
async fn main() {
    let path = "path/to/your/file.torrent";
    match Torrent::from_torrent_file(path).await {
        Ok(torrent) => {
            // Process the torrent metadata
            println!("Torrent Info: {:?}", torrent);
        }
        Err(err) => {
            eprintln!("Error parsing torrent file: {}", err);
        }
    }
}
```

### Downloading Files

```rust
use your_library_name::torrent::{Torrent, Tracker};
use your_library_name::file::Files;

#[tokio::main]
async fn main() {
    // Load torrent information
    let torrent_path = "path/to/your/file.torrent";
    let torrent = Torrent::from_torrent_file(torrent_path).await.expect("Error parsing torrent file");

    // Create a tracker for finding peers
    let tracker_address = "tracker.example.com:1337".parse().expect("Error parsing tracker address");
    let mut tracker = Tracker::new(tracker_address).await.expect("Error creating tracker");

    // Find peers and start downloading
    let peer_id = "your_peer_id";
    let mut files = Files::new();
    files.create_files(&torrent, "download_directory").await;

    let peers = tracker.find_peers(&torrent, peer_id).await;

    // Implement your download logic using the found peers
    // ...

    println!("Download completed!");
}
```

## Contribution

Contributions are welcome! Feel free to open issues or submit pull requests.

## License

This project is licensed under the MIT License - see the LICENSE file for details.