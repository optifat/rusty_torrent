use crate::torrent_file_handler::torrent_data_extractor::TorrentData;
use portpicker::pick_unused_port;
use std::convert::TryInto;
use std::net::UdpSocket;
use std::time;
use url::Url;

/*
 *   Specs can be found here: https://www.bittorrent.org/beps/bep_0015.html
 */

pub async fn make_udp_request(
    url: Url,
    torrent_data: &TorrentData,
    peer_id: &Vec<u8>,
    port: u16,
    info_hash: &Vec<u8>,
) -> anyhow::Result<(Vec<String>, i64)> {
    let binding_port = pick_unused_port().ok_or(anyhow::anyhow!("Couldn't pick unused port"))?;
    let binding_ip = format!("0.0.0.0:{}", binding_port);

    let link = format!(
        "{}:{}",
        url.host()
            .ok_or(anyhow::anyhow!("Couldn't get url hostname"))?,
        url.port().ok_or(anyhow::anyhow!("Couldn't get url port"))?
    );

    let socket = UdpSocket::bind(binding_ip).expect("couldn't bind to address");
    socket.set_read_timeout(Some(time::Duration::new(20, 0)))?;
    socket.set_write_timeout(Some(time::Duration::new(20, 0)))?;

    socket.connect(link)?;

    let (handshake, transaction_id) = create_udp_handshake();
    socket.send(&handshake).expect("couldn't send message");
    let mut connect_response: [u8; 16] = [0; 16];
    let _bytes_recieved = socket.recv(&mut connect_response)?;

    let connection_id = check_udp_response(connect_response, transaction_id)?;

    let (announce_msg, transaction_id) =
        create_udp_announce(connection_id, torrent_data, peer_id, port, info_hash);

    socket.send(&announce_msg)?;

    let mut announce_response: [u8; 1024] = [0; 1024];
    let bytes_recieved = socket.recv(&mut announce_response)?;

    parse_udp_announce_response(
        announce_response[0..bytes_recieved].to_vec(),
        transaction_id,
    )
}

fn create_udp_handshake() -> (Vec<u8>, u32) {
    let mut handshake_bytes = Vec::new();
    let protocol_id: u64 = 4497486125440; // magical constant 0x41727101980
    let action: u32 = 0; // connect
    let transaction_id: u32 = 54676;

    for byte in protocol_id.to_be_bytes().iter() {
        handshake_bytes.push(*byte);
    }

    for byte in action.to_be_bytes().iter() {
        handshake_bytes.push(*byte);
    }

    for byte in transaction_id.to_be_bytes().iter() {
        handshake_bytes.push(*byte);
    }

    (handshake_bytes, transaction_id)
}

fn check_udp_response(response: [u8; 16], transaction_id: u32) -> anyhow::Result<u64> {
    anyhow::ensure!(
        u32::from_be_bytes(response[0..4].try_into()?) == 0,
        "Wrong connection id"
    );

    anyhow::ensure!(
        u32::from_be_bytes(response[4..8].try_into()?) == transaction_id,
        "Wrong transaction id"
    );

    Ok(u64::from_be_bytes(response[8..].try_into()?))
}

fn create_udp_announce(
    connection_id: u64,
    torrent_data: &TorrentData,
    peer_id: &Vec<u8>,
    port: u16,
    info_hash: &Vec<u8>,
) -> (Vec<u8>, u32) {
    let mut announce_bytes = Vec::new();

    for byte in connection_id.to_be_bytes().iter() {
        announce_bytes.push(*byte);
    }

    let action: u32 = 1;
    for byte in action.to_be_bytes().iter() {
        announce_bytes.push(*byte);
    }

    let transaction_id: u32 = 542178; // some random number
    for byte in transaction_id.to_be_bytes().iter() {
        announce_bytes.push(*byte);
    }

    for byte in info_hash {
        announce_bytes.push(*byte);
    }

    for byte in peer_id {
        announce_bytes.push(*byte);
    }

    let downloaded: u64 = 0;
    for byte in downloaded.to_be_bytes().iter() {
        announce_bytes.push(*byte);
    }

    let mut left: u64 = 0;
    for file in &torrent_data.files {
        left += file.size as u64;
    }
    for byte in left.to_be_bytes().iter() {
        announce_bytes.push(*byte);
    }

    let uploaded: u64 = 0;
    for byte in uploaded.to_be_bytes().iter() {
        announce_bytes.push(*byte);
    }

    let event: u32 = 0;
    for byte in event.to_be_bytes().iter() {
        announce_bytes.push(*byte);
    }

    let ip_address: u32 = 0; // default value
    for byte in ip_address.to_be_bytes().iter() {
        announce_bytes.push(*byte);
    }

    let key: u32 = 7561816; // random number
    for byte in key.to_be_bytes().iter() {
        announce_bytes.push(*byte);
    }

    let num_want: i32 = -1; // default value
    for byte in num_want.to_be_bytes().iter() {
        announce_bytes.push(*byte);
    }

    for byte in port.to_be_bytes().iter() {
        announce_bytes.push(*byte);
    }

    (announce_bytes, transaction_id)
}

fn parse_udp_announce_response(
    response: Vec<u8>,
    transaction_id: u32,
) -> anyhow::Result<(Vec<String>, i64)> {
    let mut peers_list: Vec<String> = Vec::new();
    anyhow::ensure!(
        u32::from_be_bytes(response[0..4].try_into()?) == 1,
        "Wrong action id"
    );

    anyhow::ensure!(
        u32::from_be_bytes(response[4..8].try_into()?) == transaction_id,
        "Wrong transaction id"
    );

    let interval = u32::from_be_bytes(response[8..12].try_into()?);
    let _number_of_leechers = u32::from_be_bytes(response[12..16].try_into()?);
    let _number_of_seeders = u32::from_be_bytes(response[16..20].try_into()?);

    let mut ip = String::new();
    let mut port: u16 = 0;

    for (index, number) in response[20..].iter().enumerate() {
        match index % 6 {
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
                port += (*number as u16) * 256;
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
