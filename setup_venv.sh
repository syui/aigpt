#!/bin/zsh
# Setup Python virtual environment in the new config directory

VENV_DIR="$HOME/.config/syui/ai/gpt/venv"

echo "Creating Python virtual environment at: $VENV_DIR"
python -m venv "$VENV_DIR"

echo "Activating virtual environment..."
source "$VENV_DIR/bin/activate"

echo "Installing aigpt package..."
cd "$(dirname "$0")"
pip install -e .

echo "Setup complete!"
echo "To activate the virtual environment, run:"
echo "source ~/.config/syui/ai/gpt/venv/bin/activate"

if [ -z "`$SHELL -i -c \"alias aigpt\"`" ]; then
	echo 'alias aigpt="$HOME/.config/syui/ai/gpt/venv/bin/aigpt"' >> ${HOME}/.$(basename $SHELL)rc
	exec $SHELL
fi
