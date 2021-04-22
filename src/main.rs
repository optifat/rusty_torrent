use std::env;

use rustorrent::torrent_file_parser;

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

    for (key, val) in torrent_data.iter(){
        println!("{:?}, {:?}", key, val);
    }
}
