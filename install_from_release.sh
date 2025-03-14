#!/bin/bash

# ezstats installer script for pre-built releases
set -e  # Exit on any error

# Configuration
VERSION=${1:-latest}  # Use specified version or 'latest'
INSTALL_DIR="$HOME/.local/bin"
TEMP_DIR="/tmp/ezstats-installer"
USE_NVIDIA=${2:-default}  # default or nvidia

# Determine platform
PLATFORM="unknown"
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
  PLATFORM="linux-x86_64"
elif [[ "$OSTYPE" == "darwin"* ]]; then
  PLATFORM="macos-x86_64"
else
  echo "Unsupported platform: $OSTYPE"
  exit 1
fi

# Clean up any previous temp directory
rm -rf "$TEMP_DIR"
mkdir -p "$TEMP_DIR"

# Function to download and install the latest release
install_latest() {
  echo "Fetching the latest release information..."
  RELEASE_URL=$(curl -s https://api.github.com/repos/GITHUB_USERNAME/ezstats/releases/latest | grep "browser_download_url.*$PLATFORM-$USE_NVIDIA" | cut -d '"' -f 4)
  
  if [ -z "$RELEASE_URL" ]; then
    echo "Error: Could not find a suitable release for $PLATFORM-$USE_NVIDIA."
    exit 1
  fi
  
  echo "Downloading from: $RELEASE_URL"
  curl -L "$RELEASE_URL" -o "$TEMP_DIR/ezstats.tar.gz"
}

# Function to download and install a specific version
install_version() {
  RELEASE_URL="https://github.com/GITHUB_USERNAME/ezstats/releases/download/v$VERSION/ezstats-$PLATFORM-$USE_NVIDIA.tar.gz"
  echo "Downloading version $VERSION from: $RELEASE_URL"
  curl -L "$RELEASE_URL" -o "$TEMP_DIR/ezstats.tar.gz"
}

# Download the appropriate release
echo "Installing ezstats ($PLATFORM, $USE_NVIDIA)..."

if [ "$VERSION" = "latest" ]; then
  install_latest
else
  install_version
fi

# Extract the downloaded archive
echo "Extracting files..."
tar -xzf "$TEMP_DIR/ezstats.tar.gz" -C "$TEMP_DIR"

# Find the binary
BINARY_PATH=""
if [ "$USE_NVIDIA" = "nvidia" ]; then
  BINARY_PATH=$(find "$TEMP_DIR" -name "ezstats-nvidia" -type f)
else
  BINARY_PATH=$(find "$TEMP_DIR" -name "ezstats" -type f | grep -v "nvidia")
fi

if [ -z "$BINARY_PATH" ]; then
  echo "Error: Could not find the ezstats binary in the downloaded package."
  exit 1
fi

# Create installation directory if it doesn't exist
mkdir -p "$INSTALL_DIR"

# Copy the binary to the installation directory
echo "Installing ezstats to $INSTALL_DIR..."
if [ "$USE_NVIDIA" = "nvidia" ]; then
  cp "$BINARY_PATH" "$INSTALL_DIR/ezstats"
else
  cp "$BINARY_PATH" "$INSTALL_DIR/ezstats"
fi
chmod +x "$INSTALL_DIR/ezstats"

# Clean up
echo "Cleaning up temporary files..."
rm -rf "$TEMP_DIR"

# Check if the installation directory is in PATH
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo "Adding $INSTALL_DIR to your PATH..."
    
    # Determine which shell configuration file to use
    SHELL_CONFIG=""
    if [ -f "$HOME/.zshrc" ]; then
        SHELL_CONFIG="$HOME/.zshrc"
    elif [ -f "$HOME/.bashrc" ]; then
        SHELL_CONFIG="$HOME/.bashrc"
    elif [ -f "$HOME/.bash_profile" ]; then
        SHELL_CONFIG="$HOME/.bash_profile"
    fi
    
    if [ -n "$SHELL_CONFIG" ]; then
        echo "export PATH=\"\$PATH:$INSTALL_DIR\"" >> "$SHELL_CONFIG"
        echo "Added $INSTALL_DIR to PATH in $SHELL_CONFIG"
        echo "Please run 'source $SHELL_CONFIG' or restart your terminal to update your PATH."
    else
        echo "Warning: Could not identify shell configuration file."
        echo "Please add the following line to your shell configuration file manually:"
        echo "export PATH=\"\$PATH:$INSTALL_DIR\""
    fi
else
    echo "$INSTALL_DIR is already in your PATH."
fi

echo "Installation complete! You can now run 'ezstats' from anywhere."
echo "Note: You installed the $USE_NVIDIA version of ezstats."