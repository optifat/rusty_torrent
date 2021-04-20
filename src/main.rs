use rustorrent::torrent_parser;

fn main() {
    let filename = String::from("test_Celldweller_Cellout.torrent");
    torrent_parser::parse_torrent_file(filename).unwrap();
}
