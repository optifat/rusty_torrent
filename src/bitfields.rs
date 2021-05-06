use std::net::TcpStream;
use std::io;
use std::io::Read;

pub fn parse_bitfield(stream: &mut TcpStream, expected_length: usize) -> Result<Vec<u8>, io::Error>{
    let mut length: [u8; 4] = [0; 4];
    stream.read(&mut length).unwrap();
    let length = u32::from_be_bytes(length) - 1; // bitfield length + 1 for id

    if length as usize != expected_length{
        return Err(io::Error::new(io::ErrorKind::Other, "Expected and recieved lengths don't match"));
    }

    let mut buf: [u8; 1] = [0];

    stream.read(&mut buf).unwrap();
    let id = buf[0];
    if id != 5{
        return Err(io::Error::new(io::ErrorKind::Other, "Wrong message id"));
    }

    let mut bitfield = Vec::new();
    for _ in 0..length{
        stream.read(&mut buf).unwrap();
        bitfield.push(buf[0]);
    }
    Ok(bitfield)
}
