# IPFS: Files Have Fingerprints Now, Apparently

```admonish abstract title="Chapter Guide"
**What you'll build:** IPFS protocol implementation with gateway fallback and local daemon support.

**What you'll learn:** Content-addressed storage, CID parsing and verification, IPFS gateway API, DAG traversal for large files, IPNS (mutable content addresses).

**End state:** Task 20 completed. `IpfsProtocol` implementation can resolve CIDs via HTTP gateways or local IPFS daemon, handle multi-block DAG structures, and verify content hashes.
```

