use rustorrent::torrent_file_parser;

fn main() {
    let filename = String::from("tears-of-steel.torrent");
    let torrent_data = torrent_file_parser::parse_torrent_file(filename).unwrap();

    for (key, val) in torrent_data.iter(){
        println!("{:?}", key);
    }
}
