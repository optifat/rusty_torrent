use std::env;

use rustorrent::torrent_file_parser;
use rustorrent::torrent_data_extractor;

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
    let torrent_data = torrent_file_parser::parse_torrent_file(filename).unwrap();

    let torrent_data = torrent_data_extractor::extract_data(torrent_data);
    println!("{:?}", torrent_data);
}
