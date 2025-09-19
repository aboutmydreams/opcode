#!/bin/bash

echo "🚀 Starting OpCode API Server..."

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Cargo not found. Please install Rust:"
    echo "   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Build and run the server
cd "$(dirname "$0")"
echo "📦 Building API server..."
cargo build --release

if [ $? -eq 0 ]; then
    echo "✅ Build successful!"
    echo "🌐 Starting server on http://localhost:3000"
    echo "📚 API docs at http://localhost:3000/docs"
    cargo run --release
else
    echo "❌ Build failed!"
    exit 1
fi