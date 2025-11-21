#!/bin/bash
set -e

# Build release script for hyprchoosy
# This script builds release binaries for multiple targets

TARGETS=(
    "x86_64-unknown-linux-gnu"
    "x86_64-unknown-linux-musl"
    "aarch64-unknown-linux-gnu"
    "aarch64-unknown-linux-musl"
)

echo "Installing cross if not present..."
if ! command -v cross &> /dev/null; then
    cargo install cross --git https://github.com/cross-rs/cross
fi

mkdir -p release

for target in "${TARGETS[@]}"; do
    echo "Building for $target..."
    cross build --release --target "$target"
    
    echo "Creating archive for $target..."
    cd "target/$target/release"
    tar czf "hyprchoosy-$target.tar.gz" hyprchoosy
    mv "hyprchoosy-$target.tar.gz" ../../../release/
    cd ../../..
done

echo "Building DEB package..."
if ! command -v cargo-deb &> /dev/null; then
    cargo install cargo-deb
fi
cargo deb
cp target/debian/*.deb release/

echo "Done! Release artifacts are in ./release/"
ls -lh release/
