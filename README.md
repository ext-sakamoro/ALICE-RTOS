# ALICE-RTOS

**Math-First Real-Time OS — Don't schedule processes, schedule equations**

> "An RTOS that knows the math can guarantee the deadline."

```
FreeRTOS:    Generic task scheduler + tick interrupt + heap allocator
ALICE-RTOS:  Equation-aware scheduler + zero-alloc + deterministic eval
```

## The Problem

ALICE-Edge fits a linear model in 751ns. ALICE-Synth generates audio at 44.1kHz. ALICE-Motion evaluates NURBS trajectories at 10kHz. On bare-metal MCUs (Cortex-M, RISC-V, ESP32), these workloads currently run either:

1. **Bare-metal super-loop** — no preemption, priority inversion, missed deadlines
2. **FreeRTOS / Zephyr** — generic scheduler overhead, heap fragmentation, non-deterministic latency

Neither is optimized for **periodic mathematical evaluation** — the core workload of every ALICE crate.

## The Solution

A minimal RTOS kernel designed exclusively for deterministic equation evaluation:

- **Static task table** — no dynamic allocation, no heap, no fragmentation
- **Rate-Monotonic Scheduling** — mathematically provable deadline guarantees
- **Zero-copy task communication** — lock-free SPSC ring buffers between tasks
- **Equation-aware priorities** — synth at 44.1kHz > motion at 10kHz > edge at 1kHz

The entire kernel fits in < 2 KB of flash and uses < 256 bytes of RAM for the scheduler.

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         ALICE-RTOS                               │
│                    (< 2 KB flash, < 256 B RAM)                   │
├─────────────────────────────────────────────────────────────────┤
│                                                                   │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  Task Table (static, compile-time allocated)              │   │
│  │                                                            │   │
│  │  [0] ALICE-Synth    period=22.7µs  (44.1kHz)  prio=0    │   │
│  │  [1] ALICE-Motion   period=100µs   (10kHz)    prio=1    │   │
│  │  [2] ALICE-Edge     period=1ms     (1kHz)     prio=2    │   │
│  │  [3] ALICE-Sync     period=16.7ms  (60Hz)     prio=3    │   │
│  │  [4] Telemetry      period=1s      (1Hz)      prio=4    │   │
│  └──────────────────────────────────────────────────────────┘   │
│                             │                                     │
│              ┌──────────────┼──────────────┐                     │
│              ▼              ▼              ▼                     │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐            │
│  │  Timer HAL   │ │  Context     │ │  IPC Rings   │            │
│  │  (SysTick /  │ │  Switch      │ │  (lock-free  │            │
│  │   RISC-V     │ │  (< 1 µs)   │ │   SPSC)      │            │
│  │   mtime)     │ │              │ │              │            │
│  └──────────────┘ └──────────────┘ └──────────────┘            │
│                                                                   │
│  ┌─────────────────────────────────────────────────┐            │
│  │  Hardware Abstraction Layer (HAL)                │            │
│  │  Cortex-M (NVIC) │ RISC-V (PLIC) │ Xtensa      │            │
│  └─────────────────────────────────────────────────┘            │
│                                                                   │
└─────────────────────────────────────────────────────────────────┘
```

## Scheduling Model

### Rate-Monotonic Scheduling (RMS)

Tasks are prioritized by period — shorter period = higher priority. RMS guarantees all deadlines are met if:

```
U = Σ (Cᵢ / Tᵢ) ≤ n(2^(1/n) - 1)

Where:
  Cᵢ = worst-case execution time of task i
  Tᵢ = period of task i
  n  = number of tasks
```

### ALICE Task Budget Example (Raspberry Pi Pico, Cortex-M0+ @ 133MHz)

| Task | Period | WCET | Utilization |
|------|--------|------|-------------|
| ALICE-Synth (4-voice FM) | 22.7 µs | 8 µs | 35.2% |
| ALICE-Motion (3-DOF NURBS) | 100 µs | 15 µs | 15.0% |
| ALICE-Edge (100-sample fit) | 1 ms | 50 µs | 5.0% |
| ALICE-Sync (input frame) | 16.7 ms | 200 µs | 1.2% |
| Telemetry (dashboard) | 1 s | 500 µs | 0.05% |
| **Total** | | | **56.5%** |

RMS bound for 5 tasks: 74.3% → **schedulable with 17.8% margin**.

## Kernel Primitives

### Static Task Definition (compile-time)

```rust
#[alice_task(period_us = 100, priority = 1, stack_size = 256)]
fn motion_task(ctx: &mut TaskContext) {
    let pos = ctx.shared.trajectory.position(ctx.time());
    ctx.shared.actuator.set_position(pos);
}
```

### Lock-Free IPC (SPSC Ring Buffer)

```
Producer (high-prio)              Consumer (low-prio)
     │                                  │
     ▼                                  ▼
┌─────────────────────────────────────────┐
│  [  ][  ][WR][  ][  ][RD][  ][  ]      │  16-slot ring
│       write_idx ──▶    read_idx ──▶     │  No mutex, no CAS
└─────────────────────────────────────────┘
```

### Minimal Context Switch

```
Cortex-M:   Push R4-R11 + PSP → Switch MSP → Pop R4-R11 + PSP
            Total: 12 cycles (~90 ns @ 133 MHz)

