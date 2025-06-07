"""Submodule management commands for ai.gpt."""

from pathlib import Path
from typing import Dict, List, Optional, Tuple
import subprocess
import json

import typer
from rich.console import Console
from rich.panel import Panel
from rich.table import Table

from ..docs.config import get_ai_root, load_docs_config
from ..docs.git_utils import (
    check_git_repository, 
    get_git_branch, 
    get_git_remote_url
)
from ..docs.utils import run_command

console = Console()
submodules_app = typer.Typer(help="Submodule management for AI ecosystem")


def get_submodules_from_gitmodules(repo_path: Path) -> Dict[str, str]:
    """Parse .gitmodules file to get submodule information."""
    gitmodules_path = repo_path / ".gitmodules"
    if not gitmodules_path.exists():
        return {}
    
    submodules = {}
    current_name = None
    
    with open(gitmodules_path, 'r') as f:
        for line in f:
            line = line.strip()
            if line.startswith('[submodule "') and line.endswith('"]'):
                current_name = line[12:-2]  # Extract module name
            elif line.startswith('path = ') and current_name:
                path = line[7:]  # Extract path
                submodules[current_name] = path
                current_name = None
    
    return submodules


def get_branch_for_module(config, module_name: str) -> str:
    """Get target branch for a module from ai.json."""
    project_info = config.get_project_info(module_name)
    if project_info and project_info.branch:
        return project_info.branch
    return "main"  # Default branch


@submodules_app.command("list")
def list_submodules(
    dir: Optional[Path] = typer.Option(None, "--dir", "-d", help="AI ecosystem root directory"),
    verbose: bool = typer.Option(False, "--verbose", "-v", help="Show detailed information")
):
    """List all submodules and their status."""
    try:
        config = load_docs_config(dir)
        ai_root = get_ai_root(dir)
        
        if not check_git_repository(ai_root):
            console.print("[red]Error: Not a git repository[/red]")
            raise typer.Abort()
        
        submodules = get_submodules_from_gitmodules(ai_root)
        
        if not submodules:
            console.print("[yellow]No submodules found[/yellow]")
            return
        
        table = Table(title="Submodules Status")
        table.add_column("Module", style="cyan")
        table.add_column("Path", style="blue")
        table.add_column("Branch", style="green")
        table.add_column("Status", style="yellow")
        
        for module_name, module_path in submodules.items():
            full_path = ai_root / module_path
            
            if not full_path.exists():
                status = "❌ Missing"
                branch = "N/A"
            else:
                branch = get_git_branch(full_path) or "detached"
                
                # Check if submodule is up to date
                returncode, stdout, stderr = run_command(
                    ["git", "submodule", "status", module_path],
                    cwd=ai_root
                )
                
                if returncode == 0 and stdout:
                    status_char = stdout[0] if stdout else ' '
                    if status_char == ' ':
                        status = "✅ Clean"
                    elif status_char == '+':
                        status = "📝 Modified"
                    elif status_char == '-':
                        status = "❌ Not initialized"
                    elif status_char == 'U':
                        status = "⚠️ Conflicts"
                    else:
                        status = "❓ Unknown"
                else:
                    status = "❓ Unknown"
            
            target_branch = get_branch_for_module(config, module_name)
            branch_display = f"{branch}"
            if branch != target_branch:
                branch_display += f" (target: {target_branch})"
            
            table.add_row(module_name, module_path, branch_display, status)
        
        console.print(table)
        
        if verbose:
            console.print(f"\n[dim]Total submodules: {len(submodules)}[/dim]")
            console.print(f"[dim]Repository root: {ai_root}[/dim]")
        
    except Exception as e:
        console.print(f"[red]Error: {e}[/red]")
        raise typer.Abort()


