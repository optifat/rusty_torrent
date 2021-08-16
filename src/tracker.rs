use curl::easy::Easy;
use url::Url;
use std::net::{UdpSocket, ToSocketAddrs, SocketAddr};
use std::time;
use std::io;
use std::convert::TryInto;
use crate::torrent_data_extractor::TorrentData;
use crate::torrent_file_parser::parse_byte_data;

pub fn request_peers(torrent_data: &TorrentData, peer_id: &Vec<u8>, port: u16, info_hash: &Vec<u8>) -> (Vec<String>, i64){
    //let url = create_tracker_url();
    let response = make_request(torrent_data, peer_id, port, info_hash);
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

fn make_request(torrent_data: &TorrentData, peer_id: &Vec<u8>, port: u16, info_hash: &Vec<u8>) -> Vec<u8>{
    let mut announce_list = Vec::new();
    match &torrent_data.announce_list{
        Some(content) => {
            announce_list = content.to_vec();
        }
        None => {
            announce_list.push(torrent_data.announce.clone());
        }
    }
    let mut data = Vec::new();

    for tracker in announce_list{
        println!("{}", tracker);
        let url = Url::parse(&tracker).unwrap();
        let is_udp = url.scheme() == "udp";
        if is_udp{
            // specs can be found here: https://www.bittorrent.org/beps/bep_0015.html
            let link = format!("{}:{}", url.host().unwrap(), url.port().unwrap());
            let socket = UdpSocket::bind("0.0.0.0:7878").expect("couldn't bind to address");
            socket.set_read_timeout(Some(time::Duration::new(20, 0))).expect("set_read_timeout call failed");
            socket.set_write_timeout(Some(time::Duration::new(20, 0))).expect("set_write_timeout call failed");
            socket.connect(link).expect("connect function failed");

            let (handshake, transaction_id) = create_udp_handshake();
            socket.send(&handshake).expect("couldn't send message");

            let bytes_recieved;
            let mut connect_response: [u8; 16] = [0; 16];
            match socket.recv(&mut connect_response){
                Ok(number_of_bytes) => {
                    bytes_recieved = number_of_bytes;
                }
                Err(_) => {
                    continue;
                }
            }

            let connection_id;
            match check_udp_response(connect_response, transaction_id){
                Ok(id) => {
                    connection_id = id;
                }
                Err(_) => {
                    continue;
                }
            }

            let (announce_msg, transaction_id) = create_udp_announce(connection_id, torrent_data, peer_id, port, info_hash);
            socket.send(&announce_msg).expect("couldn't send message");

            let bytes_recieved;
            let mut announce_response: [u8; 1024] = [0; 1024];
            match socket.recv(&mut announce_response){
                Ok(number_of_bytes) => {
                    bytes_recieved = number_of_bytes;
                }
                Err(_) => {
                    continue;
                }
            }
            println!("{}", bytes_recieved);
            println!("{:?}", announce_response);
            data.push(0);
            break;
        }
        else{
            let url = create_tcp_tracker_url(tracker, torrent_data, peer_id, port, info_hash);

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
        }
    }
    data
}

fn create_udp_handshake() -> (Vec<u8>, u32){
    // specs can be found here: https://www.bittorrent.org/beps/bep_0015.html, "connect" part
    let mut handshake_bytes = Vec::new();
    let protocol_id: u64 = 4497486125440; // magical constant 0x41727101980
    let action: u32 = 0; // connect
    let transaction_id: u32 = 54676;

    for byte in protocol_id.to_be_bytes().iter(){
        handshake_bytes.push(*byte);
    }

    for byte in action.to_be_bytes().iter(){
        handshake_bytes.push(*byte);
    }

    for byte in transaction_id.to_be_bytes().iter(){
        handshake_bytes.push(*byte);
    }

    (handshake_bytes, transaction_id)
}

fn check_udp_response(response: [u8; 16], transaction_id: u32) -> Result<u64, io::Error>{
    if u32::from_be_bytes(response[0..4].try_into().unwrap()) != 0{
        return Err(io::Error::new(io::ErrorKind::Other, "Wrong connection id"));
    }
    if u32::from_be_bytes(response[4..8].try_into().unwrap()) != transaction_id{
        return Err(io::Error::new(io::ErrorKind::Other, "Wrong transaction id"));
    }
    Ok(u64::from_be_bytes(response[8..].try_into().unwrap()))
}

fn create_udp_announce(connection_id: u64, torrent_data: &TorrentData, peer_id: &Vec<u8>, port: u16, info_hash: &Vec<u8>) -> (Vec<u8>, u32){
    // specs can be found here: https://www.bittorrent.org/beps/bep_0015.html, "announce" part
    let mut announce_bytes = Vec::new();

    for byte in connection_id.to_be_bytes().iter(){
        announce_bytes.push(*byte);
    }

    let action: u32 = 1;
    for byte in action.to_be_bytes().iter(){
        announce_bytes.push(*byte);
    }

    let transaction_id: u32 = 542178; // some random number
    for byte in transaction_id.to_be_bytes().iter(){
        announce_bytes.push(*byte);
    }

    for byte in info_hash{
        announce_bytes.push(*byte);
    }

    for byte in peer_id{
        announce_bytes.push(*byte);
    }

    let downloaded: u64 = 0;
    for byte in downloaded.to_be_bytes().iter(){
        announce_bytes.push(*byte);
    }

    let mut left: u64 = 0;
    for file in &torrent_data.files{
        left += file.size as u64;
    }
    for byte in left.to_be_bytes().iter(){
        announce_bytes.push(*byte);
    }

    let uploaded: u64 = 0;
    for byte in uploaded.to_be_bytes().iter(){
        announce_bytes.push(*byte);
    }

    let event: u32 = 0;
    for byte in event.to_be_bytes().iter(){
        announce_bytes.push(*byte);
    }

    let ip_address: u32 = 0; // default value
    for byte in ip_address.to_be_bytes().iter(){
        announce_bytes.push(*byte);
    }

    let key: u32 = 7561816; // random number
    for byte in key.to_be_bytes().iter(){
        announce_bytes.push(*byte);
    }

    let num_want: i32 = -1; // default value
    for byte in num_want.to_be_bytes().iter(){
        announce_bytes.push(*byte);
    }

    for byte in port.to_be_bytes().iter(){
        announce_bytes.push(*byte);
    }

    (announce_bytes, transaction_id)
}

fn create_tcp_tracker_url(tracker: String, torrent_data: &TorrentData, peer_id: &Vec<u8>, port: u16, info_hash: &Vec<u8>) -> String{
    let mut url = String::new();

    url.push_str(&tracker);
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
