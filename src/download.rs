use std::collections::VecDeque;
use std::net::TcpStream;
use std::sync::mpsc;
use std::thread;
use rand::Rng;

use crate::torrent_file_parser;
use crate::torrent_data_extractor;
use crate::messages;
use crate::tracker;
use crate::handshake;
use crate::bitfields;

pub fn download(filename: String){
    let (torrent_data, info_hash) = torrent_file_parser::parse_torrent_file(filename).unwrap();
    let torrent_data = torrent_data_extractor::extract_data(torrent_data);

    let mut rng = rand::thread_rng();
    let peer_id: Vec<u8> = (0..20).map(|_| rng.gen::<u8>()).collect(); // random peer id

    let (peers, interval) = tracker::request_peers(&torrent_data, &peer_id, 7878, &info_hash);
    // let mut peer_connection = handshake::perform_handshake(&peers[0], &info_hash, &peer_id, None).unwrap();

    let bitfield_expected_length = torrent_data.pieces.len()/8 + (torrent_data.pieces.len()%8 > 0) as usize;

    let mut workers = Vec::new();

    for peer in peers.iter(){
        //let (tx, rx) = mpsc::channel();
        let peer_clone = peer.clone();
        let info_hash_clone = info_hash.clone();
        let peer_id_clone = peer_id.clone();
        workers.push(thread::spawn(move || create_download_worker(peer_clone, info_hash_clone, peer_id_clone, bitfield_expected_length)));
    }

    for worker in workers{
        worker.join().unwrap();
    }

}

fn create_download_worker(peer: String, info_hash: Vec<u8>, peer_id:Vec<u8>, expected_length: usize){
    match(handshake::perform_handshake(peer, info_hash, peer_id, None)){
        Ok(mut peer_connection) => {
            bitfields::parse_bitfield(&mut peer_connection, expected_length);
        }
        Err(err) => {
            eprintln!("{:?}", err);
        }
    }

}
