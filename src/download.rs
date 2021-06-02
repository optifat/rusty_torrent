use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::io::{Write, Read};
use std::mem;
use std::thread;
use std::time;
use rand::Rng;
use sha1::{Sha1, Digest};

use crate::torrent_file_parser;
use crate::torrent_data_extractor;
use crate::messages;
use crate::tracker;
use crate::handshake;
use crate::bitfields;

const BLOCK_SIZE: usize = 16384;

pub fn download(filename: String){
    let (torrent_data, info_hash) = torrent_file_parser::parse_torrent_file(filename).unwrap();
    let torrent_data = torrent_data_extractor::extract_data(torrent_data);

    let mut rng = rand::thread_rng();
    let peer_id: Vec<u8> = (0..20).map(|_| rng.gen::<u8>()).collect(); // random peer id

    let (peers, interval) = tracker::request_peers(&torrent_data, &peer_id, 7878, &info_hash);
    // let mut peer_connection = handshake::perform_handshake(&peers[0], &info_hash, &peer_id, None).unwrap();

    let pieces_len = torrent_data.pieces.len();
    let bitfield_expected_length = pieces_len/8 + (pieces_len%8 > 0) as usize;

    let piece_size = torrent_data.piece_length;

    let mut workers = Vec::new();
    let mut pieces_queue = VecDeque::new();
    for i in 0..pieces_len{
        pieces_queue.push_back(i);
    }
    let queue_ptr = Arc::new(Mutex::new(pieces_queue));
    let torrent_data_ptr = Arc::new(torrent_data);

    for peer in peers.iter(){
        //let (tx, rx) = mpsc::channel();
        let peer_clone = peer.clone();
        let info_hash_clone = info_hash.clone();
        let peer_id_clone = peer_id.clone();
        let queue_ptr_clone = Arc::clone(&queue_ptr);
        let torrent_data_ptr_clone = Arc::clone(&torrent_data_ptr);
        workers.push(thread::spawn(move || create_download_worker(peer_clone, info_hash_clone, peer_id_clone, piece_size, bitfield_expected_length, queue_ptr_clone, torrent_data_ptr_clone)));
    }

    for worker in workers{
        worker.join().unwrap();
    }

}

fn create_download_worker(peer: String,
                          info_hash: Vec<u8>,
                          peer_id:Vec<u8>,
                          piece_size: usize,
                          expected_length: usize,
                          queue_ptr: Arc<Mutex<VecDeque<usize>>>,
                          torrent_data_ptr: Arc<torrent_data_extractor::TorrentData>)
                         {
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

    connection.write(&messages::create_unchoke_msg()).unwrap();
    connection.write(&messages::create_interested_msg()).unwrap();

    let mut queue = queue_ptr.lock().unwrap();
    let mut index_opt = queue.pop_front();
    mem::drop(queue);
    let mut index;
    while index_opt.is_some(){
        index = index_opt.unwrap();
        if bitfield[index/8] & (1 << index%8) == 0{
            // peer doesn't have index-piece
            let mut queue = queue_ptr.lock().unwrap();
            queue.push_back(index);
            index_opt = queue.pop_front();
            mem::drop(queue);
        }
        else{
            // downloading piece
            let mut piece = Vec::new();
            let number_of_blocks: u32 = (piece_size/BLOCK_SIZE) as u32 + (piece_size%BLOCK_SIZE != 0) as u32;
            let mut piece_msg: [u8; BLOCK_SIZE+18] = [0; BLOCK_SIZE+18];
            let mut bytes_got = 0;
            for i in 0..number_of_blocks{
                connection.write(&messages::create_request_msg(index as u32, i*(BLOCK_SIZE as u32), BLOCK_SIZE as u32)).unwrap();
                thread::sleep(time::Duration::from_millis(1000));
                bytes_got = connection.read(&mut piece_msg).unwrap();
                // println!("{:?}", piece_msg.to_vec());
                // println!("Bytes got {:?}", bytes_got);
                let mut block: Vec<u8> = Vec::new();
                if bytes_got == BLOCK_SIZE+18{
                    match messages::parse_piece_msg(piece_msg[5..].to_vec()){
                        Ok(result) => {
                            block = result;
                        }
                        Err(err) => {
                            let mut queue = queue_ptr.lock().unwrap();
                            queue.push_back(index);
                            mem::drop(queue);
                            //println!("{:?}", err);
                            break;
                        }
                    }
                }
                else if bytes_got == BLOCK_SIZE+13{
                    match messages::parse_piece_msg(piece_msg[0..BLOCK_SIZE+13].to_vec()){
                        Ok(result) => {
                            block = result;
                        }
                        Err(err) => {
                            let mut queue = queue_ptr.lock().unwrap();
                            queue.push_back(index);
                            mem::drop(queue);
                            //println!("{:?}", err);
                            break;
                        }
                    }
                }
                else{
                    let mut queue = queue_ptr.lock().unwrap();
                    queue.push_back(index);
                    index_opt = queue.pop_front();
                    mem::drop(queue);
                    break;
                }


                for byte in block.iter(){
                    piece.push(*byte);
                }
            }

            if !check_piece(&piece, &(*torrent_data_ptr).pieces[index]){
                //println!("Failed piece with index {:?} integrity check", index);
                let mut queue = queue_ptr.lock().unwrap();
                queue.push_back(index);
                index_opt = queue.pop_front();
                mem::drop(queue);
                continue;
            }
            else{
                println!("Piece {} downloaded", index);
                //println!("Index: {:?},  piece: {:?}", index, piece);
            }
            //println!("Index: {:?},  piece: {:?}", index, piece);
            let mut queue = queue_ptr.lock().unwrap();
            index_opt = queue.pop_front();
            mem::drop(queue);
        }
    }
}

fn check_piece(piece: &Vec<u8>, expected_hash: &Vec<u8>) -> bool {
    let mut hasher = Sha1::new();
    hasher.update(&piece);
    let piece_hash = hasher.finalize().to_vec();

    // println!("piece: {:?}, expected: {:?}", piece_hash, expected_hash);
    //println!("{:?}", piece);

    for i in 0..20{
        if piece_hash[i] != expected_hash[i]{
            return false;
        }
    }
    true
}
