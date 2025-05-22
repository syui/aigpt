# cli.py
import sys
import subprocess
from pathlib import Path

SCRIPT_DIR = Path.home() / ".config" / "aigpt" / "mcp" / "scripts"
def run_script(name):
    script_path = SCRIPT_DIR / f"{name}.py"
    if not script_path.exists():
        print(f"❌ スクリプトが見つかりません: {script_path}")
        sys.exit(1)

    args = sys.argv[2:]  # ← "ask" の後の引数を取り出す
    result = subprocess.run(["python", str(script_path)] + args, capture_output=True, text=True)
    print(result.stdout)
    if result.stderr:
        print(result.stderr)
def main():
    if len(sys.argv) < 2:
        print("Usage: mcp <script>")
        return

    command = sys.argv[1]

    if command in {"summarize", "ask", "setup", "server"}:
        run_script(command)
    else:
        print(f"❓ 未知のコマンド: {command}")
