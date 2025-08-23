#!/bin/bash

# Development cleanup script - kills lingering processes and frees ports
# Usage: ./scripts/cleanup-dev.sh

echo "🧹 Cleaning up development environment..."

# Kill any lingering ttyd processes
echo "Stopping ttyd processes..."
pkill -f ttyd 2>/dev/null || echo "  No ttyd processes found"

# Kill any lingering memory-service processes
echo "Stopping memory-service processes..."
pkill -f memory-service 2>/dev/null || echo "  No memory-service processes found"

# Kill any lingering backend-server processes
echo "Stopping backend-server processes..."
pkill -f backend-server 2>/dev/null || echo "  No backend-server processes found"

# Kill any lingering Electron processes
echo "Stopping Electron processes..."
pkill -f electron-poc 2>/dev/null || echo "  No Electron processes found"

# Check for any processes using our port ranges
echo ""
echo "Checking port usage..."

# Check ttyd port range (7100-7999)
for port in {7100..7110}; do
  if lsof -i :$port &>/dev/null; then
    echo "  Port $port is in use, killing process..."
    lsof -ti :$port | xargs kill -9 2>/dev/null
  fi
done

# Check backend port (3457)
if lsof -i :3457 &>/dev/null; then
  echo "  Port 3457 is in use, killing process..."
  lsof -ti :3457 | xargs kill -9 2>/dev/null
fi

echo ""
echo "✅ Cleanup complete!"
echo ""
echo "You can now run 'npm start' safely."