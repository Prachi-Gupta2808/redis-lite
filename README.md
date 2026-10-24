# Redis Lite

A small Redis-style key-value store written in Rust with Tokio, built step by step as a learning project. A TCP load balancer that distributes traffic across several store instances is planned next.

> **Status:** It's a work in progress everyone!

## Project structure

This is a Cargo workspace (monorepo) with two crates:

```
redis-lite/
├── Cargo.toml          # workspace manifest
├── mini-redis/         # the key-value server
└── load-balancer/      # round-robin TCP proxy (planned)
```

| Crate | What it is |
|---|---|
| `mini-redis` | An async TCP server that stores key-value pairs in memory and is built on Tokio. |
| `load-balancer` | A lightweight TCP proxy that forwards client connections to several `mini-redis` instances using round-robin. |

## mini-redis

### Supported commands

| Command | Description |
|---|---|
| `SET key value` | Store a value. Values may contain spaces. |
| `SET key value EX seconds` | Store a value that expires after the given number of seconds. |
| `GET key` | Fetch a value, or nil if the key is missing or expired. |
| `DEL key` | Delete a key. |
| `EXISTS key` | Check whether a key exists and has not expired. |
| `TTL key` | Seconds until the key expires. `-1` means no expiry, `-2` means no such key. |
| `EXPIRE key seconds` | Set or change a key's expiry. |
| `PING` | Health check, replies `PONG`. |

Commands are case-insensitive. The protocol is plain text, one command per line, so you can talk to the server with `telnet` or `nc`.

### Known limitations

- A value cannot literally end with `EX <number>`, because that is read as a TTL.
- Repeated spaces in a value are collapsed into one.
- No persistence: all data is lost when the server stops.
- Only string values are supported.

## Getting started

You need a recent stable Rust toolchain (install from [rustup.rs](https://rustup.rs)).

```bash
git clone https://github.com/Prachi-Gupta2808/redis-lite.git
cd redis-lite

# run the tests
cargo test -p mini-redis

# start the server on 127.0.0.1:6380
cargo run -p mini-redis
```

Then connect from another terminal:

```bash
telnet 127.0.0.1 6380
```

## What I'm learning

Async Rust with Tokio, ownership across tasks, shared state with `Arc` and `Mutex`, designing a small text protocol, and structuring a multi-crate Cargo workspace.
