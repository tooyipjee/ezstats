#!/bin/bash

# ezstats local build and install script
echo "Installing ezstats..."

# Build the release version without Apple GPU support by default
cargo build --release

# Create installation directory if it doesn't exist
INSTALL_DIR="$HOME/.local/bin"
mkdir -p "$INSTALL_DIR"

# Copy the binary to the installation directory
cp target/release/ezstats "$INSTALL_DIR/"

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

echo "Installation complete. You can now run 'ezstats' from anywhere."
echo "Note: To enable NVIDIA GPU support, use 'cargo install --path . --features nvidia-gpu'"