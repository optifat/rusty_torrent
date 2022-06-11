use std::io::Read;
use std::net::TcpStream;

pub fn parse_bitfield(stream: &mut TcpStream, expected_length: usize) -> anyhow::Result<Vec<u8>> {
    let mut length: [u8; 4] = [0; 4];
    stream.read(&mut length)?;
    let length = u32::from_be_bytes(length) - 1; // bitfield length + 1 for id

    anyhow::ensure!(
        length as usize == expected_length,
        "Expected and recieved lengths don't match",
    );

    let mut buf: [u8; 1] = [0];

    stream.read(&mut buf)?;
    let id = buf[0];

    anyhow::ensure!(id == 5, "Wrong message id");

    let mut bitfield = Vec::new();
    for _ in 0..length {
        stream.read(&mut buf)?;
        bitfield.push(buf[0]);
    }
    Ok(bitfield)
}
