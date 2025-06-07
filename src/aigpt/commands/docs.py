"""Documentation management commands for ai.gpt."""

from pathlib import Path
from typing import Dict, List, Optional

import typer
from rich.console import Console
from rich.panel import Panel
from rich.progress import track
from rich.table import Table

from ..docs.config import get_ai_root, load_docs_config
from ..docs.templates import DocumentationTemplateManager
from ..docs.git_utils import ensure_submodules_available
from ..docs.wiki_generator import WikiGenerator
from ..docs.utils import (
    ProgressManager,
    count_lines,
    find_project_directories,
    format_file_size,
    safe_write_file,
    validate_project_name,
)

console = Console()
docs_app = typer.Typer(help="Documentation management for AI ecosystem")


@docs_app.command("generate")
def generate_docs(
    project: str = typer.Option(..., "--project", "-p", help="Project name (os, gpt, card, etc.)"),
    output: Path = typer.Option(Path("./claude.md"), "--output", "-o", help="Output file path"),
    include: str = typer.Option("core,specific", "--include", "-i", help="Components to include"),
    dir: Optional[Path] = typer.Option(None, "--dir", "-d", help="AI ecosystem root directory"),
    auto_pull: bool = typer.Option(True, "--auto-pull/--no-auto-pull", help="Automatically pull missing submodules"),
    ai_gpt_integration: bool = typer.Option(False, "--ai-gpt-integration", help="Enable ai.gpt integration"),
    dry_run: bool = typer.Option(False, "--dry-run", help="Show what would be generated without writing files"),
    verbose: bool = typer.Option(False, "--verbose", "-v", help="Enable verbose output"),
) -> None:
    """Generate project documentation with Claude AI integration.
    
    Creates comprehensive documentation by combining core philosophy,
    architecture, and project-specific content. Supports ai.gpt
    integration for enhanced documentation generation.
    
    Examples:
    
        # Generate basic documentation
        aigpt docs generate --project=os
        
        # Generate with custom directory
        aigpt docs generate --project=gpt --dir ~/ai/ai
        
        # Generate without auto-pulling missing submodules
        aigpt docs generate --project=card --no-auto-pull
        
        # Generate with ai.gpt integration
        aigpt docs generate --project=card --ai-gpt-integration
        
        # Preview without writing
        aigpt docs generate --project=verse --dry-run
    """
    try:
        # Load configuration
        with ProgressManager("Loading configuration...") as progress:
            config = load_docs_config(dir)
            ai_root = get_ai_root(dir)
        
        # Ensure submodules are available
        if auto_pull:
            with ProgressManager("Checking submodules...") as progress:
                success, errors = ensure_submodules_available(ai_root, config, auto_clone=True)
                if not success:
                    console.print(f"[red]Submodule errors: {errors}[/red]")
                    if not typer.confirm("Continue anyway?"):
                        raise typer.Abort()
            
        # Validate project
        available_projects = config.list_projects()
        if not validate_project_name(project, available_projects):
            console.print(f"[red]Error: Project '{project}' not found[/red]")
            console.print(f"Available projects: {', '.join(available_projects)}")
            raise typer.Abort()
        
        # Parse components
        components = [c.strip() for c in include.split(",")]
        
        # Initialize template manager
        template_manager = DocumentationTemplateManager(config)
        
        # Validate components
        valid_components = template_manager.validate_components(components)
        if valid_components != components:
            console.print("[yellow]Some components were invalid and filtered out[/yellow]")
        
        # Show generation info
        project_info = config.get_project_info(project)
        
        info_table = Table(title=f"Documentation Generation: {project}")
        info_table.add_column("Property", style="cyan")
        info_table.add_column("Value", style="green")
        
        info_table.add_row("Project Type", project_info.type if project_info else "Unknown")
        info_table.add_row("Status", project_info.status if project_info else "Unknown")
        info_table.add_row("Output Path", str(output))
        info_table.add_row("Components", ", ".join(valid_components))
        info_table.add_row("AI.GPT Integration", "✓" if ai_gpt_integration else "✗")
        info_table.add_row("Mode", "Dry Run" if dry_run else "Generate")
        
        console.print(info_table)
        console.print()
        
        # AI.GPT integration
        if ai_gpt_integration:
            console.print("[blue]🤖 AI.GPT Integration enabled[/blue]")
            try:
                enhanced_content = _integrate_with_ai_gpt(project, valid_components, verbose)
                if enhanced_content:
                    console.print("[green]✓ AI.GPT enhancement applied[/green]")
                else:
                    console.print("[yellow]⚠ AI.GPT enhancement failed, using standard generation[/yellow]")
            except Exception as e:
                console.print(f"[yellow]⚠ AI.GPT integration error: {e}[/yellow]")
                console.print("[dim]Falling back to standard generation[/dim]")
        
        # Generate documentation
        with ProgressManager("Generating documentation...") as progress:
            content = template_manager.generate_documentation(
                project_name=project,
                components=valid_components,
                output_path=None if dry_run else output,
            )
        
        # Show results
        if dry_run:
            console.print(Panel(
                f"[dim]Preview of generated content ({len(content.splitlines())} lines)[/dim]\n\n" +
                content[:500] + "\n\n[dim]... (truncated)[/dim]",
                title="Dry Run Preview",
                expand=False,
            ))
            console.print(f"[yellow]🔍 Dry run completed. Would write to: {output}[/yellow]")
        else:
            # Write content if not dry run
            if safe_write_file(output, content):
                file_size = output.stat().st_size
                line_count = count_lines(output)
                
                console.print(f"[green]✅ Generated: {output}[/green]")
                console.print(f"[dim]📏 Size: {format_file_size(file_size)} ({line_count} lines)[/dim]")
                
                # Show component breakdown
                if verbose:
                    console.print("\n[blue]📋 Component breakdown:[/blue]")
                    for component in valid_components:
                        component_display = component.replace("_", " ").title()
                        console.print(f"  • {component_display}")
            else:
                console.print("[red]❌ Failed to write documentation[/red]")
                raise typer.Abort()
        
    except Exception as e:
        if verbose:
            console.print_exception()
        else:
            console.print(f"[red]Error: {e}[/red]")
        raise typer.Abort()


