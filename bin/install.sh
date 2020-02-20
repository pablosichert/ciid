#!/bin/bash
function install_package_manager_dependencies() {
    case "$OSTYPE" in
    darwin*) ;;
    linux*)
        sudo apt-get update
        sudo apt-get install -y \
            wget \
            curl \
            git \
            autoconf \
            libtool \
            pkg-config \
            build-essential \
            clang
        ;;
    esac
}

function install_exiftool() {
    # Check if ExifTool is already installed
    if command -v exiftool >/dev/null; then
        return
    fi

    temp=$(mktemp -d)
    trap rm -rf $temp RETURN

    git clone https://github.com/exiftool/exiftool $temp

    pushd $temp

    perl Makefile.PL
    make
    make test
    sudo make install

    popd
}

function install_libraw() {
    # Check if LibRaw is already installed
    case "$OSTYPE" in
    darwin*)
        if [ -f /usr/local/lib/libraw.dylib ]; then
            return
        fi
        ;;
    linux*)
        if [ -f /usr/local/lib/libraw.so ]; then
            return
        fi
        ;;
    esac

    temp=$(mktemp -d)
    trap rm -rf $temp RETURN

    git clone https://github.com/LibRaw/LibRaw $temp

    pushd $temp

    ./mkdist.sh
    ./configure
    sudo make install
    sudo ldconfig

    popd
}

function install_rust_toolchain() {
    curl -s https://sh.rustup.rs | sh -s -- -y
}

function install_ciid() {
    cargo install ciid
}

# Return if this script has been sourced
# https://stackoverflow.com/questions/2683279/how-to-detect-if-a-script-is-being-sourced
if (return 0 2>/dev/null); then
    return
fi

set -euo pipefail

# Run commands without sudo if sudo is not available
if command -v sudo >/dev/null; then
    function sudo() {
        "$@"
    }
fi

echo "Requesting sudo privileges for installation"
sudo true

echo "Installing package manager dependencies"
install_package_manager_dependencies

echo "Installing ExifTool"
install_exiftool

echo "Installing LibRaw"
install_libraw

echo "Installing Rust toolchain"
install_rust_toolchain

echo "Installing ciid"
install_ciid
