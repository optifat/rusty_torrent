use std::net::{TcpStream, SocketAddr};
use std::time::Duration;
use std::io;
use std::io::prelude::*;

pub fn perform_handshake(peer_ip: String, info_hash: Vec<u8>, peer_id: Vec<u8>, pstr_option: Option<String>) -> Result<TcpStream, io::Error>{
    //println!("Performing handshake with {:?}", peer_ip);
    match TcpStream::connect_timeout(&peer_ip.parse::<SocketAddr>().unwrap(), Duration::new(3, 0)) {
        Ok(mut stream) => {
            stream.write(&create_handshake_msg(&info_hash, &peer_id, pstr_option)).unwrap(); // my panic code: 104, kind: ConnectionReset, message: "Connection reset by peer"
            let mut buf: [u8; 1] = [0; 1];
            let mut pstr_len: [u8; 1] = [0];
            let mut pstr_and_reserved = Vec::new();
            let mut hash: [u8; 20] = [0; 20];
            let mut id: [u8; 20] = [0; 20];
            match stream.read(&mut pstr_len){
                Ok(_) => {},
                Err(err) => {
                    println!("{:?}", err);
                    return Err(err);
                }
            }

            for _ in 0..pstr_len[0]+8{
                stream.read(&mut buf).unwrap();
                pstr_and_reserved.push(buf[0]);
            }
            stream.read(&mut hash).unwrap();
            stream.read(&mut id).unwrap();
            for i in 0..20{
                if hash[i] != info_hash[i] {
                    //println!("Hash infos don't match with {:?}", peer_ip);
                    return Err(io::Error::new(io::ErrorKind::Other, "Hash infos do not match"));
                }
            }
            //println!("Connected to {:?}", peer_ip);
            Ok(stream)
        },
        Err(_) => {
            //println!("Failed to connect to {:?}", peer_ip);
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
