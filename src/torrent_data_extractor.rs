use std::collections::HashMap;
use crate::bencode_content::Content;

#[derive(Debug)]
pub struct TorrentFilesAndHashes{
    pieces: Vec<Vec<u8>>,
    piece_length: usize,
    files: Vec<File>,
}

#[derive(Debug)]
struct File{
    path_to_file: Vec<String>,
    size: usize
}

pub fn extract_data(torrent_data: HashMap<String, Content>) -> TorrentFilesAndHashes {
    let info = torrent_data.get("info").unwrap()
                           .get_dict().unwrap();
    let files_data = info.get("files");
    let hashes = info.get("pieces").unwrap()
                     .get_bytes().unwrap();

    let mut pieces: Vec<Vec<u8>> = Vec::new();
    let piece_length: usize;
    let mut
    files: Vec<File> = Vec::new();

    if files_data.is_some(){
        let directory = info.get("name").unwrap()
                             .get_str().unwrap()
                             .clone().to_string();
        let files_data = files_data.unwrap().get_list().unwrap();
        for file in files_data.iter(){
            let mut path_to_file: Vec<String> = Vec::new();
            path_to_file.push(directory.clone());
            let path =  file.get_dict().unwrap()
                                 .get("path").unwrap()
                                 .get_list().unwrap();
            for path_elem in path{
                path_to_file.push(path_elem.get_str().unwrap().to_string());
            }
            files.push(File{
                path_to_file: path_to_file,
                size: *file.get_dict().unwrap()
                           .get("length").unwrap()
                           .get_int().unwrap() as usize,
            });
        }
    }
    else{
        files.push(File{
            path_to_file: vec![info.get("name").unwrap()
                          .get_str().unwrap()
                          .clone().to_string()],
            size: *info.get("size").unwrap()
                       .get_int().unwrap() as usize,
        });
    }

    piece_length = *info.get("piece length").unwrap()
                        .get_int().unwrap() as usize;

    let mut hash: Vec<u8> = Vec::new();
    for (index, byte) in hashes.iter().enumerate(){
        hash.push(*byte);
        if index%20 == 19{
            pieces.push(hash);
            hash = Vec::new();
        }
    }

    TorrentFilesAndHashes{pieces, piece_length, files}
}