@docs_app.command("sync")
def sync_docs(
    project: Optional[str] = typer.Option(None, "--project", "-p", help="Sync specific project"),
    sync_all: bool = typer.Option(False, "--all", "-a", help="Sync all available projects"),
    dry_run: bool = typer.Option(False, "--dry-run", help="Show what would be done without making changes"),
    include: str = typer.Option("core,specific", "--include", "-i", help="Components to include in sync"),
    dir: Optional[Path] = typer.Option(None, "--dir", "-d", help="AI ecosystem root directory"),
    auto_pull: bool = typer.Option(True, "--auto-pull/--no-auto-pull", help="Automatically pull missing submodules"),
    ai_gpt_integration: bool = typer.Option(False, "--ai-gpt-integration", help="Enable ai.gpt integration"),
    verbose: bool = typer.Option(False, "--verbose", "-v", help="Enable verbose output"),
) -> None:
    """Sync documentation across multiple projects.
    
    Synchronizes Claude documentation from the central claude/ directory
    to individual project directories. Supports both single-project and
    bulk synchronization operations.
    
    Examples:
    
        # Sync specific project
        aigpt docs sync --project=os
        
        # Sync all projects with custom directory
        aigpt docs sync --all --dir ~/ai/ai
        
        # Preview sync operations
        aigpt docs sync --all --dry-run
        
        # Sync without auto-pulling submodules
        aigpt docs sync --project=gpt --no-auto-pull
    """
    # Validate arguments
    if not project and not sync_all:
        console.print("[red]Error: Either --project or --all is required[/red]")
        raise typer.Abort()
    
    if project and sync_all:
        console.print("[red]Error: Cannot use both --project and --all[/red]")
        raise typer.Abort()
    
    try:
        # Load configuration
        with ProgressManager("Loading configuration...") as progress:
            config = load_docs_config(dir)
            ai_root = get_ai_root(dir)
        
        # Ensure submodules are available
        if auto_pull:
            with ProgressManager("Checking submodules...") as progress:
                success, errors = ensure_submodules_available(ai_root, config, auto_clone=True)
                if not success:
                    console.print(f"[red]Submodule errors: {errors}[/red]")
                    if not typer.confirm("Continue anyway?"):
                        raise typer.Abort()
        
        available_projects = config.list_projects()
        
        # Validate specific project if provided
        if project and not validate_project_name(project, available_projects):
            console.print(f"[red]Error: Project '{project}' not found[/red]")
            console.print(f"Available projects: {', '.join(available_projects)}")
            raise typer.Abort()
        
        # Determine projects to sync
        if sync_all:
            target_projects = available_projects
        else:
            target_projects = [project]
        
        # Find project directories
        project_dirs = find_project_directories(ai_root, target_projects)
        
        # Show sync information
        sync_table = Table(title="Documentation Sync Plan")
        sync_table.add_column("Project", style="cyan")
        sync_table.add_column("Directory", style="blue")
        sync_table.add_column("Status", style="green")
        sync_table.add_column("Components", style="yellow")
        
        for proj in target_projects:
            if proj in project_dirs:
                target_file = project_dirs[proj] / "claude.md"
                status = "✓ Found" if target_file.parent.exists() else "⚠ Missing"
                sync_table.add_row(proj, str(project_dirs[proj]), status, include)
            else:
                sync_table.add_row(proj, "Not found", "❌ Missing", "N/A")
        
        console.print(sync_table)
        console.print()
        
        if dry_run:
            console.print("[yellow]🔍 DRY RUN MODE - No files will be modified[/yellow]")
        
        # AI.GPT integration setup
        if ai_gpt_integration:
            console.print("[blue]🤖 AI.GPT Integration enabled[/blue]")
            console.print("[dim]Enhanced documentation generation will be applied[/dim]")
            console.print()
        
        # Perform sync operations
        sync_results = []
        
        for proj in track(target_projects, description="Syncing projects..."):
            result = _sync_project(
                proj, 
                project_dirs.get(proj),
                include,
                dry_run,
                ai_gpt_integration,
                verbose
            )
            sync_results.append((proj, result))
        
        # Show results summary
        _show_sync_summary(sync_results, dry_run)
        
    except Exception as e:
        if verbose:
            console.print_exception()
        else:
            console.print(f"[red]Error: {e}[/red]")
        raise typer.Abort()


