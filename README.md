# rustorrent

## About

This is my implementation of torrent client. We don't encourage piracy, use for legal purposes only.

## Usage

`cargo run --release path_to_torrent_file.torrent`

## Further upgrades

Right now there are some problems and missing features (in order of need to fix or implement): <br/>
1. Test properly file saving (it should work, but who knows, trivial things work)
2. Code refactoring (it's just a mess RN).
3. Proper error handling.
4. UDP connections support.
5. Performance issues (threads are obviously not the best approach here, something like epoll would be better). <br/>

<br/> Also it can stuck when a couple of pieces are left. Latest test have shown
this is probably just choked peers or something like that, but I'm not 100% sure about it.
