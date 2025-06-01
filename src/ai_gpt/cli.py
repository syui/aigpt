"""CLI interface for ai.gpt using typer"""

import typer
from pathlib import Path
from typing import Optional
from rich.console import Console
from rich.table import Table
from rich.panel import Panel
from datetime import datetime, timedelta

from .persona import Persona
from .transmission import TransmissionController
from .mcp_server import AIGptMcpServer
from .ai_provider import create_ai_provider
from .scheduler import AIScheduler, TaskType
from .config import Config

app = typer.Typer(help="ai.gpt - Autonomous transmission AI with unique personality")
console = Console()

# Configuration
config = Config()
DEFAULT_DATA_DIR = config.data_dir


def get_persona(data_dir: Optional[Path] = None) -> Persona:
    """Get or create persona instance"""
    if data_dir is None:
        data_dir = DEFAULT_DATA_DIR
    
    data_dir.mkdir(parents=True, exist_ok=True)
    return Persona(data_dir)


@app.command()
def chat(
    user_id: str = typer.Argument(..., help="User ID (atproto DID)"),
    message: str = typer.Argument(..., help="Message to send to AI"),
    data_dir: Optional[Path] = typer.Option(None, "--data-dir", "-d", help="Data directory"),
    model: Optional[str] = typer.Option(None, "--model", "-m", help="AI model to use"),
    provider: Optional[str] = typer.Option(None, "--provider", help="AI provider (ollama/openai)")
):
    """Chat with the AI"""
    persona = get_persona(data_dir)
    
    # Create AI provider if specified
    ai_provider = None
    if provider and model:
        try:
            ai_provider = create_ai_provider(provider, model)
            console.print(f"[dim]Using {provider} with model {model}[/dim]\n")
        except Exception as e:
            console.print(f"[yellow]Warning: Could not create AI provider: {e}[/yellow]")
            console.print("[yellow]Falling back to simple responses[/yellow]\n")
    
    # Process interaction
    response, relationship_delta = persona.process_interaction(user_id, message, ai_provider)
    
    # Get updated relationship
    relationship = persona.relationships.get_or_create_relationship(user_id)
    
    # Display response
    console.print(Panel(response, title="AI Response", border_style="cyan"))
    
    # Show relationship status
    status_color = "green" if relationship.transmission_enabled else "yellow"
    if relationship.is_broken:
        status_color = "red"
    
    console.print(f"\n[{status_color}]Relationship Status:[/{status_color}] {relationship.status.value}")
    console.print(f"Score: {relationship.score:.2f} / {relationship.threshold}")
    console.print(f"Transmission: {'✓ Enabled' if relationship.transmission_enabled else '✗ Disabled'}")
    
    if relationship.is_broken:
        console.print("[red]⚠️  This relationship is broken and cannot be repaired.[/red]")


@app.command()
def status(
    user_id: Optional[str] = typer.Argument(None, help="User ID to check status for"),
    data_dir: Optional[Path] = typer.Option(None, "--data-dir", "-d", help="Data directory")
):
    """Check AI status and relationships"""
    persona = get_persona(data_dir)
    state = persona.get_current_state()
    
    # Show AI state
    console.print(Panel(f"[cyan]ai.gpt Status[/cyan]", expand=False))
    console.print(f"Mood: {state.current_mood}")
    console.print(f"Fortune: {state.fortune.fortune_value}/10")
    
    if state.fortune.breakthrough_triggered:
        console.print("[yellow]⚡ Breakthrough triggered![/yellow]")
    
    # Show personality traits
    table = Table(title="Current Personality")
    table.add_column("Trait", style="cyan")
    table.add_column("Value", style="magenta")
    
    for trait, value in state.base_personality.items():
        table.add_row(trait.capitalize(), f"{value:.2f}")
    
    console.print(table)
    
    # Show specific relationship if requested
    if user_id:
        rel = persona.relationships.get_or_create_relationship(user_id)
        console.print(f"\n[cyan]Relationship with {user_id}:[/cyan]")
        console.print(f"Status: {rel.status.value}")
        console.print(f"Score: {rel.score:.2f}")
        console.print(f"Total Interactions: {rel.total_interactions}")
        console.print(f"Transmission Enabled: {rel.transmission_enabled}")


@app.command()
def fortune(
    data_dir: Optional[Path] = typer.Option(None, "--data-dir", "-d", help="Data directory")
):
    """Check today's AI fortune"""
    persona = get_persona(data_dir)
    fortune = persona.fortune_system.get_today_fortune()
    
    # Fortune display
    fortune_bar = "🌟" * fortune.fortune_value + "☆" * (10 - fortune.fortune_value)
    
    console.print(Panel(
        f"{fortune_bar}\n\n"
        f"Today's Fortune: {fortune.fortune_value}/10\n"
        f"Date: {fortune.date}",
        title="AI Fortune",
        border_style="yellow"
    ))
    
    if fortune.consecutive_good > 0:
        console.print(f"[green]Consecutive good days: {fortune.consecutive_good}[/green]")
    if fortune.consecutive_bad > 0:
        console.print(f"[red]Consecutive bad days: {fortune.consecutive_bad}[/red]")
    
    if fortune.breakthrough_triggered:
        console.print("\n[yellow]⚡ BREAKTHROUGH! Special fortune activated![/yellow]")


