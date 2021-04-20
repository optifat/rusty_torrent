use rustorrent::torrent_file_parser;

fn main() {
    let filename = String::from("test_Celldweller_Cellout.torrent");
    torrent_file_parser::parse_torrent_file(filename).unwrap();
}