RISC-V:     Push s0-s11 + sp → Switch tp → Pop s0-s11 + sp
            Total: 16 cycles (~120 ns @ 133 MHz)
```

## Memory Layout

```
Flash (< 2 KB total):
┌──────────────────────────────┐
│  Kernel code         (1.2 KB)│
│  Task table (static) (0.2 KB)│
│  ISR vectors         (0.1 KB)│
│  IPC ring descriptors(0.1 KB)│
└──────────────────────────────┘

RAM (< 256 B kernel + task stacks):
┌──────────────────────────────┐
│  Scheduler state      (64 B) │
│  Task control blocks (128 B) │  ← 5 tasks × 24 B + padding
│  IPC ring metadata    (48 B) │
│  ──── task stacks ────────── │  ← 256 B per task (configurable)
└──────────────────────────────┘
```

## Size Comparison

| RTOS | Flash | RAM (kernel) | Context Switch | Heap Required |
|------|-------|-------------|----------------|--------------|
| FreeRTOS | 6-10 KB | 1-2 KB | ~200 cycles | Yes |
| Zephyr | 8-20 KB | 2-4 KB | ~150 cycles | Optional |
| RIOT | 5-10 KB | 1.5 KB | ~180 cycles | Optional |
| **ALICE-RTOS** | **< 2 KB** | **< 256 B** | **~12 cycles** | **No** |

## API Design

```rust
#![no_std]
#![no_main]

use alice_rtos::{Kernel, Task, SpscRing, hal};

// Static task table (compile-time)
static TASKS: &[Task] = &[
    Task::new("synth",  synth_task,  22,   0, 512),  // 22µs, prio 0
    Task::new("motion", motion_task, 100,  1, 256),  // 100µs, prio 1
    Task::new("edge",   edge_task,   1000, 2, 256),  // 1ms, prio 2
];

// Lock-free IPC: edge → motion (sensor feedback)
static SENSOR_RING: SpscRing<SensorReading, 16> = SpscRing::new();

#[alice_rtos::entry]
fn main() -> ! {
    let hal = hal::init(); // Platform-specific HAL
    let mut kernel = Kernel::new(&hal, TASKS);

    // Verify schedulability at boot
    assert!(kernel.utilization() < kernel.rms_bound());

    kernel.start() // Never returns
}

fn edge_task(ctx: &mut TaskContext) {
    let samples = ctx.hal.adc.read_batch(100);
    let (slope, intercept) = alice_edge::fit_linear_fixed(&samples);
    SENSOR_RING.push(SensorReading { slope, intercept });
}

fn motion_task(ctx: &mut TaskContext) {
    if let Some(reading) = SENSOR_RING.pop() {
        ctx.shared.trajectory.adjust(&reading);
    }
    let pos = ctx.shared.trajectory.position(ctx.time());
    ctx.hal.pwm.set_duty(pos);
}
```

## Target Platforms

| Platform | CPU | Clock | Flash | RAM |
|----------|-----|-------|-------|-----|
| Raspberry Pi Pico | Cortex-M0+ (RP2040) | 133 MHz | 2 MB | 264 KB |
| STM32F4 | Cortex-M4F | 168 MHz | 512 KB | 128 KB |
| STM32H7 | Cortex-M7 | 480 MHz | 2 MB | 1 MB |
| ESP32-C3 | RISC-V | 160 MHz | 4 MB | 400 KB |
| GD32VF103 | RISC-V | 108 MHz | 128 KB | 32 KB |
| ESP32-S3 | Xtensa LX7 | 240 MHz | 8 MB | 512 KB |

## Ecosystem Integration

```
┌──────────────────────────────────────────────────────┐
│                    ALICE-RTOS Kernel                   │
│                                                        │
│  ┌────────────┐ ┌────────────┐ ┌────────────┐        │
│  │ALICE-Synth │ │ALICE-Motion│ │ ALICE-Edge │        │
│  │ 44.1 kHz   │ │  10 kHz    │ │   1 kHz    │        │
│  │ FM/Additive│ │ NURBS eval │ │ fit_linear │        │
│  └─────┬──────┘ └─────┬──────┘ └─────┬──────┘        │
│        │ IPC          │ IPC          │ IPC            │
│        ▼              ▼              ▼                │
│  ┌────────────┐ ┌────────────┐ ┌────────────┐        │
│  │  DAC/I2S   │ │  PWM/Step  │ │  ADC/I2C   │        │
│  │  (audio)   │ │  (motors)  │ │  (sensors) │        │
│  └────────────┘ └────────────┘ └────────────┘        │
│                                                        │
└──────────────────────────────────────────────────────┘
```

## Feature Flags

| Feature | Dependencies | Description |
|---------|-------------|-------------|
| *(default)* | None | Kernel + scheduler, pure no_std |
| `cortex-m` | None | ARM Cortex-M HAL (NVIC, SysTick) |
| `riscv` | None | RISC-V HAL (PLIC, mtime) |
| `esp32` | None | Xtensa/RISC-V ESP32 HAL |
| `edge` | None | ALICE-Edge task template |
| `synth` | None | ALICE-Synth task template |
| `motion` | None | ALICE-Motion task template |

## License

AGPL-3.0

## Author

Moroya Sakamoto

---

*"The best RTOS is one that knows what it's computing."*