def _sync_project(
    project_name: str,
    project_dir: Optional[Path],
    include: str,
    dry_run: bool,
    ai_gpt_integration: bool,
    verbose: bool,
) -> Dict:
    """Sync a single project."""
    result = {
        "project": project_name,
        "success": False,
        "message": "",
        "output_file": None,
        "lines": 0,
    }
    
    if not project_dir:
        result["message"] = "Directory not found"
        return result
    
    if not project_dir.exists():
        result["message"] = f"Directory does not exist: {project_dir}"
        return result
    
    target_file = project_dir / "claude.md"
    
    if dry_run:
        result["success"] = True
        result["message"] = f"Would sync to {target_file}"
        result["output_file"] = target_file
        return result
    
    try:
        # Use the generate functionality
        config = load_docs_config()
        template_manager = DocumentationTemplateManager(config)
        
        # Generate documentation
        content = template_manager.generate_documentation(
            project_name=project_name,
            components=[c.strip() for c in include.split(",")],
            output_path=target_file,
        )
        
        result["success"] = True
        result["message"] = "Successfully synced"
        result["output_file"] = target_file
        result["lines"] = len(content.splitlines())
        
        if verbose:
            console.print(f"[dim]✓ Synced {project_name} → {target_file}[/dim]")
        
    except Exception as e:
        result["message"] = f"Sync failed: {str(e)}"
        if verbose:
            console.print(f"[red]✗ Failed {project_name}: {e}[/red]")
    
    return result


def _show_sync_summary(sync_results: List[tuple], dry_run: bool) -> None:
    """Show sync operation summary."""
    success_count = sum(1 for _, result in sync_results if result["success"])
    total_count = len(sync_results)
    error_count = total_count - success_count
    
    # Summary table
    summary_table = Table(title="Sync Summary")
    summary_table.add_column("Metric", style="cyan")
    summary_table.add_column("Value", style="green")
    
    summary_table.add_row("Total Projects", str(total_count))
    summary_table.add_row("Successful", str(success_count))
    summary_table.add_row("Failed", str(error_count))
    
    if not dry_run:
        total_lines = sum(result["lines"] for _, result in sync_results if result["success"])
        summary_table.add_row("Total Lines Generated", str(total_lines))
    
    console.print()
    console.print(summary_table)
    
    # Show errors if any
    if error_count > 0:
        console.print()
        console.print("[red]❌ Failed Projects:[/red]")
        for project_name, result in sync_results:
            if not result["success"]:
                console.print(f"  • {project_name}: {result['message']}")
    
    # Final status
    console.print()
    if dry_run:
        console.print("[yellow]🔍 This was a dry run. To apply changes, run without --dry-run[/yellow]")
    elif error_count == 0:
        console.print("[green]🎉 All projects synced successfully![/green]")
    else:
        console.print(f"[yellow]⚠ Completed with {error_count} error(s)[/yellow]")