@app.command()
def transmit(
    data_dir: Optional[Path] = typer.Option(None, "--data-dir", "-d", help="Data directory"),
    dry_run: bool = typer.Option(True, "--dry-run/--execute", help="Dry run or execute")
):
    """Check and execute autonomous transmissions"""
    persona = get_persona(data_dir)
    controller = TransmissionController(persona, persona.data_dir)
    
    eligible = controller.check_transmission_eligibility()
    
    if not eligible:
        console.print("[yellow]No users eligible for transmission.[/yellow]")
        return
    
    console.print(f"[green]Found {len(eligible)} eligible users for transmission:[/green]")
    
    for user_id, rel in eligible.items():
        message = controller.generate_transmission_message(user_id)
        if message:
            console.print(f"\n[cyan]To:[/cyan] {user_id}")
            console.print(f"[cyan]Message:[/cyan] {message}")
            console.print(f"[cyan]Relationship:[/cyan] {rel.status.value} (score: {rel.score:.2f})")
            
            if not dry_run:
                # In real implementation, send via atproto or other channel
                controller.record_transmission(user_id, message, success=True)
                console.print("[green]✓ Transmitted[/green]")
            else:
                console.print("[yellow]→ Would transmit (dry run)[/yellow]")


@app.command()
def maintenance(
    data_dir: Optional[Path] = typer.Option(None, "--data-dir", "-d", help="Data directory")
):
    """Run daily maintenance tasks"""
    persona = get_persona(data_dir)
    
    console.print("[cyan]Running daily maintenance...[/cyan]")
    persona.daily_maintenance()
    console.print("[green]✓ Maintenance completed[/green]")


@app.command()
def relationships(
    data_dir: Optional[Path] = typer.Option(None, "--data-dir", "-d", help="Data directory")
):
    """List all relationships"""
    persona = get_persona(data_dir)
    
    table = Table(title="All Relationships")
    table.add_column("User ID", style="cyan")
    table.add_column("Status", style="magenta")
    table.add_column("Score", style="green")
    table.add_column("Transmission", style="yellow")
    table.add_column("Last Interaction")
    
    for user_id, rel in persona.relationships.relationships.items():
        transmission = "✓" if rel.transmission_enabled else "✗"
        if rel.is_broken:
            transmission = "💔"
        
        last_interaction = rel.last_interaction.strftime("%Y-%m-%d") if rel.last_interaction else "Never"
        
        table.add_row(
            user_id[:16] + "...",
            rel.status.value,
            f"{rel.score:.2f}",
            transmission,
            last_interaction
        )
    
    console.print(table)


@app.command()
def server(
    host: str = typer.Option("localhost", "--host", "-h", help="Server host"),
    port: int = typer.Option(8000, "--port", "-p", help="Server port"),
    data_dir: Optional[Path] = typer.Option(None, "--data-dir", "-d", help="Data directory"),
    model: str = typer.Option("qwen2.5", "--model", "-m", help="AI model to use"),
    provider: str = typer.Option("ollama", "--provider", help="AI provider (ollama/openai)")
):
    """Run MCP server for AI integration"""
    import uvicorn
    
    if data_dir is None:
        data_dir = DEFAULT_DATA_DIR
    
    data_dir.mkdir(parents=True, exist_ok=True)
    
    # Create MCP server
    mcp_server = AIGptMcpServer(data_dir)
    app_instance = mcp_server.get_server().get_app()
    
    console.print(Panel(
        f"[cyan]Starting ai.gpt MCP Server[/cyan]\n\n"
        f"Host: {host}:{port}\n"
        f"Provider: {provider}\n"
        f"Model: {model}\n"
        f"Data: {data_dir}",
        title="MCP Server",
        border_style="green"
    ))
    
    # Store provider info in app state for later use
    app_instance.state.ai_provider = provider
    app_instance.state.ai_model = model
    
    # Run server
    uvicorn.run(app_instance, host=host, port=port)


