#!/bin/bash
# Build script for Cloudflare Pages
# Only publishes index.html, not the entire repo

set -e

mkdir -p public
cp index.html public/
echo "✓ Copied index.html to public/"
echo "Build complete!"