def _integrate_with_ai_gpt(project: str, components: List[str], verbose: bool) -> Optional[str]:
    """Integrate with ai.gpt for enhanced documentation generation."""
    try:
        from ..ai_provider import create_ai_provider
        from ..persona import Persona
        from ..config import Config
        
        config = Config()
        ai_root = config.data_dir.parent if config.data_dir else Path.cwd()
        
        # Create AI provider
        provider = config.get("default_provider", "ollama")
        model = config.get(f"providers.{provider}.default_model", "qwen2.5")
        
        ai_provider = create_ai_provider(provider=provider, model=model)
        persona = Persona(config.data_dir)
        
        # Create enhancement prompt
        enhancement_prompt = f"""As an AI documentation expert, enhance the documentation for project '{project}'.

Project type: {project}
Components to include: {', '.join(components)}

Please provide:
1. Improved project description
2. Key features that should be highlighted
3. Usage examples
4. Integration points with other AI ecosystem projects
5. Development workflow recommendations

Focus on making the documentation more comprehensive and user-friendly."""
        
        if verbose:
            console.print("[dim]Generating AI-enhanced content...[/dim]")
        
        # Get AI response
        response, _ = persona.process_interaction(
            "docs_system", 
            enhancement_prompt, 
            ai_provider
        )
        
        if verbose:
            console.print("[green]✓ AI enhancement generated[/green]")
        
        return response
        
    except ImportError as e:
        if verbose:
            console.print(f"[yellow]AI integration unavailable: {e}[/yellow]")
        return None
    except Exception as e:
        if verbose:
            console.print(f"[red]AI integration error: {e}[/red]")
        return None


# Add aliases for convenience
@docs_app.command("gen")
def generate_docs_alias(
    project: str = typer.Option(..., "--project", "-p", help="Project name"),
    output: Path = typer.Option(Path("./claude.md"), "--output", "-o", help="Output file path"),
    include: str = typer.Option("core,specific", "--include", "-i", help="Components to include"),
    ai_gpt_integration: bool = typer.Option(False, "--ai-gpt-integration", help="Enable ai.gpt integration"),
    dry_run: bool = typer.Option(False, "--dry-run", help="Preview mode"),
    verbose: bool = typer.Option(False, "--verbose", "-v", help="Verbose output"),
) -> None:
    """Alias for generate command."""
    generate_docs(project, output, include, ai_gpt_integration, dry_run, verbose)


