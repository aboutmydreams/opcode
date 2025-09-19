#!/bin/bash

echo "🚀 Starting OpCode API Server..."

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Cargo not found. Please install Rust:"
    echo "   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Check and kill processes using port 3000
PORT=3000
echo "🔍 Checking for processes using port $PORT..."

# Find processes using the port
PIDS=$(lsof -ti:$PORT 2>/dev/null)

if [ ! -z "$PIDS" ]; then
    echo "⚠️  Found processes using port $PORT: $PIDS"
    echo "🔥 Killing processes..."
    
    # Kill the processes
    echo "$PIDS" | xargs kill -9 2>/dev/null
    
    # Wait a moment for processes to terminate
    sleep 1
    
    # Verify port is now free
    if lsof -ti:$PORT &> /dev/null; then
        echo "❌ Failed to free port $PORT. Please manually kill the processes and try again."
        exit 1
    else
        echo "✅ Port $PORT is now free"
    fi
else
    echo "✅ Port $PORT is available"
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