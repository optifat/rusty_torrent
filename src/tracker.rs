use curl::easy::Easy;
use crate::torrent_data_extractor::TorrentData;
use crate::torrent_file_parser::parse_byte_data;

pub fn request_peers(torrent_data: &TorrentData, peer_id: &Vec<u8>, port: u16, info_hash: &Vec<u8>) -> (Vec<String>, i64){
    let url = create_tracker_url(torrent_data, peer_id, port, info_hash);
    let response = make_request(url);
    let response_data = parse_byte_data(&response).unwrap();
    if response_data.get("failure reason").is_some(){
        panic!("Announce failure response: {:?}", response_data.get("failure reason").unwrap());
    }

    let peers = response_data.get("peers").unwrap().get_bytes().unwrap();
    if peers.len()%6 != 0{
        panic!("Corrupted peers data");
    }
    let mut ip = String::new();
    let mut port: u16 = 0;
    let mut peers_list: Vec<String> = Vec::new();

    for (index, number) in peers.iter().enumerate(){
        match index%6{
            0 => {
                ip = String::new();
                port = 0;
                ip.push_str(&*number.to_string());
                ip.push('.');
            }
            3 => {
                ip.push_str(&*number.to_string());
                ip.push(':');
            }
            4 => {
                port += (*number as u16)*256;
            }
            5 => {
                port += *number as u16;
                ip.push_str(&port.to_string());
                peers_list.push(ip.clone());
            }
            _ => {
                ip.push_str(&*number.to_string());
                ip.push('.');
            }
        }
    }

    (peers_list, *response_data.get("interval").unwrap().get_int().unwrap())
}

fn make_request(url: String) -> Vec<u8>{
    let mut data = Vec::new();
    let mut tracker = Easy::new();
    tracker.url(&url).unwrap();
    {
        let mut transfer = tracker.transfer();
        transfer.write_function(|new_data| {
            data.extend_from_slice(new_data);
            Ok(new_data.len())
        }).unwrap();
        transfer.perform().unwrap();
    }
    data
}

fn create_tracker_url(torrent_data: &TorrentData, peer_id: &Vec<u8>, port: u16, info_hash: &Vec<u8>) -> String{
    let mut url = String::new();

    url.push_str(&torrent_data.announce);
    url.push_str("?compact=1&downloaded=0&info_hash=");

    url.push_str(&bytes_to_url(info_hash));

    url.push_str("&left=");
    let mut length: usize = 0;
    for file in &torrent_data.files{
        length += file.size;
    }
    url.push_str(&length.to_string());

    url.push_str("&peer_id=");
    url.push_str(&bytes_to_url(peer_id));

    url.push_str("&port=");
    url.push_str(&(port as i32).to_string());

    url.push_str("&uploaded=0");

    url
}

fn bytes_to_url(bytes: &Vec<u8>) -> String{
    let mut url = String::new();
    for number in bytes{
        url.push('%');
        url.push_str(&u8_decimal_to_hex(number));
    }
    url
}

fn u8_decimal_to_hex(decimal: &u8) -> String{
    let mut hex = String::new();
    let first_digit = decimal/16;
    let second_digit = decimal%16;
    if first_digit > 9{
        hex.push(('A' as u8 + first_digit - 10) as char);
    }
    else{
        hex.push(('0' as u8 + first_digit) as char);
    }
    if second_digit > 9{
        hex.push(('A' as u8 + second_digit - 10) as char);
    }
    else{
        hex.push(('0' as u8 + second_digit) as char);
    }
    hex
}