@docs_app.command("wiki")
def wiki_management(
    action: str = typer.Option("update-auto", "--action", "-a", help="Action to perform (update-auto, build-home, status)"),
    dir: Optional[Path] = typer.Option(None, "--dir", "-d", help="AI ecosystem root directory"),
    auto_pull: bool = typer.Option(True, "--auto-pull/--no-auto-pull", help="Pull latest wiki changes before update"),
    ai_enhance: bool = typer.Option(False, "--ai-enhance", help="Use AI to enhance wiki content"),
    dry_run: bool = typer.Option(False, "--dry-run", help="Show what would be done without making changes"),
    verbose: bool = typer.Option(False, "--verbose", "-v", help="Enable verbose output"),
) -> None:
    """Manage AI wiki generation and updates.
    
    Automatically generates wiki pages from project claude.md files
    and maintains the ai.wiki repository structure.
    
    Actions:
    - update-auto: Generate auto/ directory with project summaries
    - build-home: Rebuild Home.md from all projects
    - status: Show wiki repository status
    
    Examples:
    
        # Update auto-generated content (with auto-pull)
        aigpt docs wiki --action=update-auto
        
        # Update without pulling latest changes
        aigpt docs wiki --action=update-auto --no-auto-pull
        
        # Update with custom directory
        aigpt docs wiki --action=update-auto --dir ~/ai/ai
        
        # Preview what would be generated
        aigpt docs wiki --action=update-auto --dry-run
        
        # Check wiki status
        aigpt docs wiki --action=status
    """
    try:
        # Load configuration
        with ProgressManager("Loading configuration...") as progress:
            config = load_docs_config(dir)
            ai_root = get_ai_root(dir)
        
        # Initialize wiki generator
        wiki_generator = WikiGenerator(config, ai_root)
        
        if not wiki_generator.wiki_root:
            console.print("[red]❌ ai.wiki directory not found[/red]")
            console.print(f"Expected location: {ai_root / 'ai.wiki'}")
            console.print("Please ensure ai.wiki submodule is cloned")
            raise typer.Abort()
        
        # Show wiki information
        if verbose:
            console.print(f"[blue]📁 Wiki root: {wiki_generator.wiki_root}[/blue]")
            console.print(f"[blue]📁 AI root: {ai_root}[/blue]")
        
        if action == "status":
            _show_wiki_status(wiki_generator, ai_root)
        
        elif action == "update-auto":
            if dry_run:
                console.print("[yellow]🔍 DRY RUN MODE - No files will be modified[/yellow]")
                if auto_pull:
                    console.print("[blue]📥 Would pull latest wiki changes[/blue]")
                # Show what would be generated
                project_dirs = find_project_directories(ai_root, config.list_projects())
                console.print(f"[blue]📋 Would generate {len(project_dirs)} project pages:[/blue]")
                for project_name in project_dirs.keys():
                    console.print(f"  • auto/{project_name}.md")
                console.print("  • Home.md")
            else:
                with ProgressManager("Updating wiki auto directory...") as progress:
                    success, updated_files = wiki_generator.update_wiki_auto_directory(
                        auto_pull=auto_pull, 
                        ai_enhance=ai_enhance
                    )
                
                if success:
                    console.print(f"[green]✅ Successfully updated {len(updated_files)} files[/green]")
                    if verbose:
                        for file in updated_files:
                            console.print(f"  • {file}")
                else:
                    console.print("[red]❌ Failed to update wiki[/red]")
                    raise typer.Abort()
        
        elif action == "build-home":
            console.print("[blue]🏠 Building Home.md...[/blue]")
            # This would be implemented to rebuild just Home.md
            console.print("[yellow]⚠ build-home action not yet implemented[/yellow]")
        
        else:
            console.print(f"[red]Unknown action: {action}[/red]")
            console.print("Available actions: update-auto, build-home, status")
            raise typer.Abort()
        
    except Exception as e:
        if verbose:
            console.print_exception()
        else:
            console.print(f"[red]Error: {e}[/red]")
        raise typer.Abort()


def _show_wiki_status(wiki_generator: WikiGenerator, ai_root: Path) -> None:
    """Show wiki repository status."""
    console.print("[blue]📊 AI Wiki Status[/blue]")
    
    # Check wiki directory structure
    wiki_root = wiki_generator.wiki_root
    status_table = Table(title="Wiki Directory Status")
    status_table.add_column("Directory", style="cyan")
    status_table.add_column("Status", style="green")
    status_table.add_column("Files", style="yellow")
    
    directories = ["auto", "claude", "manual"]
    for dir_name in directories:
        dir_path = wiki_root / dir_name
        if dir_path.exists():
            file_count = len(list(dir_path.glob("*.md")))
            status = "✓ Exists"
            files = f"{file_count} files"
        else:
            status = "❌ Missing"
            files = "N/A"
        
        status_table.add_row(dir_name, status, files)
    
    # Check Home.md
    home_path = wiki_root / "Home.md"
    home_status = "✓ Exists" if home_path.exists() else "❌ Missing"
    status_table.add_row("Home.md", home_status, "1 file" if home_path.exists() else "N/A")
    
    console.print(status_table)
    
    # Show project coverage
    config = wiki_generator.config
    project_dirs = find_project_directories(ai_root, config.list_projects())
    auto_dir = wiki_root / "auto"
    
    if auto_dir.exists():
        existing_wiki_files = set(f.stem for f in auto_dir.glob("*.md"))
        available_projects = set(project_dirs.keys())
        
        missing = available_projects - existing_wiki_files
        orphaned = existing_wiki_files - available_projects
        
        console.print(f"\n[blue]📋 Project Coverage:[/blue]")
        console.print(f"  • Total projects: {len(available_projects)}")
        console.print(f"  • Wiki pages: {len(existing_wiki_files)}")
        
        if missing:
            console.print(f"  • Missing wiki pages: {', '.join(missing)}")
        if orphaned:
            console.print(f"  • Orphaned wiki pages: {', '.join(orphaned)}")
        
        if not missing and not orphaned:
            console.print(f"  • ✅ All projects have wiki pages")


