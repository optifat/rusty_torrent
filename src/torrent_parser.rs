use std::fs::read;
use std::io;
use std::collections::HashMap;

pub fn parse_torrent_file(filename: String) -> Result<HashMap::<String, String>, io::Error>{
    let mut torrent_contents = HashMap::new();
    let binary_contents = read(filename)?;
    let string_contents = String::from_utf8_lossy(&binary_contents);

    let mut reading_key = true;
    let mut key = String::new();
    let mut value = String::new();
    let mut value_length: u64 = 0;
    let mut hash = Vec::<u8>::new();

    for i in 0..string_contents.len(){
        if reading_key{

        }
    }
    Ok(torrent_contents)
}
