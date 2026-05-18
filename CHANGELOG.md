# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-05-17

### Added

- Terminal UI built on ratatui and crossterm, with an async render loop
  decoupled from a blocking metrics poller.
- CPU panel: aggregate usage as a line chart over time, plus load average.
- Memory panel: RAM and swap usage as line charts over time.
- GPU panel: side-by-side utilization and VRAM line charts, one series per
  device. NVIDIA support via NVML, with a fallback that locates
  `libnvidia-ml.so.1` on WSL and on hosts without the CUDA `-dev` package.
- Thermals panel: hottest CPU sensor (read from `/sys/class/hwmon`) over time.
- Detailed views per panel: per-core CPU gauges, RAM/swap gauges, per-GPU
  blocks (utilization, VRAM, temperature, power), and the per-sensor list.
- `--detailed`/`-d` to expand every panel, plus `--cpu-cores`,
  `--mem-detailed`, `--gpu-detailed`, and `--temp-detailed` for individual
  panels.
- `--interval` (default 1500 ms, matching htop) and `--history` flags;
  interval is also adjustable at runtime with `+` and `-`.
- Pause with space, quit with `q`/`Esc`/`Ctrl-C`.
- Optional `nvidia` Cargo feature (enabled by default) so the binary builds
  and runs without NVML present.

[Unreleased]: https://github.com/Shnrqpdr/rstop/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/Shnrqpdr/rstop/releases/tag/v0.1.0
