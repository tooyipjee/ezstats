# ezstats

A simple, lightweight system monitoring tool written in Rust that displays real-time CPU, RAM, and GPU usage statistics in the terminal with a clean visual interface.

## Features

- Real-time CPU usage monitoring (overall and per-core)
- Memory usage statistics
- GPU monitoring support:
  - NVIDIA GPUs via NVML
  - Apple GPUs via Metal framework (experimental)
- Color-coded metrics with visual bar charts for easy assessment
- Extremely low resource footprint
- Cross-platform compatibility

## Project Structure

```
ezstats/
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
- For Apple GPU monitoring: macOS with Metal-compatible GPU (experimental)

## Installation

You have two options for installing ezstats:

### Option 1: Install from pre-built binaries (recommended)

#### Linux/macOS

```bash
# Install latest version (default build)
./install_from_release.sh

# Install latest version with NVIDIA support
./install_from_release.sh latest nvidia

# Install specific version
./install_from_release.sh 1.0.0
```

#### Windows (PowerShell)

```powershell
# Install latest version (default build)
.\install_from_release.ps1

# Install latest version with NVIDIA support
.\install_from_release.ps1 -Type nvidia

# Install specific version
.\install_from_release.ps1 -Version 1.0.0
```

### Option 2: Build and install from source

#### Using the installation scripts

```bash
# Linux/macOS
./install.sh

# Windows (PowerShell)
.\install.ps1
```

This builds the default version and installs it to your PATH.

#### Manual installation

##### Installing with Cargo

```bash
cargo install --path .
```

This installs the `ezstats` binary to your Cargo bin directory (usually `~/.cargo/bin/`), which should be in your PATH.

##### Building specific versions

###### Basic build (CPU and RAM monitoring only)

```bash
cargo build --release
```

###### With NVIDIA GPU support

```bash
cargo build --release --features nvidia-gpu
```

###### With Apple GPU support (macOS only, experimental)

```bash
cargo build --release --features apple-gpu
```

###### With both GPU monitoring systems

```bash
cargo build --release --features "nvidia-gpu apple-gpu"
```

## Running

After installation:
```bash
ezstats
```

Or run directly after building:
```bash
./target/release/ezstats
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

### Apple GPUs (Experimental)
- Monitors basic information and estimated utilization
- Uses the Metal framework (via the metal crate)
- Works with both integrated and discrete Apple GPUs

## Resource Usage

This application is designed to be extremely lightweight with minimal resource usage, making it suitable for embedded systems and devices with limited compute capabilities.

## Continuous Integration / Deployment

ezstats includes CI/CD configuration using GitHub Actions, which:

1. Builds the application for Linux, macOS, and Windows
2. Creates both default and NVIDIA-enabled versions
3. Runs tests and verifies the code
4. Automatically creates releases with pre-built binaries when tags are pushed

### Creating a Release

To create a new release:

```bash
# Tag the release
git tag -a v1.0.0 -m "Release v1.0.0"

# Push the tag
git push origin v1.0.0
```

This will trigger the CI/CD pipeline to build the binaries and create a GitHub release with the pre-built packages.