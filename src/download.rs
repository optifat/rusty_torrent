use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::io::{Write, Read};
use std::mem;
use std::thread;
use std::time;

use rand::Rng;
use rand::prelude::*;
use sha1::{Sha1, Digest};

use crate::torrent_file_parser;
use crate::torrent_data_extractor;
use crate::messages;
use crate::tracker;
use crate::handshake;
use crate::bitfields;
use crate::download_status;
use crate::filewriter;

const BLOCK_SIZE: usize = 16384;

pub fn download(filename: String){
    let (torrent_data, info_hash) = torrent_file_parser::parse_torrent_file(filename).unwrap();
    let torrent_data = torrent_data_extractor::extract_data(torrent_data);

    let mut rng = rand::thread_rng();
    let peer_id: Vec<u8> = (0..20).map(|_| rng.gen::<u8>()).collect(); // random peer id

    let pieces_len = torrent_data.pieces.len();
    let bitfield_expected_length = pieces_len/8 + (pieces_len%8 > 0) as usize;

    let piece_size = torrent_data.piece_length;
    let download_status = download_status::DownloadStatus{
                              total_pieces: pieces_len as u32,
                              pieces_downloaded: 0,
                          };

    let saved_pieces_dir_name = ".test".to_string();
    filewriter::create_directory(&saved_pieces_dir_name);

    let mut pieces_queue = VecDeque::new();
    for i in 0..pieces_len{
        pieces_queue.push_back(i);
    }
    pieces_queue.make_contiguous().shuffle(&mut rand::thread_rng());
    let queue_ptr = Arc::new(Mutex::new(pieces_queue));

    let torrent_data_ptr = Arc::new(torrent_data);
    let download_status_ptr = Arc::new(Mutex::new(download_status));

    loop{
        let mut workers = Vec::new();
        let (peers, interval) = tracker::request_peers(&*torrent_data_ptr, &peer_id, 7878, &info_hash);


        let download_status = download_status_ptr.lock().unwrap();
        let mut current_progress = download_status.pieces_downloaded;
        mem::drop(download_status);

        for peer in peers.iter(){
            let queue = queue_ptr.lock().unwrap();
            let queue_len = queue.len();
            mem::drop(queue);

            let peer_clone = peer.clone();
            let info_hash_clone = info_hash.clone();
            let peer_id_clone = peer_id.clone();
            let queue_ptr_clone = Arc::clone(&queue_ptr);
            let torrent_data_ptr_clone = Arc::clone(&torrent_data_ptr);
            let download_status_ptr_clone = Arc::clone(&download_status_ptr);
            let saved_pieces_dir_name_clone = saved_pieces_dir_name.clone();

            if queue_len == 0{
                filewriter::compose_files(&*torrent_data_ptr, saved_pieces_dir_name.to_string().clone()).unwrap();
                //filewriter::remove_directory(&saved_pieces_dir_name.to_string());
                println!("Success!");
                for byte in info_hash{
                    print!("{} ", byte);
                }
                return;
            }
            else if queue_len < 10{
                create_download_worker(peer_clone,
                                       info_hash_clone,
                                       peer_id_clone,
                                       piece_size,
                                       bitfield_expected_length,
                                       queue_ptr_clone,
                                       torrent_data_ptr_clone,
                                       download_status_ptr_clone,
                                       saved_pieces_dir_name_clone
                                    );
            }
            else{
                workers.push(thread::spawn(move || create_download_worker(peer_clone,
                                                                          info_hash_clone,
                                                                          peer_id_clone,
                                                                          piece_size,
                                                                          bitfield_expected_length,
                                                                          queue_ptr_clone,
                                                                          torrent_data_ptr_clone,
                                                                          download_status_ptr_clone,
                                                                          saved_pieces_dir_name_clone
                                                                      )));
            }
        }

        if workers.len() > 0{
            for worker in workers{
                worker.join().unwrap();
            }
        }

        let download_status = download_status_ptr.lock().unwrap();

        if download_status.pieces_downloaded  == current_progress{
            println!("Seems like there is no available leechers/seeders. Aborting.");
            return;
        }

        let mut current_progress = download_status.pieces_downloaded;
        mem::drop(download_status);
    }
}

