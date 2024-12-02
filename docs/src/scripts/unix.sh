#!/bin/bash

VERSION="acac"

# Function to detect OS and Architecture
detect_os_and_arch() {
    if [[ "$OSTYPE" == linux-gnu* ]]; then
        OS="linux"
        ARCH=$(uname -m)
        case $ARCH in
            x86_64) BINARY_NAME="x86_64-unknown-linux-gnu" ;;
            *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
        esac
    elif [[ "$OSTYPE" == darwin* ]]; then
        OS="macos"
        ARCH=$(uname -m)
        case $ARCH in
            x86_64) BINARY_NAME="x86_64-apple-darwin" ;;
            arm64) BINARY_NAME="aarch64-apple-darwin" ;;
            *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
        esac
    else
        echo "Unsupported operating system."
        exit 1
    fi
}

# Function to download and install the binary
download_and_install() {
    RELEASE_URL="https://api.github.com/repos/ethanhamilthon/leverans/releases/tags/v$VERSION"

    # Get the latest release URL
    LATEST_RELEASE=$(curl -s $RELEASE_URL | grep "browser_download_url.*$BINARY_NAME" | head -n 1 | cut -d '"' -f 4)

    if [[ -z "$LATEST_RELEASE" ]]; then
        echo "No matching binary found for your OS and architecture."
        exit 1
    fi

    # Download the binary
    curl -LO $LATEST_RELEASE

    # Extract filename from URL
    FILENAME=$(basename $LATEST_RELEASE)

    # Move to /usr/local/bin
    sudo mv $FILENAME /usr/local/bin/lev
    sudo chmod +x /usr/local/bin/lev

    echo "Successfully installed lev cli for $BINARY_NAME"
}

# Main script execution
detect_os_and_arch
download_and_install
