use super::bencode_content::Content;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct TorrentData {
    pub pieces: Vec<Vec<u8>>,
    pub piece_length: usize,
    pub files: Vec<File>,
    pub announce: String,
    pub announce_list: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct File {
    pub path_to_file: Vec<String>,
    pub size: usize,
}

pub fn extract_data(torrent_data: HashMap<String, Content>) -> anyhow::Result<TorrentData> {
    let info = torrent_data
        .get("info")
        .ok_or(anyhow::anyhow!("No 'info' field in torrent_data"))?
        .get_dict()
        .ok_or(anyhow::anyhow!("Couldn't get dictionary"))?;

    let files_data = info.get("files");

    let hashes = info
        .get("pieces")
        .ok_or(anyhow::anyhow!("No 'pieces' field in hashes data"))?
        .get_bytes()
        .ok_or(anyhow::anyhow!("Couldn't get bytes"))?;

    let mut pieces: Vec<Vec<u8>> = Vec::new();
    let piece_length: usize;
    let mut files: Vec<File> = Vec::new();

    if files_data.is_some() {
        let directory = info
            .get("name")
            .ok_or(anyhow::anyhow!(
                "No 'name' field for directory data in torrent file"
            ))?
            .get_str()
            .ok_or(anyhow::anyhow!("Couldn't get str"))?
            .clone()
            .to_string();
        let files_data = files_data
            .unwrap()
            .get_list()
            .ok_or(anyhow::anyhow!("Couldn't get list"))?;
        for file in files_data.iter() {
            let mut path_to_file: Vec<String> = Vec::new();
            path_to_file.push(directory.clone());
            let path = file
                .get_dict()
                .ok_or(anyhow::anyhow!("Couldn't get dictionary"))?
                .get("path")
                .ok_or(anyhow::anyhow!("Couldn't define path in torrent file"))?
                .get_list()
                .ok_or(anyhow::anyhow!("Couldn't get list"))?;

            for path_elem in path {
                path_to_file.push(
                    path_elem
                        .get_str()
                        .ok_or(anyhow::anyhow!("Couldn't get str"))?
                        .to_string(),
                );
            }
            files.push(File {
                path_to_file: path_to_file,
                size: *file
                    .get_dict()
                    .ok_or(anyhow::anyhow!("Couldn't get dictionary"))?
                    .get("length")
                    .ok_or(anyhow::anyhow!("No'length' field"))?
                    .get_int()
                    .ok_or(anyhow::anyhow!("Couldn't get int"))? as usize,
            });
        }
    } else {
        files.push(File {
            path_to_file: vec![info
                .get("name")
                .ok_or(anyhow::anyhow!("No 'name' field"))?
                .get_str()
                .ok_or(anyhow::anyhow!("Couldn't get str"))?
                .clone()
                .to_string()],
            size: *info
                .get("length")
                .ok_or(anyhow::anyhow!("No 'length' field"))?
                .get_int()
                .ok_or(anyhow::anyhow!("Couldn't get int"))? as usize,
        });
    }

    piece_length = *info
        .get("piece length")
        .ok_or(anyhow::anyhow!("No 'piece length' field"))?
        .get_int()
        .ok_or(anyhow::anyhow!("Couldn't get list"))? as usize;

    let mut hash: Vec<u8> = Vec::new();
    for (index, byte) in hashes.iter().enumerate() {
        hash.push(*byte);
        if index % 20 == 19 {
            pieces.push(hash);
            hash = Vec::new();
        }
    }

    let announce = torrent_data
        .get("announce")
        .ok_or(anyhow::anyhow!("No 'announce' field"))?
        .get_str()
        .ok_or(anyhow::anyhow!("Couldn't get str"))?
        .to_string();

    let mut announce_list_vec = Vec::new();
    let announce_list;
    match torrent_data.get("announce-list") {
        Some(content) => {
            for elem in content
                .get_list()
                .ok_or(anyhow::anyhow!("Couldn't get list"))?
            {
                announce_list_vec.push(
                    (*elem
                        .get_list()
                        .ok_or(anyhow::anyhow!("Couldn't get list"))?[0]
                        .get_str()
                        .ok_or(anyhow::anyhow!("Couldn't get str"))?)
                    .clone(),
                );
            }
            announce_list = Some(announce_list_vec);
        }
        None => {
            announce_list = None;
        }
    }

    Ok(TorrentData {
        pieces,
        piece_length,
        files,
        announce,
        announce_list,
    })
}