@docs_app.command("config")
def docs_config(
    action: str = typer.Option("show", "--action", "-a", help="Action (show, set-dir, clear-dir)"),
    value: Optional[str] = typer.Option(None, "--value", "-v", help="Value to set"),
    verbose: bool = typer.Option(False, "--verbose", help="Enable verbose output"),
) -> None:
    """Manage documentation configuration.
    
    Configure default settings for aigpt docs commands to avoid
    repeating options like --dir every time.
    
    Actions:
    - show: Display current configuration
    - set-dir: Set default AI root directory
    - clear-dir: Clear default AI root directory
    
    Examples:
    
        # Show current config
        aigpt docs config --action=show
        
        # Set default directory
        aigpt docs config --action=set-dir --value=~/ai/ai
        
        # Clear default directory
        aigpt docs config --action=clear-dir
    """
    try:
        from ..config import Config
        config = Config()
        
        if action == "show":
            console.print("[blue]📁 AI Documentation Configuration[/blue]")
            
            # Show current ai_root resolution
            current_ai_root = get_ai_root()
            console.print(f"[green]Current AI root: {current_ai_root}[/green]")
            
            # Show resolution method
            import os
            env_dir = os.getenv("AI_DOCS_DIR")
            config_dir = config.get("docs.ai_root")
            
            resolution_table = Table(title="Directory Resolution")
            resolution_table.add_column("Method", style="cyan")
            resolution_table.add_column("Value", style="yellow")
            resolution_table.add_column("Status", style="green")
            
            resolution_table.add_row("Environment (AI_DOCS_DIR)", env_dir or "Not set", "✓ Active" if env_dir else "Not used")
            resolution_table.add_row("Config file (docs.ai_root)", config_dir or "Not set", "✓ Active" if config_dir and not env_dir else "Not used")
            resolution_table.add_row("Default (relative)", str(Path(__file__).parent.parent.parent.parent.parent), "✓ Active" if not env_dir and not config_dir else "Not used")
            
            console.print(resolution_table)
            
            if verbose:
                console.print(f"\n[dim]Config file: {config.config_file}[/dim]")
        
        elif action == "set-dir":
            if not value:
                console.print("[red]Error: --value is required for set-dir action[/red]")
                raise typer.Abort()
            
            # Expand and validate path
            ai_root_path = Path(value).expanduser().absolute()
            
            if not ai_root_path.exists():
                console.print(f"[yellow]Warning: Directory does not exist: {ai_root_path}[/yellow]")
                if not typer.confirm("Set anyway?"):
                    raise typer.Abort()
            
            # Check if ai.json exists
            ai_json_path = ai_root_path / "ai.json"
            if not ai_json_path.exists():
                console.print(f"[yellow]Warning: ai.json not found at: {ai_json_path}[/yellow]")
                if not typer.confirm("Set anyway?"):
                    raise typer.Abort()
            
            # Save to config
            config.set("docs.ai_root", str(ai_root_path))
            
            console.print(f"[green]✅ Set default AI root directory: {ai_root_path}[/green]")
            console.print("[dim]This will be used when --dir is not specified and AI_DOCS_DIR is not set[/dim]")
        
        elif action == "clear-dir":
            config.delete("docs.ai_root")
            
            console.print("[green]✅ Cleared default AI root directory[/green]")
            console.print("[dim]Will use default relative path when --dir and AI_DOCS_DIR are not set[/dim]")
        
        else:
            console.print(f"[red]Unknown action: {action}[/red]")
            console.print("Available actions: show, set-dir, clear-dir")
            raise typer.Abort()
        
    except Exception as e:
        if verbose:
            console.print_exception()
        else:
            console.print(f"[red]Error: {e}[/red]")
        raise typer.Abort()


# Export the docs app
__all__ = ["docs_app"]