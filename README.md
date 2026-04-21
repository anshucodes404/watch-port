# watch_man

A small Rust CLI tool that inspects Linux TCP sockets from `/proc` and maps them to process IDs and names.

## What it does

- Reads TCP socket entries from `/proc/net/tcp`
- Parses local port and socket state
- Builds an inode -> PID map from `/proc/<pid>/fd`
- Resolves process names from `/proc/<pid>/comm`
- Sorts results by port

## Current status

This project currently parses and resolves data, but the output is still in a debug-oriented state:

- It prints raw parsed columns from `/proc/net/tcp`
- It prints the full inode/PID map
- `main` only prints a success/error message

So the core collection logic exists, but a final user-facing table/list output is not implemented yet.

## Requirements

- Linux (uses `/proc` paths specific to Linux)
- Rust toolchain (recommended: latest stable)

## Build and run

From the project root:

```bash
cargo run
```

Build release binary:

```bash
cargo build --release
```

Run tests (when added):

```bash
cargo test
```

## Project layout

- `src/main.rs`: TCP parsing and process mapping logic
- `Cargo.toml`: package and dependency configuration

## Notes

Dependencies currently include terminal/UI crates (`ratatui`, `crossterm`), but the present `main.rs` implementation is command-line and debug-print based.

## License

No license file is currently included in this repository.
