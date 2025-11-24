#!/bin/bash
# Build hyprchoosy with debug logging enabled

echo "Building hyprchoosy with debug logging..."
cargo build --release --features debug

if [ $? -eq 0 ]; then
    echo ""
    echo "✓ Build successful!"
    echo "  Binary: target/release/hyprchoosy"
    echo "  Log location: /tmp/hyprchoosy/hyprchoosy.log"
    echo ""
    echo "To install the debug version:"
    echo "  sudo cp target/release/hyprchoosy /usr/bin/hyprchoosy"
    echo ""
    echo "To view logs in real-time:"
    echo "  tail -f /tmp/hyprchoosy/hyprchoosy.log"
else
    echo "✗ Build failed"
    exit 1
fi
