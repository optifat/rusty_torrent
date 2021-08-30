pub mod torrent_file_handler;
pub mod tracker;
pub mod p2p;
pub mod download;
pub mod filewriter;

use std::env;

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

    download::download(filename);
}
