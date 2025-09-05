#!/bin/bash

# Setup Python environment for AI helpers
echo "🐍 Setting up Python environment for Hive AI helpers..."

# Get current directory
HIVE_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
VENV_DIR="$HIVE_DIR/venv"
PYTHON_REQUIREMENTS="$HIVE_DIR/python/requirements.txt"

# Check if virtual environment exists
if [ ! -d "$VENV_DIR" ]; then
    echo "📦 Creating virtual environment..."
    python3 -m venv "$VENV_DIR"
    if [ $? -ne 0 ]; then
        echo "❌ Failed to create virtual environment"
        exit 1
    fi
else
    echo "✅ Virtual environment already exists"
fi

# Activate virtual environment
echo "🔄 Activating virtual environment..."
source "$VENV_DIR/bin/activate"

# Upgrade pip
echo "⬆️  Upgrading pip..."
pip install --upgrade pip

# Install requirements
echo "📚 Installing Python dependencies..."
if [ -f "$PYTHON_REQUIREMENTS" ]; then
    pip install -r "$PYTHON_REQUIREMENTS"
    if [ $? -ne 0 ]; then
        echo "❌ Failed to install Python dependencies"
        exit 1
    fi
else
    echo "❌ Requirements file not found: $PYTHON_REQUIREMENTS"
    exit 1
fi

# Test the installation
echo "🧪 Testing Python model service..."
echo '{"type": "health", "request_id": "test"}' | python3 "$HIVE_DIR/python/model_service.py" --model-cache-dir ~/.hive/ai_models

if [ $? -eq 0 ]; then
    echo "✅ Python environment setup complete!"
    echo "🚀 Virtual environment ready at: $VENV_DIR"
    echo "🐍 Python executable: $VENV_DIR/bin/python3"
else
    echo "❌ Python model service test failed"
    exit 1
fi