# rustorrent

## About

This is my implementation of torrent client. We don't encourage piracy, use for legal purposes only.

## Usage

`cargo run --release path_to_torrent_file.torrent`

## Further updates

Right now there are some problems and missing features (in order of need to fix or implement): <br/>
1. Last pieces won't be downloaded in most of the cases.
2. No saving files.
3. Code refactoring (it's just a mess RN).
4. Performance issues.
