# Rusty Torrent

Welcome to the Rusty torrent Workspace, a Rust project that provides a BitTorrent client and a BitTorrent library for handling torrent files, downloading files from torrents, and communicating with BitTorrent trackers. 

>[!WARNING]
>This project was created as a way for me to learn about the BitTorrent protocol. While it offers functionality for handling torrent files, downloading files from torrents, and communicating with BitTorrent trackers, please note that it may not be production-ready. Use it at your own discretion and consider its experimental nature.

# Overview
## Links

- [Lib Rusty Torrent Readme](./lib_rusty_torrent/README.md)
- [Rusty Torrenter Readme](./rusty_torrenter/README.md)

## Lib Rusty Torrent

The BitTorrent Library is a Rust library for interacting with BitTorrent files, trackers, and downloading files. It provides a set of APIs for parsing torrent files, communicating with trackers, and managing file downloads.

### Features

- **Torrent Parsing:** Parse and deserialize torrent files into a structured representation.
- **BitTorrent Tracker Communication:** Communicate with BitTorrent trackers using the UDP protocol.
- **File Management:** Manage the creation and writing of files associated with a torrent download.

## Rusty Torrenter

The BitTorrent Client is an example application built using the BitTorrent Library. It demonstrates how to use the library to download files from torrents, find peers, and manage the download process.

### Features:

- **Torrent Download:** Download files from torrents using the BitTorrent Library.
- **Tracker Communication:** Communicate with BitTorrent trackers to discover and connect with peers.
- **File Management:** Create and manage files associated with the torrent download.
