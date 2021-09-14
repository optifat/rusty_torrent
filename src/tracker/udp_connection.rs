use url::Url;
use portpicker::pick_unused_port;
use std::net::UdpSocket;
use std::convert::TryInto;
use std::io;
use std::time;
use crate::torrent_file_handler::torrent_data_extractor::TorrentData;

/*
 *   Specs can be found here: https://www.bittorrent.org/beps/bep_0015.html
 */

pub fn make_udp_request(url: Url, torrent_data: &TorrentData, peer_id: &Vec<u8>, port: u16, info_hash: &Vec<u8>) -> Result<(Vec<String>, i64), io::Error>{
    let mut peers_list: Vec<String> = Vec::new();
    let mut interval = 0;

    let binding_port = pick_unused_port().unwrap();
    let binding_ip = format!("0.0.0.0:{}", port);

    let link = format!("{}:{}", url.host().unwrap(), url.port().unwrap());
    let socket = UdpSocket::bind(binding_ip).expect("couldn't bind to address");
    socket.set_read_timeout(Some(time::Duration::new(20, 0))).expect("set_read_timeout call failed");
    socket.set_write_timeout(Some(time::Duration::new(20, 0))).expect("set_write_timeout call failed");
    if let Err(err) = socket.connect(link){
        return Err(err);
    }

    let (handshake, transaction_id) = create_udp_handshake();
    socket.send(&handshake).expect("couldn't send message");
    let bytes_recieved;
    let mut connect_response: [u8; 16] = [0; 16];
    match socket.recv(&mut connect_response){
        Ok(number_of_bytes) => {
            bytes_recieved = number_of_bytes;
        }
        Err(err) => {
            return Err(err);
        }
    }

    let connection_id;
    match check_udp_response(connect_response, transaction_id){
        Ok(id) => {
            connection_id = id;
        }
        Err(err) => {
            return Err(err);
        }
    }

    let (announce_msg, transaction_id) = create_udp_announce(connection_id, torrent_data, peer_id, port, info_hash);

    if let Err(err) = socket.send(&announce_msg){
        return Err(err);
    }

    let bytes_recieved;
    let mut announce_response: [u8; 1024] = [0; 1024];
    match socket.recv(&mut announce_response){
        Ok(number_of_bytes) => {
            bytes_recieved = number_of_bytes;
        }
        Err(err) => {
            return Err(err);
        }
    }

    match parse_udp_announce_response(announce_response[0..bytes_recieved].to_vec(), transaction_id){
        Ok((peers, interval)) => {
            return Ok((peers, interval));
        }
        Err(err) => {
            return Err(err);
        }
    }
}

fn create_udp_handshake() -> (Vec<u8>, u32){
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

fn parse_udp_announce_response(response: Vec<u8>, transaction_id: u32)-> Result<(Vec<String>, i64), io::Error>{
    let mut peers_list: Vec<String> = Vec::new();
    if u32::from_be_bytes(response[0..4].try_into().unwrap()) != 1{
        return Err(io::Error::new(io::ErrorKind::Other, "Wrong action id"));
    }
    if u32::from_be_bytes(response[4..8].try_into().unwrap()) != transaction_id{
        return Err(io::Error::new(io::ErrorKind::Other, "Wrong transaction id"));
    }
    let interval = u32::from_be_bytes(response[8..12].try_into().unwrap());
    let number_of_leechers = u32::from_be_bytes(response[12..16].try_into().unwrap());
    let number_of_seeders = u32::from_be_bytes(response[16..20].try_into().unwrap());

    let mut ip = String::new();
    let mut port: u16 = 0;

    for (index, number) in response[20..].iter().enumerate(){
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
    Ok((peers_list, interval as i64))
}
