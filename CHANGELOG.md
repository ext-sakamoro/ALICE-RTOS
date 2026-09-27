# Changelog

All notable changes to ALICE-RTOS will be documented in this file.

## [Unreleased]

### Changed
- **License: `AGPL-3.0` → `AGPL-3.0 OR LicenseRef-Commercial` (dual-licensed、2026-09-27)** AGPL 側の条件は変更なし (既存 AGPL 利用者への影響ゼロ)、商用という選択肢が追加されただけ SPDX が AGPL 単独だと cargo-deny / FOSSA / SBOM に「商用オプションなし」と見えるため宣言を dual に 変更点: SPDX / `LICENSE` → `LICENSE-AGPL` rename / `LICENSE-COMMERCIAL.md` (商用トリガー 6 条件 = クローズド製品・商用 SaaS・エッジ・ファームウェア配布・plugin 再配布・プラットフォーム NDA・保証、社内利用は AGPL 側で無償と明記) / README の選択肢表 商用窓口は法人 `contact@extoria.co.jp`

## [0.1.0] - 2026-02-23

### Added
- `task` — static no-alloc task descriptors with `TaskPriority` (CRITICAL/HIGH/NORMAL/LOW/IDLE) and WCET
- `scheduler` — Rate-Monotonic scheduler with deadline tracking (max 16 tasks)
- `timer` — hardware-abstracted system timer (tick / µs / ms conversion)
- `spsc` — lock-free single-producer single-consumer ring buffer (power-of-two capacity)
- `kernel` — top-level RTOS manager (scheduler + timer + 1 KB scratch, < 2 KB total)
- `edge_tasks` (feature `edge`): ALICE-Edge task templates (1 kHz inference, configurable)
- `synth_tasks` (feature `synth`): ALICE-Synth task templates (44.1/48 kHz audio, voice budget calc)
- `motion_tasks` (feature `motion`): ALICE-Motion task templates (10 kHz NURBS, servo/stepper)
- C-ABI FFI bindings: 66 `extern "C"` functions (feature `ffi`)
- Unity C# bindings: 66 `[DllImport]` wrappers + RAII handles (`bindings/unity/AliceRtos.cs`)
- UE5 C++ bindings: 66 `extern "C"` declarations + 5 RAII handles (`bindings/ue5/AliceRtos.h`)
- PyO3 Python bindings: 5 classes — Kernel, KernelStats, Scheduler, SysTimer, SpscRing (feature `python`)
- `#![no_std]` — zero heap, zero external dependencies (core), `std` available via `ffi`/`python` feature
- Feature flags: `cortex-m`, `riscv`, `esp32`, `edge`, `synth`, `motion`, `ffi`, `python`, `std`
- 58 tests (33 core + 11 FFI + 13 task templates + 1 doc-test)
