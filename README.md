# rusty_torrent

## About

This is my implementation of torrent client. We don't encourage piracy, use for legal purposes only.

## Usage

`cargo run --release path_to_torrent_file.torrent`

## Further upgrades

Right now there are some problems and missing features (in order of need to fix or implement): <br/>
[ ] Proper testing (it can stuck in some places + didn't test file saving that good) <br/>
[ ] Code refactoring (it's just a mess RN) <br/>
[x] Proper error handling <br/>
[x] Performance issues (replaced multithreading with async) <br/>
[ ] Algorithms improvements <br/>

