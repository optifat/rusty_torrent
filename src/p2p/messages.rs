use std::convert::TryInto;
use std::{thread, time};

pub fn create_request_msg(index: u32, begin: u32, length: u32) -> Vec<u8> {
    let mut request = Vec::new();

    // prefix 13 in four-byte big-endian format
    for byte in (13 as u32).to_be_bytes().iter() {
        request.push(*byte);
    }

    // id = 6
    request.push(6);

    for byte in index.to_be_bytes().iter() {
        request.push(*byte);
    }
    for byte in begin.to_be_bytes().iter() {
        request.push(*byte);
    }
    for byte in length.to_be_bytes().iter() {
        request.push(*byte);
    }

    request
}

#[allow(dead_code)]
pub fn create_have_msg(index: u32) -> Vec<u8> {
    let mut have = Vec::new();

    // prefix 5 in four-byte big-endian format
    for byte in (5 as u32).to_be_bytes().iter() {
        have.push(*byte);
    }

    // id = 4
    have.push(4);

    for byte in index.to_be_bytes().iter() {
        have.push(*byte);
    }
    have
}

pub fn create_unchoke_msg() -> Vec<u8> {
    let mut msg = Vec::new();

    for byte in (1 as u32).to_be_bytes().iter() {
        msg.push(*byte);
    }
    msg.push(1);
    msg
}

pub fn create_interested_msg() -> Vec<u8> {
    let mut msg = Vec::new();

    for byte in (1 as u32).to_be_bytes().iter() {
        msg.push(*byte);
    }
    msg.push(2);
    msg
}

pub fn read_message(message: Vec<u8>) -> anyhow::Result<(bool, Option<Vec<u8>>)> {
    // returns true if peer choked us
    let id = message[4];
    match id {
        0 => {
            thread::sleep(time::Duration::new(30, 0));
            return Ok((true, None));
        }
        7 => {
            return Ok((false, Some(parse_piece_msg(message)?)));
        }
        _ => {}
    };
    Ok((false, None))
}

pub fn parse_piece_msg(message: Vec<u8>) -> anyhow::Result<Vec<u8>> {
    anyhow::ensure!(message.len() >= 13, "Message is too short");

    let id: u8 = message[4];
    anyhow::ensure!(
        id == 7,
        format!("Not a piece message: wrong id, expected 7, got {}", id)
    );

    let len = u32::from_be_bytes(message[0..4].try_into()?);
    anyhow::ensure!(len >= 9, "Message announced length is too small");

    anyhow::ensure!(
        message[13..].len() == (len - 9) as usize,
        format!(
            "Block length and announced length are different: expected {}, got {}, real len: {}",
            len - 9,
            message[13..].len(),
            message.len()
        )
    );
    //println!("{:?}", message[13..].to_vec());
    Ok(message[13..].to_vec())
}
