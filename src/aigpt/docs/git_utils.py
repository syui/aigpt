"""Git utilities for documentation management."""

import subprocess
from pathlib import Path
from typing import List, Optional, Tuple

from rich.console import Console
from rich.progress import track

from .utils import run_command

console = Console()


def check_git_repository(path: Path) -> bool:
    """Check if path is a git repository."""
    return (path / ".git").exists()


def get_submodules_status(repo_path: Path) -> List[dict]:
    """Get status of all submodules."""
    if not check_git_repository(repo_path):
        return []
    
    returncode, stdout, stderr = run_command(
        ["git", "submodule", "status"],
        cwd=repo_path
    )
    
    if returncode != 0:
        return []
    
    submodules = []
    for line in stdout.strip().splitlines():
        if line.strip():
            # Parse git submodule status output
            # Format: " commit_hash path (tag)" or "-commit_hash path" (not initialized)
            parts = line.strip().split()
            if len(parts) >= 2:
                status_char = line[0] if line else ' '
                commit = parts[0].lstrip('-+ ')
                path = parts[1]
                
                submodules.append({
                    "path": path,
                    "commit": commit,
                    "initialized": status_char != '-',
                    "modified": status_char == '+',
                    "status": status_char
                })
    
    return submodules


def init_and_update_submodules(repo_path: Path, specific_paths: Optional[List[str]] = None) -> Tuple[bool, str]:
    """Initialize and update submodules."""
    if not check_git_repository(repo_path):
        return False, "Not a git repository"
    
    try:
        # Initialize submodules
        console.print("[blue]🔧 Initializing submodules...[/blue]")
        returncode, stdout, stderr = run_command(
            ["git", "submodule", "init"],
            cwd=repo_path
        )
        
        if returncode != 0:
            return False, f"Failed to initialize submodules: {stderr}"
        
        # Update submodules
        console.print("[blue]📦 Updating submodules...[/blue]")
        
        if specific_paths:
            # Update specific submodules
            for path in specific_paths:
                console.print(f"[dim]Updating {path}...[/dim]")
                returncode, stdout, stderr = run_command(
                    ["git", "submodule", "update", "--init", "--recursive", path],
                    cwd=repo_path
                )
                
                if returncode != 0:
                    return False, f"Failed to update submodule {path}: {stderr}"
        else:
            # Update all submodules
            returncode, stdout, stderr = run_command(
                ["git", "submodule", "update", "--init", "--recursive"],
                cwd=repo_path
            )
            
            if returncode != 0:
                return False, f"Failed to update submodules: {stderr}"
        
        console.print("[green]✅ Submodules updated successfully[/green]")
        return True, "Submodules updated successfully"
        
    except Exception as e:
        return False, f"Error updating submodules: {str(e)}"


def clone_missing_submodules(repo_path: Path, ai_config) -> Tuple[bool, List[str]]:
    """Clone missing submodules based on ai.json configuration."""
    if not check_git_repository(repo_path):
        return False, ["Not a git repository"]
    
    try:
        # Get current submodules
        current_submodules = get_submodules_status(repo_path)
        current_paths = {sub["path"] for sub in current_submodules}
        
        # Get expected projects from ai.json
        expected_projects = ai_config.list_projects()
        
        # Find missing submodules
        missing_submodules = []
        for project in expected_projects:
            if project not in current_paths:
                # Check if directory exists but is not a submodule
                project_path = repo_path / project
                if not project_path.exists():
                    missing_submodules.append(project)
        
        if not missing_submodules:
            console.print("[green]✅ All submodules are present[/green]")
            return True, []
        
        console.print(f"[yellow]📋 Found {len(missing_submodules)} missing submodules: {missing_submodules}[/yellow]")
        
        # Clone missing submodules
        cloned = []
        for project in track(missing_submodules, description="Cloning missing submodules..."):
            git_url = ai_config.get_project_git_url(project)
            branch = ai_config.get_project_branch(project)
            
            console.print(f"[blue]📦 Adding submodule: {project}[/blue]")
            console.print(f"[dim]URL: {git_url}[/dim]")
            console.print(f"[dim]Branch: {branch}[/dim]")
            
            returncode, stdout, stderr = run_command(
                ["git", "submodule", "add", "-b", branch, git_url, project],
                cwd=repo_path
            )
            
            if returncode == 0:
                cloned.append(project)
                console.print(f"[green]✅ Added {project}[/green]")
            else:
                console.print(f"[red]❌ Failed to add {project}: {stderr}[/red]")
        
        if cloned:
            console.print(f"[green]🎉 Successfully cloned {len(cloned)} submodules[/green]")
        
        return True, cloned
        
    except Exception as e:
        return False, [f"Error cloning submodules: {str(e)}"]


