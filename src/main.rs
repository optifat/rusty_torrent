#![deny(warnings)]

mod download;
mod filewriter;
mod p2p;
mod torrent_file_handler;
mod tracker;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        println!("Please provide a torrent file name");
        return;
    } else if args.len() > 2 {
        println!("Too many arguments: please provide only a torrent file name");
        return;
    }
    let filename = (&args[1]).to_string();

    match download::download(filename) {
        Ok(()) => println!("Download finished successfully"),
        Err(err) => println!("{:?}", err),
    }
}
