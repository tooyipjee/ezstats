# Lightweight System Monitor

A simple, lightweight system monitoring tool written in Rust that displays real-time CPU, RAM, and GPU usage statistics in the terminal with a clean visual interface.

## Features

- Real-time CPU usage monitoring (overall and per-core)
- Memory usage statistics
- GPU monitoring support:
  - NVIDIA GPUs via NVML
  - Apple GPUs via Metal framework
- Color-coded metrics with visual bar charts for easy assessment
- Extremely low resource footprint
- Cross-platform compatibility

## Project Structure

```
system-monitor/
├── Cargo.toml
├── src/
│   ├── main.rs         # Main entry point and system monitoring logic
│   ├── gpu.rs          # NVIDIA GPU monitoring module
│   ├── mac_gpu.rs      # Apple GPU monitoring module
│   └── widget.rs       # Terminal UI widget system
```

## Requirements

- Rust (latest stable version recommended)
- For NVIDIA GPU monitoring: NVIDIA GPU with drivers installed
- For Apple GPU monitoring: macOS with Metal-compatible GPU

## Building

### Basic build (CPU and RAM monitoring only)

```bash
cargo build --release
```

### With NVIDIA GPU support

```bash
cargo build --release --features nvidia-gpu
```

### With Apple GPU support (macOS only)

```bash
cargo build --release --features apple-gpu
```

### With both GPU monitoring systems

```bash
cargo build --release --features "nvidia-gpu apple-gpu"
```

## Running

```bash
./target/release/system-monitor
```

## UI Features

The system monitor uses a widget-based UI system that provides:
- Color-coded bar charts (green/yellow/red based on utilization levels)
- Clean sections for CPU, memory, and GPU metrics
- Real-time updates with configurable refresh rate

## Customization

You can modify the refresh rate by changing the millisecond value in the `SystemMonitor::new()` call in `main.rs`. The default is set to 1000ms (1 second).

## GPU Support

### NVIDIA GPUs
- Monitors utilization, temperature, and memory usage
- Requires NVML library (included via the nvml-wrapper crate)

### Apple GPUs
- Monitors basic information and estimated utilization
- Uses the Metal framework (via the metal crate)
- Works with both integrated and discrete Apple GPUs

## Adding support for other GPU vendors

The current implementation supports NVIDIA GPUs via the NVML library and Apple GPUs via the Metal framework. Support for AMD or Intel GPUs would require implementing vendor-specific modules similar to the existing implementations.

## Resource Usage

This application is designed to be extremely lightweight with minimal resource usage, making it suitable for embedded systems and devices with limited compute capabilities.