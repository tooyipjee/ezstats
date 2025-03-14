#!/bin/bash

# ezstats uninstaller script
set -e  # Exit on any error

# Default installation directory
INSTALL_DIR="$HOME/.local/bin"

echo "Uninstalling ezstats..."

# Check if ezstats is installed
if [ ! -f "$INSTALL_DIR/ezstats" ]; then
    echo "ezstats does not appear to be installed in $INSTALL_DIR."
    
    # Try to find it in PATH
    EZSTATS_PATH=$(which ezstats 2>/dev/null || echo "")
    
    if [ -n "$EZSTATS_PATH" ]; then
        echo "Found ezstats at $EZSTATS_PATH"
        read -p "Do you want to uninstall from this location? (y/n): " CONFIRM
        if [[ $CONFIRM =~ ^[Yy]$ ]]; then
            INSTALL_DIR=$(dirname "$EZSTATS_PATH")
        else
            echo "Uninstallation cancelled."
            exit 1
        fi
    else
        echo "Could not find ezstats in your PATH."
        read -p "Please enter the directory where ezstats is installed: " CUSTOM_DIR
        if [ -f "$CUSTOM_DIR/ezstats" ]; then
            INSTALL_DIR="$CUSTOM_DIR"
        else
            echo "Could not find ezstats at $CUSTOM_DIR. Uninstallation cancelled."
            exit 1
        fi
    fi
fi

# Remove binary
echo "Removing ezstats from $INSTALL_DIR..."
rm -f "$INSTALL_DIR/ezstats"

# Check if directory is now empty and ask if user wants to remove it
if [ -z "$(ls -A "$INSTALL_DIR")" ]; then
    read -p "The directory $INSTALL_DIR is now empty. Remove it? (y/n): " REMOVE_DIR
    if [[ $REMOVE_DIR =~ ^[Yy]$ ]]; then
        rm -rf "$INSTALL_DIR"
        echo "Removed directory $INSTALL_DIR."
    fi
fi

echo "Uninstallation complete."
echo "Note: If you added $INSTALL_DIR to your PATH manually, you may want to remove it from your shell configuration file."

# Try to identify shell config file with PATH modification
SHELL_CONFIG=""
if [ -f "$HOME/.zshrc" ] && grep -q "$INSTALL_DIR" "$HOME/.zshrc"; then
    SHELL_CONFIG="$HOME/.zshrc"
elif [ -f "$HOME/.bashrc" ] && grep -q "$INSTALL_DIR" "$HOME/.bashrc"; then
    SHELL_CONFIG="$HOME/.bashrc"
elif [ -f "$HOME/.bash_profile" ] && grep -q "$INSTALL_DIR" "$HOME/.bashrc"; then
    SHELL_CONFIG="$HOME/.bash_profile"
fi

if [ -n "$SHELL_CONFIG" ]; then
    echo "Found PATH modification in $SHELL_CONFIG"
    echo "You might want to edit this file and remove the line that adds $INSTALL_DIR to your PATH."
fi

echo "Thank you for using ezstats!"