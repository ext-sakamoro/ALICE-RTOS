# Contributing to ALICE-RTOS

## Build

```bash
cargo build
```

## Test

```bash
cargo test
```

## Lint

```bash
cargo clippy -- -W clippy::all
cargo fmt -- --check
cargo doc --no-deps 2>&1 | grep warning
```

## Design Constraints

- **`#![no_std]`**: the entire crate must compile without `std` or `alloc`. No heap, no `Vec`, no `String`.
- **Static task table**: max 16 tasks, compile-time bounded. No dynamic allocation.
- **Rate-Monotonic Scheduling**: priority = 1 / period. Shorter period = higher priority.
- **< 2 KB total footprint**: scheduler (~512 B) + timer (16 B) + scratch (1 KB).
- **SPSC ring**: power-of-two capacity, lock-free, single-producer single-consumer only.
- **`opt-level = "z"`**: release profile optimises for size (flash-constrained targets).
