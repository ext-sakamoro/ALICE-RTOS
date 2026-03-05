# Changelog

All notable changes to ALICE-RTOS will be documented in this file.

## [0.1.0] - 2026-02-23

### Added
- `task` — static no-alloc task descriptors with `TaskPriority` (CRITICAL/HIGH/NORMAL/LOW/IDLE) and WCET
- `scheduler` — Rate-Monotonic scheduler with deadline tracking (max 16 tasks)
- `timer` — hardware-abstracted system timer (tick / µs / ms conversion)
- `spsc` — lock-free single-producer single-consumer ring buffer (power-of-two capacity)
- `kernel` — top-level RTOS manager (scheduler + timer + 1 KB scratch, < 2 KB total)
- C-ABI FFI bindings: 66 `extern "C"` functions (feature `ffi`)
- Unity C# bindings: 66 `[DllImport]` wrappers + RAII handles (`bindings/unity/AliceRtos.cs`)
- UE5 C++ bindings: 66 `extern "C"` declarations + 5 RAII handles (`bindings/ue5/AliceRtos.h`)
- `#![no_std]` — zero heap, zero external dependencies (core), `std` available via `ffi` feature
- Feature flags: `cortex-m`, `riscv`, `esp32`, `edge`, `synth`, `motion`, `ffi`, `std`
- 45 tests (33 core + 11 FFI + 1 doc-test)
