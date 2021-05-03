use std::net::{TcpStream, SocketAddr};
use std::time::Duration;
use std::io;
use std::io::prelude::*;

const BUF_SIZE: usize = 128;

pub fn perform_handshake(peer_ip: &String, info_hash: &Vec<u8>, peer_id: &Vec<u8>, pstr_option: Option<String>) -> Result<TcpStream, io::Error>{
    println!("Performing handshake with {:?}", peer_ip);
    let mut response = Vec::<u8>::new();
    match TcpStream::connect_timeout(&peer_ip.parse::<SocketAddr>().unwrap(), Duration::new(3, 0)) {
        Ok(mut stream) => {
            println!("Connected");
            stream.write(&create_handshake_msg(info_hash, peer_id, pstr_option)).unwrap(); // my panic code: 104, kind: ConnectionReset, message: "Connection reset by peer"
            let mut size = BUF_SIZE;
            let mut buf: [u8; BUF_SIZE];

            while size == BUF_SIZE{
                buf = [0; BUF_SIZE];
                size = stream.read(&mut buf).unwrap();
                for i in 0..size{
                    response.push(buf[i]);
                }
            }
            for i in 0..20{
                if response[i + (response[0] as usize) + 9] != info_hash[i] {
                    eprintln!("Hash infos don't match");
                    return Err(io::Error::new(io::ErrorKind::Other, "Hash infos don't match"));
                }
            }
            println!("{:?}", response);
            Ok(stream)
        },
        Err(_) => {
            eprintln!("Failed to connect");
            Err(io::Error::new(io::ErrorKind::Other, "Failed to connect"))
        }
    }
}

fn create_handshake_msg(info_hash: &Vec<u8>, peer_id: &Vec<u8>, pstr_option: Option<String>) -> Vec<u8>{
    let mut msg: Vec<u8> = Vec::new();
    let default_pstr = "BitTorrent protocol".to_string();
    let pstr = match &pstr_option{
        Some(string) => string.as_bytes(),
        None => default_pstr.as_bytes()
    };
    msg.push(pstr.len() as u8);
    for byte in pstr.iter(){
        msg.push(*byte);
    }
    for _ in 0..8{
        msg.push(0); // reserved part with 8 zero bytes
    }
    for byte in info_hash.iter(){
        msg.push(*byte);
    }
    for byte in peer_id.iter(){
        msg.push(*byte);
    }
    msg
}

#[cfg(test)]
#[test]
fn create_handshake_default_msg_test() {
    let info_hash = vec![255, 125, 75, 51, 96, 126, 249, 69, 90, 173, 209, 54, 159, 46, 10, 142, 230, 141, 83, 200];
    let peer_id = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20];
    let result = vec![
                      19, 66, 105, 116, 84, 111, 114, 114, 101, 110, 116, 32, 112, 114, 111, 116, 111, 99, 111, 108,
                      0, 0, 0, 0, 0, 0, 0, 0,
                      255, 125, 75, 51, 96, 126, 249, 69, 90, 173, 209, 54, 159, 46, 10, 142, 230, 141, 83, 200,
                      1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20
                      ];
    assert_eq!(create_handshake_msg(&info_hash, &peer_id, None), result);
}

#[test]
fn create_handshake_msg_test() {
    let info_hash = vec![255, 125, 75, 51, 96, 126, 249, 69, 90, 173, 209, 54, 159, 46, 10, 142, 230, 141, 83, 200];
    let peer_id = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20];
    let pstr = "ajhfhavbghajgjahwygajbg".to_string();
    let result = vec![
                      23, 97, 106, 104, 102, 104, 97, 118, 98, 103, 104, 97, 106, 103, 106, 97, 104, 119, 121, 103, 97, 106, 98, 103,
                      0, 0, 0, 0, 0, 0, 0, 0,
                      255, 125, 75, 51, 96, 126, 249, 69, 90, 173, 209, 54, 159, 46, 10, 142, 230, 141, 83, 200,
                      1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20
                      ];
    assert_eq!(create_handshake_msg(&info_hash, &peer_id, Some(pstr)), result);
}
