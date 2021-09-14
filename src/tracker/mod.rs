use url::Url;
use futures::executor::block_on;
use futures::future::join_all;
use crate::torrent_file_handler::torrent_data_extractor::TorrentData;
use crate::torrent_file_handler::torrent_file_parser::parse_byte_data;

mod tcp_connection;
mod udp_connection;

pub fn request_peers(torrent_data: &TorrentData, peer_id: &Vec<u8>, port: u16, info_hash: &Vec<u8>) -> (Vec<String>, i64){
    let mut announce_list = Vec::new();

    match &torrent_data.announce_list{
        Some(content) => {
            announce_list = content.to_vec();
        }
        None => {
            announce_list.push(torrent_data.announce.clone());
        }
    }

    make_requests(announce_list, (*torrent_data).clone(), (*peer_id).to_vec(), port, (*info_hash).to_vec())
}

#[tokio::main]
async fn make_requests(announce_list: Vec<String>, torrent_data: TorrentData, peer_id: Vec<u8>, port: u16, info_hash: Vec<u8>) -> (Vec<String>, i64){
    let mut peers = Vec::new();
    let mut interval = 0;
    let mut futures = Vec::new();

    for tracker in announce_list{
        let torrent_data_clone = torrent_data.clone();
        let peer_id_clone = peer_id.clone();
        let info_hash_clone = info_hash.clone();

        futures.push(tokio::spawn(async move {
            make_request(tracker, torrent_data_clone, peer_id_clone, port, info_hash_clone)
        }));
    }

    let result = join_all(futures).await;

    for elem in result{
        if let Ok(output) = elem{
            if let Some(data) = output{
                let (peer_result, interval_result) = data;
                for peer in peer_result{
                    peers.push(peer);
                }
                interval = interval_result;
            }
        }
    }
    (peers, interval)
}

fn make_request(tracker: String, torrent_data: TorrentData, peer_id: Vec<u8>, port: u16, info_hash: Vec<u8>) -> Option<(Vec<String>, i64)>{
    println!("{}", tracker);
    let url = Url::parse(&tracker).unwrap();
    let is_udp = url.scheme() == "udp";
    if is_udp{
        match udp_connection::make_udp_request(url, &torrent_data, &peer_id, port, &info_hash){
            Ok(result) => {
                return Some(result);
            }
            Err(err) => {
                println!("{}", err);
            }
        }
    }
    else{
        match tcp_connection::make_tcp_request(tracker, &torrent_data, &peer_id, port, &info_hash){
            Ok(result) => {
                return Some(result);
            }
            Err(err) => {
                println!("{}", err);
            }
        }
    }
    None
}
