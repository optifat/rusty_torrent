use std::env;
use rustorrent::torrent_file_parser;
use rustorrent::torrent_data_extractor;
use rustorrent::tracker;

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

    let peer_id = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20]; // random peer id now

    let (peers, interval) = tracker::request_peers(&torrent_data, &peer_id, 7878, &info_hash);
    println!("{:?}", peers);
}
