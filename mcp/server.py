# mcp/server.py
"""
MCP Server for aigpt CLI
"""
from fastmcp import FastMCP
import platform
import os
import sys

mcp = FastMCP("AigptMCP")

@mcp.tool()
def process_text(text: str) -> str:
    """テキストを処理する"""
    return f"Processed: {text}"

@mcp.tool()
def get_system_info() -> dict:
    """システム情報を取得"""
    return {
        "platform": platform.system(),
        "version": platform.version(),
        "python_version": sys.version,
        "current_dir": os.getcwd()
    }

@mcp.tool()
def execute_command(command: str) -> dict:
    """安全なコマンドを実行する"""
    # セキュリティのため、許可されたコマンドのみ実行
    allowed_commands = ["ls", "pwd", "date", "whoami"]
    cmd_parts = command.split()
    
    if not cmd_parts or cmd_parts[0] not in allowed_commands:
        return {
            "error": f"Command '{command}' is not allowed",
            "allowed": allowed_commands
        }
    
    try:
        import subprocess
        result = subprocess.run(
            cmd_parts, 
            capture_output=True, 
            text=True, 
            timeout=10
        )
        return {
            "stdout": result.stdout,
            "stderr": result.stderr,
            "returncode": result.returncode
        }
    except subprocess.TimeoutExpired:
        return {"error": "Command timed out"}
    except Exception as e:
        return {"error": str(e)}

@mcp.tool()
def file_operations(operation: str, filepath: str, content: str = None) -> dict:
    """ファイル操作を行う"""
    try:
        if operation == "read":
            with open(filepath, 'r', encoding='utf-8') as f:
                return {"content": f.read(), "success": True}
        elif operation == "write" and content is not None:
            with open(filepath, 'w', encoding='utf-8') as f:
                f.write(content)
            return {"message": f"File written to {filepath}", "success": True}
        elif operation == "exists":
            return {"exists": os.path.exists(filepath), "success": True}
        else:
            return {"error": "Invalid operation or missing content", "success": False}
    except Exception as e:
        return {"error": str(e), "success": False}

if __name__ == "__main__":
    print("🚀 AigptMCP Server starting...")
    mcp.run()

