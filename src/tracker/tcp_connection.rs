use crate::torrent_file_handler::torrent_data_extractor::TorrentData;
use crate::torrent_file_handler::torrent_file_parser::parse_byte_data;
use curl::easy::Easy;

pub async fn make_tcp_request(
    tracker: String,
    torrent_data: &TorrentData,
    peer_id: &Vec<u8>,
    port: u16,
    info_hash: &Vec<u8>,
) -> anyhow::Result<(Vec<String>, i64)> {
    let mut data = Vec::new();
    let mut peers_list: Vec<String> = Vec::new();

    let url = create_tcp_tracker_url(tracker, torrent_data, peer_id, port, info_hash);
    let mut tracker = Easy::new();
    tracker.url(&url)?;
    tracker.timeout(std::time::Duration::from_millis(20000))?;
    {
        let mut transfer = tracker.transfer();
        transfer.write_function(|new_data| {
            data.extend_from_slice(new_data);
            Ok(new_data.len())
        })?;
        transfer.perform()?;
    }
    let response_data = parse_byte_data(&data)?;
    anyhow::ensure!(
        response_data.get("failure reason").is_none(),
        "Announce failure response: {:?}",
        response_data.get("failure reason").unwrap()
    );

    let peers = response_data
        .get("peers")
        .ok_or(anyhow::anyhow!("No 'peers' field in responce"))?
        .get_bytes()
        .ok_or(anyhow::anyhow!("Couldn't get bytes"))?;

    let interval = *response_data
        .get("interval")
        .ok_or(anyhow::anyhow!("No 'interval' field in responce"))?
        .get_int()
        .ok_or(anyhow::anyhow!("Couldn't get int"))?;

    anyhow::ensure!(peers.len() % 6 == 0, "Corrupted peers data");

    let mut ip = String::new();
    let mut port: u16 = 0;

    for (index, number) in peers.iter().enumerate() {
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

    Ok((peers_list, interval))
}

fn create_tcp_tracker_url(
    tracker: String,
    torrent_data: &TorrentData,
    peer_id: &Vec<u8>,
    port: u16,
    info_hash: &Vec<u8>,
) -> String {
    let mut url = String::new();

    url.push_str(&tracker);
    url.push_str("?compact=1&downloaded=0&info_hash=");

    url.push_str(&bytes_to_url(info_hash));

    url.push_str("&left=");
    let mut length: usize = 0;
    for file in &torrent_data.files {
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

fn bytes_to_url(bytes: &Vec<u8>) -> String {
    let mut url = String::new();
    for number in bytes {
        url.push('%');
        url.push_str(&u8_decimal_to_hex(number));
    }
    url
}

fn u8_decimal_to_hex(decimal: &u8) -> String {
    let mut hex = String::new();
    let first_digit = decimal / 16;
    let second_digit = decimal % 16;
    if first_digit > 9 {
        hex.push(('A' as u8 + first_digit - 10) as char);
    } else {
        hex.push(('0' as u8 + first_digit) as char);
    }
    if second_digit > 9 {
        hex.push(('A' as u8 + second_digit - 10) as char);
    } else {
        hex.push(('0' as u8 + second_digit) as char);
    }
    hex
}
