"""Utility functions for documentation management."""

import subprocess
import sys
from pathlib import Path
from typing import List, Optional, Tuple

from rich.console import Console
from rich.progress import Progress, SpinnerColumn, TextColumn

console = Console()


def run_command(
    cmd: List[str],
    cwd: Optional[Path] = None,
    capture_output: bool = True,
    verbose: bool = False,
) -> Tuple[int, str, str]:
    """Run a command and return exit code, stdout, stderr."""
    if verbose:
        console.print(f"[dim]Running: {' '.join(cmd)}[/dim]")
    
    try:
        result = subprocess.run(
            cmd,
            cwd=cwd,
            capture_output=capture_output,
            text=True,
            check=False,
        )
        return result.returncode, result.stdout, result.stderr
    except FileNotFoundError:
        return 1, "", f"Command not found: {cmd[0]}"


def is_git_repository(path: Path) -> bool:
    """Check if path is a git repository."""
    return (path / ".git").exists()


def get_git_status(repo_path: Path) -> Tuple[bool, List[str]]:
    """Get git status for repository."""
    if not is_git_repository(repo_path):
        return False, ["Not a git repository"]
    
    returncode, stdout, stderr = run_command(
        ["git", "status", "--porcelain"],
        cwd=repo_path
    )
    
    if returncode != 0:
        return False, [stderr.strip()]
    
    changes = [line.strip() for line in stdout.splitlines() if line.strip()]
    return len(changes) == 0, changes


def validate_project_name(project_name: str, available_projects: List[str]) -> bool:
    """Validate project name against available projects."""
    return project_name in available_projects


def format_file_size(size_bytes: int) -> str:
    """Format file size in human readable format."""
    for unit in ['B', 'KB', 'MB', 'GB']:
        if size_bytes < 1024.0:
            return f"{size_bytes:.1f}{unit}"
        size_bytes /= 1024.0
    return f"{size_bytes:.1f}TB"


def count_lines(file_path: Path) -> int:
    """Count lines in a file."""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            return sum(1 for _ in f)
    except (OSError, UnicodeDecodeError):
        return 0


def find_project_directories(base_path: Path, projects: List[str]) -> dict:
    """Find project directories relative to base path."""
    project_dirs = {}
    
    # Look for directories matching project names
    for project in projects:
        project_path = base_path / project
        if project_path.exists() and project_path.is_dir():
            project_dirs[project] = project_path
    
    return project_dirs


def check_command_available(command: str) -> bool:
    """Check if a command is available in PATH."""
    try:
        subprocess.run([command, "--version"], 
                     capture_output=True, 
                     check=True)
        return True
    except (subprocess.CalledProcessError, FileNotFoundError):
        return False


def get_platform_info() -> dict:
    """Get platform information."""
    import platform
    
    return {
        "system": platform.system(),
        "release": platform.release(),
        "machine": platform.machine(),
        "python_version": platform.python_version(),
        "python_implementation": platform.python_implementation(),
    }


class ProgressManager:
    """Context manager for rich progress bars."""
    
    def __init__(self, description: str = "Processing..."):
        self.description = description
        self.progress = None
        self.task = None
    
    def __enter__(self):
        self.progress = Progress(
            SpinnerColumn(),
            TextColumn("[progress.description]{task.description}"),
            console=console,
        )
        self.progress.start()
        self.task = self.progress.add_task(self.description, total=None)
        return self
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        if self.progress:
            self.progress.stop()
    
    def update(self, description: str):
        """Update progress description."""
        if self.progress and self.task is not None:
            self.progress.update(self.task, description=description)


def safe_write_file(file_path: Path, content: str, backup: bool = True) -> bool:
    """Safely write content to file with optional backup."""
    try:
        # Create backup if file exists and backup requested
        if backup and file_path.exists():
            backup_path = file_path.with_suffix(file_path.suffix + ".bak")
            backup_path.write_text(file_path.read_text(), encoding="utf-8")
        
        # Ensure parent directory exists
        file_path.parent.mkdir(parents=True, exist_ok=True)
        
        # Write content
        file_path.write_text(content, encoding="utf-8")
        return True
        
    except (OSError, UnicodeError) as e:
        console.print(f"[red]Error writing file {file_path}: {e}[/red]")
        return False


def confirm_action(message: str, default: bool = False) -> bool:
    """Ask user for confirmation."""
    if not sys.stdin.isatty():
        return default
    
    suffix = " [Y/n]: " if default else " [y/N]: "
    response = input(message + suffix).strip().lower()
    
    if not response:
        return default
    
    return response in ('y', 'yes', 'true', '1')