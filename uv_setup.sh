#!/bin/bash

# ai.gpt UV environment setup script
set -e

echo "🚀 Setting up ai.gpt with UV..."

# Check if uv is installed
if ! command -v uv &> /dev/null; then
    echo "❌ UV is not installed. Installing UV..."
    curl -LsSf https://astral.sh/uv/install.sh | sh
    export PATH="$HOME/.cargo/bin:$PATH"
    echo "✅ UV installed successfully"
else
    echo "✅ UV is already installed"
fi

# Navigate to gpt directory
cd "$(dirname "$0")"
echo "📁 Working directory: $(pwd)"

# Create virtual environment if it doesn't exist
if [ ! -d ".venv" ]; then
    echo "🔧 Creating UV virtual environment..."
    uv venv
    echo "✅ Virtual environment created"
else
    echo "✅ Virtual environment already exists"
fi

# Install dependencies
echo "📦 Installing dependencies with UV..."
uv pip install -e .

# Verify installation
echo "🔍 Verifying installation..."
source .venv/bin/activate
which aigpt
aigpt --help

echo ""
echo "🎉 Setup complete!"
echo ""
echo "Usage:"
echo "  source .venv/bin/activate"
echo "  aigpt docs generate --project=os"
echo "  aigpt docs sync --all"
echo "  aigpt docs --help"
echo ""
echo "UV commands:"
echo "  uv pip install <package>    # Install package"
echo "  uv pip list                 # List packages"
echo "  uv run aigpt                # Run without activating"
echo ""