def ensure_submodules_available(repo_path: Path, ai_config, auto_clone: bool = True) -> Tuple[bool, List[str]]:
    """Ensure all submodules are available, optionally cloning missing ones."""
    console.print("[blue]🔍 Checking submodule status...[/blue]")
    
    # Get current submodule status
    submodules = get_submodules_status(repo_path)
    
    # Check for uninitialized submodules
    uninitialized = [sub for sub in submodules if not sub["initialized"]]
    
    if uninitialized:
        console.print(f"[yellow]📦 Found {len(uninitialized)} uninitialized submodules[/yellow]")
        if auto_clone:
            success, message = init_and_update_submodules(
                repo_path, 
                [sub["path"] for sub in uninitialized]
            )
            if not success:
                return False, [message]
        else:
            return False, [f"Uninitialized submodules: {[sub['path'] for sub in uninitialized]}"]
    
    # Check for missing submodules (not in .gitmodules but expected)
    if auto_clone:
        success, cloned = clone_missing_submodules(repo_path, ai_config)
        if not success:
            return False, cloned
        
        # If we cloned new submodules, update all to be safe
        if cloned:
            success, message = init_and_update_submodules(repo_path)
            if not success:
                return False, [message]
    
    return True, []


def get_git_branch(repo_path: Path) -> Optional[str]:
    """Get current git branch."""
    if not check_git_repository(repo_path):
        return None
    
    returncode, stdout, stderr = run_command(
        ["git", "branch", "--show-current"],
        cwd=repo_path
    )
    
    if returncode == 0:
        return stdout.strip()
    return None


def get_git_remote_url(repo_path: Path, remote: str = "origin") -> Optional[str]:
    """Get git remote URL."""
    if not check_git_repository(repo_path):
        return None
    
    returncode, stdout, stderr = run_command(
        ["git", "remote", "get-url", remote],
        cwd=repo_path
    )
    
    if returncode == 0:
        return stdout.strip()
    return None


def pull_repository(repo_path: Path, branch: Optional[str] = None) -> Tuple[bool, str]:
    """Pull latest changes from remote repository."""
    if not check_git_repository(repo_path):
        return False, "Not a git repository"
    
    try:
        # Get current branch if not specified
        if branch is None:
            branch = get_git_branch(repo_path)
            if not branch:
                # If in detached HEAD state, try to switch to main
                console.print("[yellow]⚠️ Repository in detached HEAD state, switching to main...[/yellow]")
                returncode, stdout, stderr = run_command(
                    ["git", "checkout", "main"],
                    cwd=repo_path
                )
                if returncode == 0:
                    branch = "main"
                    console.print("[green]✅ Switched to main branch[/green]")
                else:
                    return False, f"Could not switch to main branch: {stderr}"
        
        console.print(f"[blue]📥 Pulling latest changes for branch: {branch}[/blue]")
        
        # Check if we have uncommitted changes
        returncode, stdout, stderr = run_command(
            ["git", "status", "--porcelain"],
            cwd=repo_path
        )
        
        if returncode == 0 and stdout.strip():
            console.print("[yellow]⚠️ Repository has uncommitted changes[/yellow]")
            console.print("[dim]Consider committing changes before pull[/dim]")
            # Continue anyway, git will handle conflicts
        
        # Fetch latest changes
        console.print("[dim]Fetching from remote...[/dim]")
        returncode, stdout, stderr = run_command(
            ["git", "fetch", "origin"],
            cwd=repo_path
        )
        
        if returncode != 0:
            return False, f"Failed to fetch: {stderr}"
        
        # Pull changes
        returncode, stdout, stderr = run_command(
            ["git", "pull", "origin", branch],
            cwd=repo_path
        )
        
        if returncode != 0:
            # Check if it's a merge conflict
            if "CONFLICT" in stderr or "conflict" in stderr.lower():
                return False, f"Merge conflicts detected: {stderr}"
            return False, f"Failed to pull: {stderr}"
        
        # Check if there were any changes
        if "Already up to date" in stdout or "Already up-to-date" in stdout:
            console.print("[green]✅ Repository already up to date[/green]")
        else:
            console.print("[green]✅ Successfully pulled latest changes[/green]")
            if stdout.strip():
                console.print(f"[dim]{stdout.strip()}[/dim]")
        
        return True, "Successfully pulled latest changes"
        
    except Exception as e:
        return False, f"Error pulling repository: {str(e)}"