fn create_download_worker(peer: String,
                          info_hash: Vec<u8>,
                          peer_id:Vec<u8>,
                          piece_size: usize,
                          expected_length: usize,
                          queue_ptr: Arc<Mutex<VecDeque<usize>>>,
                          torrent_data_ptr: Arc<torrent_data_extractor::TorrentData>,
                          download_status_ptr: Arc<Mutex<download_status::DownloadStatus>>,
                          saved_pieces_dir_name: String
                         )
{
    let mut connection;
    if let Ok(peer_connection) = handshake::perform_handshake(peer, info_hash, peer_id, None){
        connection = peer_connection;
    }
    else {
        return;
    }

    connection.set_read_timeout(Some(time::Duration::new(20, 0))).expect("set_read_timeout call failed");
    connection.set_write_timeout(Some(time::Duration::new(10, 0))).expect("set_write_timeout call failed");

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

    if let Err(_) = connection.write(&messages::create_unchoke_msg()){
        return;
    }

    if let Err(_) = connection.write(&messages::create_interested_msg()){
        return;
    }

    let mut queue = queue_ptr.lock().unwrap();
    let mut index_opt = queue.pop_front();
    mem::drop(queue);

    let mut index;
    let mut fails = 0;
    let mut buffer_overlow = false;
    let mut times_choked: u8 = 0;

    while index_opt.is_some(){
        index = index_opt.unwrap();

        if bitfield[index/8] & (1 << 7-index%8) == 0{
            // the case when peer doesn't have this index piece
            let mut queue = queue_ptr.lock().unwrap();
            queue.push_back(index);
            index_opt = queue.pop_front();
            mem::drop(queue);
            fails += 1;
            if fails == 5{
                let mut queue = queue_ptr.lock().unwrap();
                queue.push_back(index);
                mem::drop(queue);
                return;
            }
        }
        else{
            // downloading piece
            let mut piece = Vec::with_capacity(piece_size);
            let number_of_blocks: u32 = (piece_size/BLOCK_SIZE) as u32 + (piece_size%BLOCK_SIZE != 0) as u32;
            let mut piece_msg: [u8; BLOCK_SIZE+18] = [0; BLOCK_SIZE+18];

            for i in 0..number_of_blocks{
                if let Err(_) = connection.write(&messages::create_request_msg(index as u32, i*(BLOCK_SIZE as u32), BLOCK_SIZE as u32)){
                    let mut queue = queue_ptr.lock().unwrap();
                    queue.push_back(index);
                    mem::drop(queue);
                    return;
                }

                let mut block: Vec<u8> = Vec::with_capacity(BLOCK_SIZE);
                let mut bytes_got = 0;
                let mut current_message: Vec<u8> = Vec::with_capacity(BLOCK_SIZE+18);

                let mut counter: u8 = 0;

                loop{
                    counter += 1;
                    if counter == 21{ // too slow download
                        let mut queue = queue_ptr.lock().unwrap();
                        queue.push_back(index);
                        mem::drop(queue);
                        return;
                    }

                    let bytes_got_this_iter;

                    if let Ok(number_of_bytes) = connection.read(&mut piece_msg){
                        bytes_got_this_iter = number_of_bytes;
                    }
                    else{
                        let mut queue = queue_ptr.lock().unwrap();
                        queue.push_back(index);
                        mem::drop(queue);
                        return;
                    }


                    bytes_got += bytes_got_this_iter;

                    for i in 0..bytes_got_this_iter{
                        current_message.push(piece_msg[i])
                    }

                    if bytes_got == 5 || bytes_got == BLOCK_SIZE+13{
                        let choked;
                        let result;

                        if let Ok((choked_result, bytes)) = messages::read_message(current_message[0..bytes_got].to_vec()){
                            choked = choked_result;
                            result = bytes;
                        }
                        else{
                            let mut queue = queue_ptr.lock().unwrap();
                            queue.push_back(index);
                            mem::drop(queue);
                            return;
                        }

                        if choked{
                            times_choked += 1;
                            if times_choked == 4{
                                let mut queue = queue_ptr.lock().unwrap();
                                queue.push_back(index);
                                mem::drop(queue);
                                return;
                            }
                        }

                        if let Some(bytes) = result{
                            for byte in bytes{
                                block.push(byte);
                            }
                        }
                        break;
                    }
                    else if bytes_got == BLOCK_SIZE+18{
                        let choked;

                        if let Ok((choked_result, _)) = messages::read_message(current_message[0..5].to_vec()){
                            choked = choked_result;
                        }
                        else {
                            let mut queue = queue_ptr.lock().unwrap();
                            queue.push_back(index);
                            mem::drop(queue);
                            return;
                        }

                        if choked{
                            times_choked += 1;
                            if times_choked == 4{
                                let mut queue = queue_ptr.lock().unwrap();
                                queue.push_back(index);
                                mem::drop(queue);
                                return;
                            }
                        }

                        let choked;
                        let result;

                        if let Ok((choked_result, bytes)) = messages::read_message(current_message[5..].to_vec()){
                            choked = choked_result;
                            result = bytes;
                        }
                        else {
                            let mut queue = queue_ptr.lock().unwrap();
                            queue.push_back(index);
                            mem::drop(queue);
                            return;
                        }

                        if choked{
                            times_choked += 1;
                            if times_choked == 4{
                                let mut queue = queue_ptr.lock().unwrap();
                                queue.push_back(index);
                                mem::drop(queue);
                                return;
                            }
                        }

                        if let Some(bytes) = result{
                            for byte in bytes{
                                block.push(byte);
                            }
                        }
                        break;
                    }
                    else if bytes_got > BLOCK_SIZE+18{
                        buffer_overlow = true;
                        break;
                    }
                }

                if buffer_overlow{
                    break;
                }

                for byte in block.iter(){
                    piece.push(*byte);
                }
            }


            if !check_piece(&piece, &(*torrent_data_ptr).pieces[index]) || buffer_overlow {
                fails += 1;
                let mut queue = queue_ptr.lock().unwrap();
                queue.push_back(index);
                mem::drop(queue);
                if fails == 5{
                    return;
                }
            }
            else{
                filewriter::save_piece(saved_pieces_dir_name.clone(), piece.clone(), index).unwrap();
                let mut download_status = download_status_ptr.lock().unwrap();
                download_status.pieces_downloaded += 1;
                let progress = 100*download_status.pieces_downloaded/download_status.total_pieces;
                println!("[{}/{}, {}%] Piece {} downloaded", download_status.pieces_downloaded, download_status.total_pieces, progress, index);
                mem::drop(download_status);
            }

            buffer_overlow = false;
            let mut queue = queue_ptr.lock().unwrap();
            index_opt = queue.pop_front();
            mem::drop(queue);
        }
    }
}

fn check_piece(piece: &Vec<u8>, expected_hash: &Vec<u8>) -> bool {
    let mut hasher = Sha1::new();
    hasher.update(&piece);
    let piece_hash = hasher.finalize();

    for i in 0..20{
        if piece_hash[i] != expected_hash[i]{
            return false;
        }
    }

    true
}
