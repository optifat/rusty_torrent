use std::io;
use std::fs;
use std::io::prelude::*;
use std::os::unix::prelude::FileExt;

use crate::torrent_file_handler::torrent_data_extractor;

pub fn create_directory(path: &String) -> io::Result<()>{
    fs::create_dir_all(path)?;
    Ok(())
}

pub fn remove_directory(path: &String) -> io::Result<()>{
    fs::remove_dir(path)?;
    Ok(())
}

pub fn save_piece(path: String, piece: Vec<u8>, index: usize) -> io::Result<()>{
    let mut filename = path;
    filename.push_str("/.");
    filename.push_str(&index.to_string());

    let mut file = fs::File::create(filename)?;
    let bytes_written = file.write(&piece)?;

    if bytes_written != piece.len(){
        return Err(io::Error::new(io::ErrorKind::Other, "Message announced length is too small"));
    }
    Ok(())
}

pub fn compose_files(torrent_data: &torrent_data_extractor::TorrentData, saved_pieces_dir_name: String) -> io::Result<()>{
    let piece_size = torrent_data.piece_length;
    let mut current_piece = 0;

    let mut bytes_from_prev_piece: Vec<u8> = Vec::new();

    for file in torrent_data.files.iter(){
        let mut path = file.path_to_file.clone();
        let mut filename: String;
        if path.len() == 1{
            filename = path[0].clone();
        }
        else{
            filename = path.pop().unwrap();
            let mut dirs: String = String::new();

            for dir in path{
                dirs.push_str(&dir);
                dirs.push('/');
            }

            create_directory(&dirs);
            dirs.push_str(&filename);
            filename = dirs;
        }

        println!("{}", filename);

        let mut bytes_written_into_file = 0;

        let f = fs::File::create(filename)?;

        if bytes_from_prev_piece.len() != 0{
            if file.size > bytes_from_prev_piece.len(){
                f.write_at(&bytes_from_prev_piece, bytes_written_into_file as u64);
                bytes_written_into_file += bytes_from_prev_piece.len();
                bytes_from_prev_piece = Vec::new();
            }
            else{
                f.write_at(&bytes_from_prev_piece[0..file.size], bytes_written_into_file as u64);
                bytes_written_into_file += bytes_from_prev_piece.len();
                bytes_from_prev_piece = bytes_from_prev_piece[file.size..].to_vec();
                continue;
            }
        }

        while file.size - bytes_written_into_file > piece_size{
            if bytes_from_prev_piece.len() != 0{
                f.write_at(&bytes_from_prev_piece, bytes_written_into_file as u64);
                bytes_written_into_file += bytes_from_prev_piece.len();
                bytes_from_prev_piece = Vec::new();
                continue;
            }
            let mut current_piece_filename = saved_pieces_dir_name.clone();
            current_piece_filename.push_str("/.");
            current_piece_filename.push_str(&current_piece.to_string());
            let current_piece_bytes = fs::read(current_piece_filename.clone())?;
            let total_bytes = current_piece_bytes.len();
            f.write_at(&current_piece_bytes, bytes_written_into_file as u64);
            bytes_written_into_file += total_bytes;
            //fs::remove_file(current_piece_filename)?;
            current_piece += 1;
        }

        let mut current_piece_filename = saved_pieces_dir_name.clone();
        current_piece_filename.push_str("/.");
        current_piece_filename.push_str(&current_piece.to_string());
        let current_piece_bytes = fs::read(current_piece_filename.clone())?;
        let total_bytes = current_piece_bytes.len();
        f.write_at(&current_piece_bytes[0..file.size - bytes_written_into_file], bytes_written_into_file as u64);
        for byte in current_piece_bytes[file.size - bytes_written_into_file..].iter(){
            bytes_from_prev_piece.push(*byte);
        }
        //fs::remove_file(current_piece_filename)?;
        current_piece += 1;
    }
    Ok(())
}