def pull_wiki_repository(wiki_path: Path) -> Tuple[bool, str]:
    """Pull latest changes from wiki repository before generating content."""
    if not wiki_path.exists():
        return False, f"Wiki directory not found: {wiki_path}"
    
    if not check_git_repository(wiki_path):
        return False, f"Wiki directory is not a git repository: {wiki_path}"
    
    console.print(f"[blue]📚 Updating wiki repository: {wiki_path.name}[/blue]")
    
    return pull_repository(wiki_path)


def push_repository(repo_path: Path, branch: Optional[str] = None, commit_message: Optional[str] = None) -> Tuple[bool, str]:
    """Commit and push changes to remote repository."""
    if not check_git_repository(repo_path):
        return False, "Not a git repository"
    
    try:
        # Get current branch if not specified
        if branch is None:
            branch = get_git_branch(repo_path)
            if not branch:
                return False, "Could not determine current branch"
        
        # Check if we have any changes to commit
        returncode, stdout, stderr = run_command(
            ["git", "status", "--porcelain"],
            cwd=repo_path
        )
        
        if returncode != 0:
            return False, f"Failed to check git status: {stderr}"
        
        if not stdout.strip():
            console.print("[green]✅ No changes to commit[/green]")
            return True, "No changes to commit"
        
        console.print(f"[blue]📝 Committing changes in: {repo_path.name}[/blue]")
        
        # Add all changes
        returncode, stdout, stderr = run_command(
            ["git", "add", "."],
            cwd=repo_path
        )
        
        if returncode != 0:
            return False, f"Failed to add changes: {stderr}"
        
        # Commit changes
        if commit_message is None:
            commit_message = f"Update wiki content - {Path().cwd().name} documentation sync"
        
        returncode, stdout, stderr = run_command(
            ["git", "commit", "-m", commit_message],
            cwd=repo_path
        )
        
        if returncode != 0:
            # Check if there were no changes to commit
            if "nothing to commit" in stderr or "nothing added to commit" in stderr:
                console.print("[green]✅ No changes to commit[/green]")
                return True, "No changes to commit"
            return False, f"Failed to commit changes: {stderr}"
        
        console.print(f"[blue]📤 Pushing to remote branch: {branch}[/blue]")
        
        # Push to remote
        returncode, stdout, stderr = run_command(
            ["git", "push", "origin", branch],
            cwd=repo_path
        )
        
        if returncode != 0:
            return False, f"Failed to push: {stderr}"
        
        console.print("[green]✅ Successfully pushed changes to remote[/green]")
        if stdout.strip():
            console.print(f"[dim]{stdout.strip()}[/dim]")
        
        return True, "Successfully committed and pushed changes"
        
    except Exception as e:
        return False, f"Error pushing repository: {str(e)}"


def push_wiki_repository(wiki_path: Path, commit_message: Optional[str] = None) -> Tuple[bool, str]:
    """Commit and push changes to wiki repository after generating content."""
    if not wiki_path.exists():
        return False, f"Wiki directory not found: {wiki_path}"
    
    if not check_git_repository(wiki_path):
        return False, f"Wiki directory is not a git repository: {wiki_path}"
    
    console.print(f"[blue]📚 Pushing wiki repository: {wiki_path.name}[/blue]")
    
    if commit_message is None:
        commit_message = "Auto-update wiki content from ai.gpt docs"
    
    return push_repository(wiki_path, branch="main", commit_message=commit_message)