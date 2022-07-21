use crate::torrent_file_handler::torrent_data_extractor::TorrentData;
use futures::future::join_all;
use url::Url;

mod tcp_connection;
mod udp_connection;

pub async fn request_peers(
    torrent_data: &TorrentData,
    peer_id: &Vec<u8>,
    port: u16,
    info_hash: &Vec<u8>,
) -> anyhow::Result<(Vec<String>, i64)> {
    let mut announce_list = Vec::new();

    match &torrent_data.announce_list {
        Some(content) => {
            announce_list = content.to_vec();
        }
        None => {
            announce_list.push(torrent_data.announce.clone());
        }
    }

    Ok(make_requests(announce_list, torrent_data, peer_id, port, info_hash).await?)
}

async fn make_requests(
    announce_list: Vec<String>,
    torrent_data: &TorrentData,
    peer_id: &Vec<u8>,
    port: u16,
    info_hash: &Vec<u8>,
) -> anyhow::Result<(Vec<String>, i64)> {
    let mut peers = Vec::new();
    let mut interval = 0;

    let trackers_responce = join_all(
        announce_list
            .iter()
            .map(|tracker| make_request(tracker, torrent_data, peer_id, port, info_hash)),
    )
    .await
    .into_iter()
    .collect::<Result<Vec<_>, _>>()?;

    for (peers_in_responce, interval_result) in trackers_responce {
        for peer in peers_in_responce {
            peers.push(peer);
        }
        interval = interval_result;
    }
    // println!("{:?}", peers);
    Ok((peers, interval))
}

async fn make_request(
    tracker: &String,
    torrent_data: &TorrentData,
    peer_id: &Vec<u8>,
    port: u16,
    info_hash: &Vec<u8>,
) -> anyhow::Result<(Vec<String>, i64)> {
    // println!("{}", tracker);
    let url = Url::parse(&tracker)?;
    let is_udp = url.scheme() == "udp";
    if is_udp {
        udp_connection::make_udp_request(url, torrent_data, peer_id, port, info_hash).await
    } else {
        tcp_connection::make_tcp_request(tracker, torrent_data, peer_id, port, info_hash).await
    }
}