@app.command()
def schedule(
    action: str = typer.Argument(..., help="Action: add, list, enable, disable, remove, run"),
    task_type: Optional[str] = typer.Argument(None, help="Task type for add action"),
    schedule_expr: Optional[str] = typer.Argument(None, help="Schedule expression (cron or interval)"),
    data_dir: Optional[Path] = typer.Option(None, "--data-dir", "-d", help="Data directory"),
    task_id: Optional[str] = typer.Option(None, "--task-id", "-t", help="Task ID"),
    provider: Optional[str] = typer.Option(None, "--provider", help="AI provider for transmission"),
    model: Optional[str] = typer.Option(None, "--model", "-m", help="AI model for transmission")
):
    """Manage scheduled tasks"""
    persona = get_persona(data_dir)
    scheduler = AIScheduler(persona.data_dir, persona)
    
    if action == "add":
        if not task_type or not schedule_expr:
            console.print("[red]Error: task_type and schedule required for add action[/red]")
            return
        
        # Parse task type
        try:
            task_type_enum = TaskType(task_type)
        except ValueError:
            console.print(f"[red]Invalid task type. Valid types: {', '.join([t.value for t in TaskType])}[/red]")
            return
        
        # Metadata for transmission tasks
        metadata = {}
        if task_type_enum == TaskType.TRANSMISSION_CHECK:
            metadata["provider"] = provider or "ollama"
            metadata["model"] = model or "qwen2.5"
        
        try:
            task = scheduler.add_task(task_type_enum, schedule_expr, task_id, metadata)
            console.print(f"[green]✓ Added task {task.task_id}[/green]")
            console.print(f"Type: {task.task_type.value}")
            console.print(f"Schedule: {task.schedule}")
        except ValueError as e:
            console.print(f"[red]Error: {e}[/red]")
    
    elif action == "list":
        tasks = scheduler.get_tasks()
        if not tasks:
            console.print("[yellow]No scheduled tasks[/yellow]")
            return
        
        table = Table(title="Scheduled Tasks")
        table.add_column("Task ID", style="cyan")
        table.add_column("Type", style="magenta")
        table.add_column("Schedule", style="green")
        table.add_column("Enabled", style="yellow")
        table.add_column("Last Run")
        
        for task in tasks:
            enabled = "✓" if task.enabled else "✗"
            last_run = task.last_run.strftime("%Y-%m-%d %H:%M") if task.last_run else "Never"
            
            table.add_row(
                task.task_id[:20] + "..." if len(task.task_id) > 20 else task.task_id,
                task.task_type.value,
                task.schedule,
                enabled,
                last_run
            )
        
        console.print(table)
    
    elif action == "enable":
        if not task_id:
            console.print("[red]Error: --task-id required for enable action[/red]")
            return
        
        scheduler.enable_task(task_id)
        console.print(f"[green]✓ Enabled task {task_id}[/green]")
    
    elif action == "disable":
        if not task_id:
            console.print("[red]Error: --task-id required for disable action[/red]")
            return
        
        scheduler.disable_task(task_id)
        console.print(f"[yellow]✓ Disabled task {task_id}[/yellow]")
    
    elif action == "remove":
        if not task_id:
            console.print("[red]Error: --task-id required for remove action[/red]")
            return
        
        scheduler.remove_task(task_id)
        console.print(f"[red]✓ Removed task {task_id}[/red]")
    
    elif action == "run":
        console.print("[cyan]Starting scheduler daemon...[/cyan]")
        console.print("Press Ctrl+C to stop\n")
        
        import asyncio
        
        async def run_scheduler():
            scheduler.start()
            try:
                while True:
                    await asyncio.sleep(1)
            except KeyboardInterrupt:
                scheduler.stop()
        
        try:
            asyncio.run(run_scheduler())
        except KeyboardInterrupt:
            console.print("\n[yellow]Scheduler stopped[/yellow]")
    
    else:
        console.print(f"[red]Unknown action: {action}[/red]")
        console.print("Valid actions: add, list, enable, disable, remove, run")


@app.command()
def config(
    action: str = typer.Argument(..., help="Action: get, set, delete, list"),
    key: Optional[str] = typer.Argument(None, help="Configuration key (dot notation)"),
    value: Optional[str] = typer.Argument(None, help="Value to set")
):
    """Manage configuration settings"""
    
    if action == "get":
        if not key:
            console.print("[red]Error: key required for get action[/red]")
            return
        
        val = config.get(key)
        if val is None:
            console.print(f"[yellow]Key '{key}' not found[/yellow]")
        else:
            console.print(f"[cyan]{key}[/cyan] = [green]{val}[/green]")
    
    elif action == "set":
        if not key or value is None:
            console.print("[red]Error: key and value required for set action[/red]")
            return
        
        # Special handling for sensitive keys
        if "password" in key or "api_key" in key:
            console.print(f"[cyan]Setting {key}[/cyan] = [dim]***hidden***[/dim]")
        else:
            console.print(f"[cyan]Setting {key}[/cyan] = [green]{value}[/green]")
        
        config.set(key, value)
        console.print("[green]✓ Configuration saved[/green]")
    
    elif action == "delete":
        if not key:
            console.print("[red]Error: key required for delete action[/red]")
            return
        
        if config.delete(key):
            console.print(f"[green]✓ Deleted {key}[/green]")
        else:
            console.print(f"[yellow]Key '{key}' not found[/yellow]")
    
    elif action == "list":
        keys = config.list_keys(key or "")
        
        if not keys:
            console.print("[yellow]No configuration keys found[/yellow]")
            return
        
        table = Table(title="Configuration Settings")
        table.add_column("Key", style="cyan")
        table.add_column("Value", style="green")
        
        for k in sorted(keys):
            val = config.get(k)
            # Hide sensitive values
            if "password" in k or "api_key" in k:
                display_val = "***hidden***" if val else "not set"
            else:
                display_val = str(val) if val is not None else "not set"
            
            table.add_row(k, display_val)
        
        console.print(table)
    
    else:
        console.print(f"[red]Unknown action: {action}[/red]")
        console.print("Valid actions: get, set, delete, list")


if __name__ == "__main__":
    app()