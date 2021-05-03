use std::convert::TryInto;

fn create_request_msg(index: u32, begin: u32, length: u32) -> Vec<u8>{
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

fn create_have_msg(index: u32) -> Vec<u8>{
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

fn parse_piece_msg(message: Vec<u8>) -> Vec<u8>{
    if message.len() < 13{
        println!("Message too short");
    }
    let id: u8 = message[4];
    if id != 7{
        println!("Not a pisce message: wrong id");
    }

    let len = u32::from_be_bytes(message[0..4].try_into().unwrap());
    if len < 9{
        println!("Message too short");
    }

    let index = u32::from_be_bytes(message[5..9].try_into().unwrap());
    let begin = u32::from_be_bytes(message[9..13].try_into().unwrap());

    if message[13..].len() != (len-9) as usize{
        println!("Block length and announced length are different");
    }
    message[13..].to_vec()
}

fn parce_have_msg(expected_index:u32, message: Vec<u8>){
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
