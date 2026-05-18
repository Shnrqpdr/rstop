# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Repository setup gotcha

The Cargo manifest is currently checked out as `CARGO~1.TOM` (a mangled 8.3 short
filename, likely from a WSL/Windows path). Cargo will not find it. Before any
build/test command works, rename it:

```bash
mv CARGO~1.TOM Cargo.toml
```

All commands below assume `Cargo.toml` exists.

## Commands

```bash
cargo build                 # debug build
cargo build --release       # optimized (thin LTO, stripped) -> target/release/rstop
cargo run -- --interval 500 --history 200 --detailed
cargo run -- --cpu-cores            # detail only the CPU panel
cargo build --no-default-features   # build without NVIDIA/NVML GPU support
cargo clippy                # lint
cargo test                  # run tests (no test suite exists yet)
cargo test <name>           # run a single test by name substring
```

The `nvidia` feature is on by default and pulls in `nvml-wrapper`. Disable it
with `--no-default-features` on machines without the NVIDIA driver/NVML.

## Platform constraints

Linux-only. Thermal collection (`src/metrics/temp.rs`) reads `/sys/class/hwmon`
directly; GPU collection talks to NVML. Both gracefully return empty rather than
erroring — an empty temp panel usually means the environment (hwmon is commonly
absent under WSL2), not a bug.

NVML init has a WSL fallback: `nvml-wrapper`'s `Nvml::init()` dlopens the
unversioned `libnvidia-ml.so`, which is absent on WSL (only
`/usr/lib/wsl/lib/libnvidia-ml.so.1` ships) and on hosts without the CUDA `-dev`
package. `nv::init_nvml` (`metrics/gpu.rs`) retries with the versioned soname
and the WSL path. To verify GPU detection without the TUI:
`cargo test gpu_probe -- --nocapture`.

## Architecture

A terminal system monitor (ratatui + crossterm) split into two concurrent
halves over a shared `Arc<RwLock<AppState>>`:

- **Render loop** (`main.rs::run_loop`, async): redraws at a fixed 100ms cadence
  (`app::render_interval`, independent of the sampling interval) and handles
  keyboard input. Reads state under a read lock.
- **Poller** (`app::spawn_poller`, a `spawn_blocking` OS thread): owns the
  `Collectors`, samples every `config.interval`, and writes the new `Snapshot`
  plus pushes into `Histories` under a write lock. It reaches the async RwLock
  via `tokio::runtime::Handle::block_on`. Skips sampling while `paused`.

The runtime is `multi_thread` with `worker_threads = 2`; the blocking poller
runs outside the worker pool.

### Data flow

`Collectors` (`metrics/mod.rs`) aggregates four independent collectors:
- `cpu` / `mem` — via `sysinfo::System` (refreshed each tick)
- `temp` — direct `/sys/class/hwmon` parsing, classifying chips named
  `coretemp`/`k10temp`/`zenpower`/`k8temp` as CPU sensors vs. "other"
- `gpu` — `GpuCollector` wrapping an NVML backend that is compiled out entirely
  when the `nvidia` feature is off (`#[cfg(feature = "nvidia")]` swaps the `nv`
  module for a no-op stub). All GPU fields are `Option`; absent metrics render
  as unavailable rather than zero.

Each tick produces a `Snapshot` (current values) which is also fed into
`Histories` — fixed-capacity `VecDeque` ring buffers (`history.rs`) backing the
aggregate line charts (cpu total, ram/swap %, per-GPU util/vram, max CPU temp).
Per-core and per-GPU history vectors are lazily (re)allocated when the detected
core/GPU count changes.

### UI

`ui::render` draws a single stacked screen: CPU, Memory, GPU and Thermals
panels top-to-bottom (header/footer on the outer rows). There are no layout
modes or tabs.

Each panel has two renderers in `ui/widgets.rs` and picks one based on
`config.detail.<panel>`:
- **default (aggregate)** — a `Chart` line graph of that metric over time
  (`line_chart` helper: X = time, Y = 0–`y_max`). CPU = total %, Memory =
  RAM%/Swap% as two datasets, GPU = two side-by-side charts (util% and VRAM%,
  one dataset per device each), Thermals = max CPU sensor °C.
- **detailed** — the htop-style breakdown: per-core gauges, RAM/Swap gauges +
  sparkline, per-GPU blocks, per-sensor list.

Adding a panel means: a `History` field for its aggregate series (pushed in
`Histories::push`), and a `<panel>_panel` fn that branches on its detail flag.

### Config

CLI args (`config::CliArgs`, clap derive) convert into a `Config` with clamping:
interval floored at 50ms, history floored at 8 samples. The interval is mutable
at runtime via `+`/`-` keys (clamped 50ms–10s), so `config` lives inside
`AppState` rather than being read-only.

Detail flags collapse into `Config::detail` (`config::Detail`): the global
`--detailed`/`-d` OR-s into every panel; `--cpu-cores`, `--mem-detailed`,
`--gpu-detailed`, `--temp-detailed` toggle one panel each. Widgets read
`state.config.detail.<panel>` only — never the raw CLI args.
