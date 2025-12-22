# Chapter 3 - FTP or Bust

```admonish abstract title="Chapter Guide"
**What you'll build:** FTP/FTPS protocol implementation that plugs into the daemon.

**What you'll learn:** Multi-channel protocols (control + data), active vs passive mode, state machines for protocol commands, TLS negotiation for FTPS, resume via REST command.

**End state:** Task 12 completed. `FtpProtocol` implementation registered in the daemon. Can download from public FTP servers with progress tracking and resume support.
```

