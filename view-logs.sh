#!/bin/bash
# View hyprchoosy debug logs

LOG_FILE="/tmp/hyprchoosy/hyprchoosy.log"

if [ ! -f "$LOG_FILE" ]; then
    echo "No log file found at $LOG_FILE"
    echo ""
    echo "Make sure you:"
    echo "  1. Built with debug mode: ./build-debug.sh"
    echo "  2. Installed the debug binary: sudo cp target/release/hyprchoosy /usr/bin/hyprchoosy"
    echo "  3. Opened a URL through hyprchoosy"
    exit 1
fi

if [ "$1" = "-f" ] || [ "$1" = "--follow" ]; then
    echo "Following log file (Ctrl+C to exit)..."
    tail -f "$LOG_FILE"
elif [ "$1" = "-c" ] || [ "$1" = "--clear" ]; then
    rm -f "$LOG_FILE"
    echo "Log file cleared"
else
    echo "=== Latest Hyprchoosy Debug Logs ==="
    echo ""
    cat "$LOG_FILE"
    echo ""
    echo "---"
    echo "To follow logs in real-time: $0 --follow"
    echo "To clear logs: $0 --clear"
fi
