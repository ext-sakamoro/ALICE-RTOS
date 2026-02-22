# Changelog

All notable changes to ALICE-RTOS will be documented in this file.

## [0.1.0] - 2026-02-23

### Added
- `task` — static no-alloc task descriptors with `TaskPriority` (CRITICAL/HIGH/NORMAL/LOW/IDLE) and WCET
- `scheduler` — Rate-Monotonic scheduler with deadline tracking (max 16 tasks)
- `timer` — hardware-abstracted system timer (tick / µs / ms conversion)
- `spsc` — lock-free single-producer single-consumer ring buffer (power-of-two capacity)
- `kernel` — top-level RTOS manager (scheduler + timer + 1 KB scratch, < 2 KB total)
- `#![no_std]` — zero heap, zero external dependencies
- Feature flags: `cortex-m`, `riscv`, `esp32`, `edge`, `synth`, `motion`
- 33 unit tests + 1 doc-test
