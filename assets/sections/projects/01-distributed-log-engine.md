# DISTRIBUTED LOG ENGINE

An append-only write-ahead log built on raw `io_uring` / `kqueue` for sub-millisecond p99 commit latency under sustained 1M ops/sec.

**Stack**: Rust, io_uring, custom mmap'd ring buffers, deterministic snapshot replication.

**Lessons**: page-cache eviction is the real enemy; group-commit beats fsync amplification; tail-latency math eats you alive if you don't measure p99.9.
