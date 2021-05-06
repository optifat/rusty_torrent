use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::mem;
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

    let pieces_len = torrent_data.pieces.len();
    let bitfield_expected_length = pieces_len/8 + (pieces_len%8 > 0) as usize;

    let mut workers = Vec::new();
    let mut pieces_queue = VecDeque::new();
    for i in 0..pieces_len{
        pieces_queue.push_back(i);
    }
    let queue_ptr = Arc::new(Mutex::new(pieces_queue));

    for peer in peers.iter(){
        //let (tx, rx) = mpsc::channel();
        let peer_clone = peer.clone();
        let info_hash_clone = info_hash.clone();
        let peer_id_clone = peer_id.clone();
        let queue_ptr_clone = Arc::clone(&queue_ptr);
        workers.push(thread::spawn(move || create_download_worker(peer_clone, info_hash_clone, peer_id_clone, bitfield_expected_length, queue_ptr_clone)));
    }

    for worker in workers{
        worker.join().unwrap();
    }

}

fn create_download_worker(peer: String, info_hash: Vec<u8>, peer_id:Vec<u8>, expected_length: usize, queue_ptr: Arc<Mutex<VecDeque<usize>>>){
    let mut connection;
    match handshake::perform_handshake(peer, info_hash, peer_id, None){
        Ok(peer_connection) => {
            connection = peer_connection;
        }
        Err(_) => {
            return;
        }
    }

    let bitfield;
    match bitfields::parse_bitfield(&mut connection, expected_length){
        Ok(returned_bitfield) => {
            bitfield = returned_bitfield;
        }
        Err(err) => {
            println!("{:?}", err);
            return;
        }
    }
    let mut queue = queue_ptr.lock().unwrap();
    let mut index_opt = queue.pop_front();
    mem::drop(queue);
    let mut index;
    while index_opt.is_some(){
        index = index_opt.unwrap();
        if bitfield[index/8] & (1 << index%8) == 0{
            let mut queue = queue_ptr.lock().unwrap();
            queue.push_back(index);
            index_opt = queue.pop_front();
            mem::drop(queue);
        }
        else{
            break;
        }
    }
}