@submodules_app.command("update")
def update_submodules(
    module: Optional[str] = typer.Option(None, "--module", "-m", help="Update specific submodule"),
    all: bool = typer.Option(False, "--all", "-a", help="Update all submodules"),
    dir: Optional[Path] = typer.Option(None, "--dir", "-d", help="AI ecosystem root directory"),
    dry_run: bool = typer.Option(False, "--dry-run", help="Show what would be done"),
    auto_commit: bool = typer.Option(False, "--auto-commit", help="Auto-commit changes"),
    verbose: bool = typer.Option(False, "--verbose", "-v", help="Show detailed output")
):
    """Update submodules to latest commits."""
    if not module and not all:
        console.print("[red]Error: Either --module or --all is required[/red]")
        raise typer.Abort()
    
    if module and all:
        console.print("[red]Error: Cannot use both --module and --all[/red]")
        raise typer.Abort()
    
    try:
        config = load_docs_config(dir)
        ai_root = get_ai_root(dir)
        
        if not check_git_repository(ai_root):
            console.print("[red]Error: Not a git repository[/red]")
            raise typer.Abort()
        
        submodules = get_submodules_from_gitmodules(ai_root)
        
        if not submodules:
            console.print("[yellow]No submodules found[/yellow]")
            return
        
        # Determine which modules to update
        if all:
            modules_to_update = list(submodules.keys())
        else:
            if module not in submodules:
                console.print(f"[red]Error: Submodule '{module}' not found[/red]")
                console.print(f"Available modules: {', '.join(submodules.keys())}")
                raise typer.Abort()
            modules_to_update = [module]
        
        if dry_run:
            console.print("[yellow]🔍 DRY RUN MODE - No changes will be made[/yellow]")
        
        console.print(f"[cyan]Updating {len(modules_to_update)} submodule(s)...[/cyan]")
        
        updated_modules = []
        
        for module_name in modules_to_update:
            module_path = submodules[module_name]
            full_path = ai_root / module_path
            target_branch = get_branch_for_module(config, module_name)
            
            console.print(f"\n[blue]📦 Processing: {module_name}[/blue]")
            
            if not full_path.exists():
                console.print(f"[red]❌ Module directory not found: {module_path}[/red]")
                continue
            
            # Get current commit
            current_commit = None
            returncode, stdout, stderr = run_command(
                ["git", "rev-parse", "HEAD"],
                cwd=full_path
            )
            if returncode == 0:
                current_commit = stdout.strip()[:8]
            
            if dry_run:
                console.print(f"[yellow]🔍 Would update {module_name} to branch {target_branch}[/yellow]")
                if current_commit:
                    console.print(f"[dim]Current: {current_commit}[/dim]")
                continue
            
            # Fetch latest changes
            console.print(f"[dim]Fetching latest changes...[/dim]")
            returncode, stdout, stderr = run_command(
                ["git", "fetch", "origin"],
                cwd=full_path
            )
            
            if returncode != 0:
                console.print(f"[red]❌ Failed to fetch: {stderr}[/red]")
                continue
            
            # Check if update is needed
            returncode, stdout, stderr = run_command(
                ["git", "rev-parse", f"origin/{target_branch}"],
                cwd=full_path
            )
            
            if returncode != 0:
                console.print(f"[red]❌ Branch {target_branch} not found on remote[/red]")
                continue
            
            latest_commit = stdout.strip()[:8]
            
            if current_commit == latest_commit:
                console.print(f"[green]✅ Already up to date[/green]")
                continue
            
            # Switch to target branch and pull
            console.print(f"[dim]Switching to branch {target_branch}...[/dim]")
            returncode, stdout, stderr = run_command(
                ["git", "checkout", target_branch],
                cwd=full_path
            )
            
            if returncode != 0:
                console.print(f"[red]❌ Failed to checkout {target_branch}: {stderr}[/red]")
                continue
            
            returncode, stdout, stderr = run_command(
                ["git", "pull", "origin", target_branch],
                cwd=full_path
            )
            
            if returncode != 0:
                console.print(f"[red]❌ Failed to pull: {stderr}[/red]")
                continue
            
            # Get new commit
            returncode, stdout, stderr = run_command(
                ["git", "rev-parse", "HEAD"],
                cwd=full_path
            )
            new_commit = stdout.strip()[:8] if returncode == 0 else "unknown"
            
            # Stage the submodule update
            returncode, stdout, stderr = run_command(
                ["git", "add", module_path],
                cwd=ai_root
            )
            
            console.print(f"[green]✅ Updated {module_name} ({current_commit} → {new_commit})[/green]")
            updated_modules.append((module_name, current_commit, new_commit))
        
        # Summary
        if updated_modules:
            console.print(f"\n[green]🎉 Successfully updated {len(updated_modules)} module(s)[/green]")
            
            if verbose:
                for module_name, old_commit, new_commit in updated_modules:
                    console.print(f"  • {module_name}: {old_commit} → {new_commit}")
            
            if auto_commit and not dry_run:
                console.print("[blue]💾 Auto-committing changes...[/blue]")
                commit_message = f"Update submodules\n\n📦 Updated modules: {len(updated_modules)}\n"
                for module_name, old_commit, new_commit in updated_modules:
                    commit_message += f"- {module_name}: {old_commit} → {new_commit}\n"
                commit_message += "\n🤖 Generated with ai.gpt submodules update"
                
                returncode, stdout, stderr = run_command(
                    ["git", "commit", "-m", commit_message],
                    cwd=ai_root
                )
                
                if returncode == 0:
                    console.print("[green]✅ Changes committed successfully[/green]")
                else:
                    console.print(f"[red]❌ Failed to commit: {stderr}[/red]")
            elif not dry_run:
                console.print("[yellow]💾 Changes staged but not committed[/yellow]")
                console.print("Run with --auto-commit to commit automatically")
        elif not dry_run:
            console.print("[yellow]No modules needed updating[/yellow]")
        
    except Exception as e:
        console.print(f"[red]Error: {e}[/red]")
        if verbose:
            console.print_exception()
        raise typer.Abort()


# Export the submodules app
__all__ = ["submodules_app"]