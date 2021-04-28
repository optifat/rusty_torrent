use std::env;
use std::net::TcpStream;
use rand::Rng;
use rustorrent::torrent_file_parser;
use rustorrent::torrent_data_extractor;
use rustorrent::tracker;
use rustorrent::handshake;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1{
        println!("Please provide a torrent file name");
        return;
    }
    else if args.len() > 2{
        println!("Too many arguments: please provide only a torrent file name");
        return;
    }
    let filename = (&args[1]).to_string();
    let (torrent_data, info_hash) = torrent_file_parser::parse_torrent_file(filename).unwrap();
    let torrent_data = torrent_data_extractor::extract_data(torrent_data);

    let mut rng = rand::thread_rng();
    let peer_id: Vec<u8> = (0..20).map(|_| rng.gen::<u8>()).collect(); // random peer id

    let (peers, interval) = tracker::request_peers(&torrent_data, &peer_id, 7878, &info_hash);
    let peer_connection = handshake::perform_handshake(&peers[0], &info_hash, &peer_id, None);
    //println!("{:?}", peers);

}
