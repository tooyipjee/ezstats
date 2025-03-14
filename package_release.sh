#!/bin/bash
# Script to package ezstats for release

# Determine platform
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
  PLATFORM="linux"
elif [[ "$OSTYPE" == "darwin"* ]]; then
  PLATFORM="macos"
elif [[ "$OSTYPE" == "msys"* || "$OSTYPE" == "cygwin"* ]]; then
  PLATFORM="windows"
else
  echo "Unsupported platform: $OSTYPE"
  exit 1
fi

echo "Packaging ezstats for $PLATFORM..."

# Clean build
echo "Building default version..."
cargo clean
cargo build --release

# Create directory structure
echo "Creating package structure..."
rm -rf release
mkdir -p release/ezstats/default
mkdir -p release/ezstats/nvidia

# Copy default version
echo "Packaging default version..."
cp target/release/ezstats release/ezstats/default/

# Build NVIDIA version if requested
if [ "$1" = "--with-nvidia" ]; then
  echo "Building NVIDIA GPU version..."
  cargo build --release --features nvidia-gpu
  cp target/release/ezstats release/ezstats/nvidia/ezstats-nvidia
else
  # Copy default to nvidia folder if not building separately
  echo "Skipping NVIDIA GPU build. Using default version..."
  cp target/release/ezstats release/ezstats/nvidia/ezstats-nvidia
fi

# Create archives
echo "Creating archives..."
cd release
tar -czvf ezstats-${PLATFORM}-default.tar.gz ezstats/default
tar -czvf ezstats-${PLATFORM}-nvidia.tar.gz ezstats/nvidia
cd ..

echo "Done! Package files are in the release directory:"
echo "  - release/ezstats-${PLATFORM}-default.tar.gz"
echo "  - release/ezstats-${PLATFORM}-nvidia.tar.gz"
echo ""
echo "Upload these files to your GitHub release."