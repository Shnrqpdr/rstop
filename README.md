# rstop

A terminal system monitor for Linux. It shows CPU, memory, GPU, and temperatures
as live line charts over time, so a sudden spike is something you can actually
see instead of a number that flickers for one refresh and is gone.

[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
![Rust 2021](https://img.shields.io/badge/rust-2021-orange.svg)

> Status: 0.1.0, the first release. Linux only. It works, but the surface is
> small on purpose.

## Why another monitor

htop is great at "what is using my machine right now." It is less good at "what
happened ten seconds ago." rstop defaults to the second question: every panel is
a line chart against time. If you want the htop-style per-core breakdown, it is
one flag away (`--cpu-cores`, or `-d` for everything), but it is not the default.

## Install

You need a Rust toolchain. The version that ships with apt on Ubuntu is too old
for the dependencies; use [rustup](https://rustup.rs) and a current stable.

```sh
cargo install --path .
```

That drops the `rstop` binary in `~/.cargo/bin` (already on your PATH if you
installed via rustup). After changing the code, re-run with `--force`.

No NVIDIA card, or building somewhere without NVML? Turn the GPU code off:

```sh
cargo install --path . --no-default-features
```

## Usage

```sh
rstop                 # aggregate line charts, ~1.5s refresh
rstop -d              # every panel in its detailed form
rstop --cpu-cores     # detail one panel, leave the rest as charts
rstop --interval 100 --history 1500   # fine-grained, ~2.5 min window
```

| Flag | Default | What it does |
|------|---------|--------------|
| `-i`, `--interval <ms>` | `1500` | Sampling period. Matches htop's default. Floored at 50ms. |
| `--history <n>` | `120` | Samples kept per chart, i.e. how far back the X axis goes. |
| `-d`, `--detailed` | off | Show the detailed view for every panel. |
| `--cpu-cores` | off | Per-core CPU gauges instead of the aggregate chart. |
| `--mem-detailed` | off | RAM/swap gauges and a sparkline. |
| `--gpu-detailed` | off | Per-device blocks: utilization, VRAM, temperature, power. |
| `--temp-detailed` | off | The full per-sensor list. |

The per-panel flags stack with `-d`; `-d --cpu-cores` is just `-d`.

### Keys

| Key | Action |
|-----|--------|
| `q`, `Esc`, `Ctrl-C` | quit |
| `space` | pause/resume sampling |
| `+` / `-` | halve / double the interval (50ms–10s) |

There are no tabs. Everything is on one screen, stacked top to bottom.

## What each panel shows

- **CPU** — total usage over time, with the 1/5/15-minute load average in the
  title. Detailed: a gauge per core with its current clock.
- **Memory** — RAM and swap as two lines on one chart. Detailed: gauges with
  absolute figures plus a history sparkline.
- **GPU** — two charts side by side, utilization and VRAM, one line per device.
  Detailed: a block per GPU with utilization, VRAM, temperature, and power.
- **Thermals** — the hottest CPU sensor over time. Detailed: every hwmon sensor,
  grouped into CPU and other.

## Platform notes

This is Linux-only and leans on Linux interfaces directly.

**Temperatures** come from `/sys/class/hwmon`. Under WSL2 that tree is usually
empty because the Windows host does not expose sensors to the VM, so the panel
will say so rather than invent numbers. On bare metal it works.

**GPU** support is NVIDIA via NVML, behind the `nvidia` feature (on by default).
NVML normally loads `libnvidia-ml.so`, which does not exist on WSL — there you
only get `/usr/lib/wsl/lib/libnvidia-ml.so.1`. rstop tries the plain name first
and then falls back to the versioned soname and the WSL path, so a working
`nvidia-smi` should mean a working GPU panel. To check detection without opening
the UI:

```sh
cargo test gpu_probe -- --nocapture
```

AMD and Intel GPUs are not supported yet.

## Building from source

```sh
cargo build --release      # ./target/release/rstop, stripped + thin LTO
cargo test                 # gpu_probe just prints what NVML finds; it never fails
```

## License

MIT. See [LICENSE](LICENSE).
