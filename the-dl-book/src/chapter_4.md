# BitTorrent: 2000 Lines of Controlled Chaos

```admonish abstract title="Chapter Guide"
**What you'll build:** BitTorrent protocol implementation with tracker support, peer wire protocol, piece management, and concurrent peer connections.

**What you'll learn:** P2P architecture, bencode parsing, TCP peer communication, managing 50+ concurrent connections, piece selection algorithms, SHA1 verification, tracker protocols (HTTP + UDP).

**End state:** Task 19 completed. `BitTorrentProtocol` implementation can download from .torrent files and magnet links, connect to trackers, exchange pieces with peers, and assemble the final file. DHT support is optional/deferred.
```

