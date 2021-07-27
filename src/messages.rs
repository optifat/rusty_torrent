use std::convert::TryInto;
use std::io;
use std::{thread, time};

pub fn create_request_msg(index: u32, begin: u32, length: u32) -> Vec<u8>{
    let mut request = Vec::new();

    // prefix 13 in four-byte big-endian format
    for byte in (13 as u32).to_be_bytes().iter(){
        request.push(*byte);
    }

    // id = 6
    request.push(6);

    for byte in index.to_be_bytes().iter(){
        request.push(*byte);
    }
    for byte in begin.to_be_bytes().iter(){
        request.push(*byte);
    }
    for byte in length.to_be_bytes().iter(){
        request.push(*byte);
    }

    request
}

pub fn create_have_msg(index: u32) -> Vec<u8>{
    let mut have = Vec::new();

    // prefix 5 in four-byte big-endian format
    for byte in (5 as u32).to_be_bytes().iter(){
        have.push(*byte);
    }

    // id = 4
    have.push(4);

    for byte in index.to_be_bytes().iter(){
        have.push(*byte);
    }
    have
}

pub fn create_unchoke_msg() -> Vec<u8>{
    let mut msg = Vec::new();

    for byte in (1 as u32).to_be_bytes().iter(){
        msg.push(*byte);
    }
    msg.push(1);
    msg
}

pub fn create_interested_msg() -> Vec<u8>{
    let mut msg = Vec::new();

    for byte in (1 as u32).to_be_bytes().iter(){
        msg.push(*byte);
    }
    msg.push(2);
    msg
}

pub fn read_message(message: Vec<u8>) -> Option<Vec<u8>>{
    let id = message[4];
    match id{
        0 => {
            thread::sleep(time::Duration::new(30, 0));
            return None;
        }
        7 => {
            return Some(parse_piece_msg(message).unwrap());
        }
        _ => {}
    };
    None
}

pub fn parse_piece_msg(message: Vec<u8>) -> Result<Vec<u8>, io::Error>{
    if message.len() < 13{
        return Err(io::Error::new(io::ErrorKind::Other, "Message is too short"));
    }
    let id: u8 = message[4];
    if id != 7{
        return Err(io::Error::new(io::ErrorKind::Other, format!("Not a piece message: wrong id, expected 7, got {}", id)));
    }

    let len = u32::from_be_bytes(message[0..4].try_into().unwrap());
    if len < 9{
        return Err(io::Error::new(io::ErrorKind::Other, "Message announced length is too small"));
    }

    let index = u32::from_be_bytes(message[5..9].try_into().unwrap());
    let begin = u32::from_be_bytes(message[9..13].try_into().unwrap());

    if message[13..].len() != (len-9) as usize{
        return Err(io::Error::new(io::ErrorKind::Other, format!("Block length and announced length are different: expected {}, got {}, real len: {}", len-9, message[13..].len(), message.len())));
    }
    //println!("{:?}", message[13..].to_vec());
    Ok(message[13..].to_vec())
}

pub fn parse_have_msg(expected_index:u32, message: Vec<u8>){
    if message.len() != 9{
        println!("Wrong message length:\"Have\" message has fixed length of 9");
    }
    let id = message[4];
    if id != 4{
        println!("Wrong message id");
    }
    let index = u32::from_be_bytes(message[5..9].try_into().unwrap());
    if index != expected_index{
        println!("Wrong index");
    }
}
