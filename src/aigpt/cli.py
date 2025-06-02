"""CLI interface for ai.gpt using typer"""

import typer
from pathlib import Path
from typing import Optional, Dict, Any
from rich.console import Console
from rich.table import Table
from rich.panel import Panel
from datetime import datetime, timedelta
import subprocess
import shlex
import httpx
import asyncio
from prompt_toolkit import prompt as ptk_prompt
from prompt_toolkit.completion import WordCompleter, Completer, Completion
from prompt_toolkit.history import FileHistory
from prompt_toolkit.auto_suggest import AutoSuggestFromHistory

from .persona import Persona
from .transmission import TransmissionController
from .mcp_server import AIGptMcpServer
from .ai_provider import create_ai_provider
from .scheduler import AIScheduler, TaskType
from .config import Config
from .project_manager import ContinuousDeveloper

app = typer.Typer(help="ai.gpt - Autonomous transmission AI with unique personality")
console = Console()

# Configuration
config = Config()
DEFAULT_DATA_DIR = config.data_dir


class MCPClient:
    """Client for communicating with MCP server using config settings"""
    
    def __init__(self, config: Optional[Config] = None):
        self.config = config or Config()
        self.enabled = self.config.get("mcp.enabled", True)
        self.auto_detect = self.config.get("mcp.auto_detect", True)
        self.servers = self.config.get("mcp.servers", {})
        self.available = False
        self.has_card_tools = False
        
        if self.enabled:
            self._check_availability()
    
    def _check_availability(self):
        """Check if any MCP server is available"""
        self.available = False
        if not self.enabled:
            print(f"🚨 [MCP Client] MCP disabled in config")
            return
            
        print(f"🔍 [MCP Client] Checking availability...")
        print(f"🔍 [MCP Client] Available servers: {list(self.servers.keys())}")
        
        # Check ai.gpt server first (primary)
        ai_gpt_config = self.servers.get("ai_gpt", {})
        if ai_gpt_config:
            base_url = ai_gpt_config.get("base_url", "http://localhost:8001")
            timeout = ai_gpt_config.get("timeout", 5.0)
            
            # Convert timeout to float if it's a string
            if isinstance(timeout, str):
                timeout = float(timeout)
            
            print(f"🔍 [MCP Client] Testing ai_gpt server: {base_url} (timeout: {timeout})")
            try:
                import httpx
                with httpx.Client(timeout=timeout) as client:
                    response = client.get(f"{base_url}/docs")
                    print(f"🔍 [MCP Client] ai_gpt response: {response.status_code}")
                    if response.status_code == 200:
                        self.available = True
                        self.active_server = "ai_gpt"
                        print(f"✅ [MCP Client] ai_gpt server connected successfully")
                        
                        # Check if card tools are available
                        try:
                            card_status = client.get(f"{base_url}/card_system_status")
                            if card_status.status_code == 200:
                                self.has_card_tools = True
                                print(f"✅ [MCP Client] ai.card tools detected and available")
                        except:
                            print(f"🔍 [MCP Client] ai.card tools not available")
                        
                        return
            except Exception as e:
                print(f"🚨 [MCP Client] ai_gpt connection failed: {e}")
        else:
            print(f"🚨 [MCP Client] No ai_gpt config found")
        
        # If auto_detect is enabled, try to find any available server
        if self.auto_detect:
            print(f"🔍 [MCP Client] Auto-detect enabled, trying other servers...")
            for server_name, server_config in self.servers.items():
                base_url = server_config.get("base_url", "")
                timeout = server_config.get("timeout", 5.0)
                
                # Convert timeout to float if it's a string
                if isinstance(timeout, str):
                    timeout = float(timeout)
                
                print(f"🔍 [MCP Client] Testing {server_name}: {base_url} (timeout: {timeout})")
                try:
                    import httpx
                    with httpx.Client(timeout=timeout) as client:
                        response = client.get(f"{base_url}/docs")
                        print(f"🔍 [MCP Client] {server_name} response: {response.status_code}")
                        if response.status_code == 200:
                            self.available = True
                            self.active_server = server_name
                            print(f"✅ [MCP Client] {server_name} server connected successfully")
                            return
                except Exception as e:
                    print(f"🚨 [MCP Client] {server_name} connection failed: {e}")
        
        print(f"🚨 [MCP Client] No MCP servers available")
    
    def _get_url(self, endpoint_name: str) -> Optional[str]:
        """Get full URL for an endpoint"""
        if not self.available or not hasattr(self, 'active_server'):
            print(f"🚨 [MCP Client] Not available or no active server")
            return None
            
        server_config = self.servers.get(self.active_server, {})
        base_url = server_config.get("base_url", "")
        endpoints = server_config.get("endpoints", {})
        endpoint_path = endpoints.get(endpoint_name, "")
        
        print(f"🔍 [MCP Client] Server: {self.active_server}")
        print(f"🔍 [MCP Client] Base URL: {base_url}")
        print(f"🔍 [MCP Client] Endpoints: {list(endpoints.keys())}")
        print(f"🔍 [MCP Client] Looking for: {endpoint_name}")
        print(f"🔍 [MCP Client] Found path: {endpoint_path}")
        
        if base_url and endpoint_path:
            return f"{base_url}{endpoint_path}"
        return None
    
    def _get_timeout(self) -> float:
        """Get timeout for the active server"""
        if not hasattr(self, 'active_server'):
            return 5.0
        server_config = self.servers.get(self.active_server, {})
        timeout = server_config.get("timeout", 5.0)
        
        # Convert timeout to float if it's a string
        if isinstance(timeout, str):
            timeout = float(timeout)
        
        return timeout
    
    async def get_memories(self, limit: int = 5) -> Optional[Dict[str, Any]]:
        """Get memories via MCP"""
        url = self._get_url("get_memories")
        if not url:
            return None
        try:
            async with httpx.AsyncClient(timeout=self._get_timeout()) as client:
                response = await client.get(f"{url}?limit={limit}")
                return response.json() if response.status_code == 200 else None
        except Exception:
            return None
    
    async def search_memories(self, keywords: list) -> Optional[Dict[str, Any]]:
        """Search memories via MCP"""
        url = self._get_url("search_memories")
        if not url:
            return None
        try:
            async with httpx.AsyncClient(timeout=self._get_timeout()) as client:
                response = await client.post(url, json={"keywords": keywords})
                return response.json() if response.status_code == 200 else None
        except Exception:
            return None
    
    async def get_contextual_memories(self, query: str, limit: int = 5) -> Optional[Dict[str, Any]]:
        """Get contextual memories via MCP"""
        url = self._get_url("get_contextual_memories")
        if not url:
            return None
        try:
            async with httpx.AsyncClient(timeout=self._get_timeout()) as client:
                response = await client.get(f"{url}?query={query}&limit={limit}")
                return response.json() if response.status_code == 200 else None
        except Exception:
            return None
    
    async def process_interaction(self, user_id: str, message: str) -> Optional[Dict[str, Any]]:
        """Process interaction via MCP"""
        url = self._get_url("process_interaction")
        if not url:
            return None
        try:
            async with httpx.AsyncClient(timeout=self._get_timeout()) as client:
                response = await client.post(url, json={"user_id": user_id, "message": message})
                return response.json() if response.status_code == 200 else None
        except Exception:
            return None
    
    async def get_relationship(self, user_id: str) -> Optional[Dict[str, Any]]:
        """Get relationship via MCP"""
        url = self._get_url("get_relationship")
        print(f"🔍 [MCP Client] get_relationship URL: {url}")
        if not url:
            print(f"🚨 [MCP Client] No URL found for get_relationship")
            return None
        try:
            async with httpx.AsyncClient(timeout=self._get_timeout()) as client:
                response = await client.get(f"{url}?user_id={user_id}")
                print(f"🔍 [MCP Client] Response status: {response.status_code}")
                if response.status_code == 200:
                    result = response.json()
                    print(f"🔍 [MCP Client] Response data: {result}")
                    return result
                else:
                    print(f"🚨 [MCP Client] HTTP error: {response.status_code}")
                    return None
        except Exception as e:
            print(f"🚨 [MCP Client] Exception: {e}")
            return None
    
    def get_server_info(self) -> Dict[str, Any]:
        """Get information about the active MCP server"""
        if not self.available or not hasattr(self, 'active_server'):
            return {"available": False}
        
        server_config = self.servers.get(self.active_server, {})
        return {
            "available": True,
            "server_name": self.active_server,
            "display_name": server_config.get("name", self.active_server),
            "base_url": server_config.get("base_url", ""),
            "timeout": server_config.get("timeout", 5.0),
            "endpoints": len(server_config.get("endpoints", {})),
            "has_card_tools": self.has_card_tools
        }
    
    # ai.card MCP methods
    async def card_get_user_cards(self, did: str, limit: int = 10) -> Optional[Dict[str, Any]]:
        """Get user's card collection via MCP"""
        if not self.has_card_tools:
            return {"error": "ai.card tools not available"}
        
        url = self._get_url("card_get_user_cards")
        if not url:
            return None
        try:
            async with httpx.AsyncClient(timeout=self._get_timeout()) as client:
                response = await client.get(f"{url}?did={did}&limit={limit}")
                return response.json() if response.status_code == 200 else None
        except Exception as e:
            return {"error": f"Failed to get cards: {str(e)}"}
    
    async def card_draw_card(self, did: str, is_paid: bool = False) -> Optional[Dict[str, Any]]:
        """Draw a card from gacha system via MCP"""
        if not self.has_card_tools:
            return {"error": "ai.card tools not available"}
        
        url = self._get_url("card_draw_card")
        if not url:
            return None
        try:
            async with httpx.AsyncClient(timeout=self._get_timeout()) as client:
                response = await client.post(url, json={"did": did, "is_paid": is_paid})
                return response.json() if response.status_code == 200 else None
        except Exception as e:
            return {"error": f"Failed to draw card: {str(e)}"}
    
    async def card_analyze_collection(self, did: str) -> Optional[Dict[str, Any]]:
        """Analyze card collection via MCP"""
        if not self.has_card_tools:
            return {"error": "ai.card tools not available"}
        
        url = self._get_url("card_analyze_collection")
        if not url:
            return None
        try:
            async with httpx.AsyncClient(timeout=self._get_timeout()) as client:
                response = await client.get(f"{url}?did={did}")
                return response.json() if response.status_code == 200 else None
        except Exception as e:
            return {"error": f"Failed to analyze collection: {str(e)}"}
    
    async def card_get_gacha_stats(self) -> Optional[Dict[str, Any]]:
        """Get gacha statistics via MCP"""
        if not self.has_card_tools:
            return {"error": "ai.card tools not available"}
        
        url = self._get_url("card_get_gacha_stats")
        if not url:
            return None
        try:
            async with httpx.AsyncClient(timeout=self._get_timeout()) as client:
                response = await client.get(url)
                return response.json() if response.status_code == 200 else None
        except Exception as e:
            return {"error": f"Failed to get gacha stats: {str(e)}"}


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
    
    # Get config instance
    config_instance = Config()
    
    # Get defaults from config if not provided
    if not provider:
        provider = config_instance.get("default_provider", "ollama")
    if not model:
        if provider == "ollama":
            model = config_instance.get("providers.ollama.default_model", "qwen2.5")
        else:
            model = config_instance.get("providers.openai.default_model", "gpt-4o-mini")
    
    # Create AI provider with MCP client if needed
    ai_provider = None
    mcp_client = None
    
    try:
        # Create MCP client for OpenAI provider
        if provider == "openai":
            mcp_client = MCPClient(config_instance)
            if mcp_client.available:
                console.print(f"[dim]MCP client connected to {mcp_client.active_server}[/dim]")
        
        ai_provider = create_ai_provider(provider=provider, model=model, mcp_client=mcp_client)
        console.print(f"[dim]Using {provider} with model {model}[/dim]\n")
    except Exception as e:
        console.print(f"[yellow]Warning: Could not create AI provider: {e}[/yellow]")
        console.print("[yellow]Falling back to simple responses[/yellow]\n")
    
    # Process interaction
    response, relationship_delta = persona.process_interaction(user_id, message, ai_provider)
    
    # Get updated relationship
    relationship = persona.relationships.get_or_create_relationship(user_id)
    
    # Display response
    console.print(Panel(response, title="AI Response", border_style="cyan", expand=True, width=None))
    
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
    port: int = typer.Option(8001, "--port", "-p", help="Server port"),
    data_dir: Optional[Path] = typer.Option(None, "--data-dir", "-d", help="Data directory"),
    model: Optional[str] = typer.Option(None, "--model", "-m", help="AI model to use"),
    provider: Optional[str] = typer.Option(None, "--provider", help="AI provider (ollama/openai)")
):
    """Run MCP server for AI integration"""
    import uvicorn
    
    if data_dir is None:
        data_dir = DEFAULT_DATA_DIR
    
    data_dir.mkdir(parents=True, exist_ok=True)
    
    # Get configuration
    config_instance = Config()
    
    # Get defaults from config if not provided
    if not provider:
        provider = config_instance.get("default_provider", "ollama")
    if not model:
        if provider == "ollama":
            model = config_instance.get("providers.ollama.default_model", "qwen3:latest")
        elif provider == "openai":
            model = config_instance.get("providers.openai.default_model", "gpt-4o-mini")
        else:
            model = "qwen3:latest"
    
    # Create MCP server
    mcp_server = AIGptMcpServer(data_dir)
    app_instance = mcp_server.app
    
    # Get endpoint categories and count
    total_routes = len(mcp_server.app.routes)
    mcp_tools = total_routes - 2  # Exclude docs and openapi
    
    # Categorize endpoints
    memory_endpoints = ["get_memories", "search_memories", "get_contextual_memories", "create_summary", "create_core_memory"]
    relationship_endpoints = ["get_relationship", "get_all_relationships", "process_interaction", "check_transmission_eligibility"]
    system_endpoints = ["get_persona_state", "get_fortune", "run_maintenance"]
    shell_endpoints = ["execute_command", "analyze_file", "write_file", "list_files", "read_project_file"]
    remote_endpoints = ["remote_shell", "ai_bot_status", "isolated_python", "isolated_analysis"]
    card_endpoints = ["card_get_user_cards", "card_draw_card", "card_get_card_details", "card_analyze_collection", "card_get_gacha_stats", "card_system_status"]
    
    # Check if ai.card tools are available
    has_card_tools = mcp_server.has_card
    
    # Build endpoint summary
    endpoint_summary = f"""🧠 Memory System: {len(memory_endpoints)} tools
🤝 Relationships: {len(relationship_endpoints)} tools  
⚙️  System State: {len(system_endpoints)} tools
💻 Shell Integration: {len(shell_endpoints)} tools
🔒 Remote Execution: {len(remote_endpoints)} tools"""
    
    if has_card_tools:
        endpoint_summary += f"\n🎴 Card Game System: {len(card_endpoints)} tools"
    
    # Check MCP client connectivity
    mcp_client = MCPClient(config_instance)
    mcp_status = "✅ MCP Client Ready" if mcp_client.available else "⚠️  MCP Client Disabled"
    
    # Add ai.card status if available
    card_status = ""
    if has_card_tools:
        card_status = "\n🎴 ai.card: ./card directory detected"
    
    # Provider configuration check
    provider_status = "✅ Ready"
    if provider == "openai":
        api_key = config_instance.get_api_key("openai")
        if not api_key:
            provider_status = "⚠️  No API Key"
    elif provider == "ollama":
        ollama_host = config_instance.get("providers.ollama.host", "http://localhost:11434")
        provider_status = f"✅ {ollama_host}"
    
    console.print(Panel(
        f"[bold cyan]🚀 ai.gpt MCP Server[/bold cyan]\n\n"
        f"[green]Server Configuration:[/green]\n"
        f"🌐 Address: http://{host}:{port}\n"
        f"📋 API Docs: http://{host}:{port}/docs\n"
        f"💾 Data Directory: {data_dir}\n\n"
        f"[green]AI Provider Configuration:[/green]\n"
        f"🤖 Provider: {provider} {provider_status}\n"
        f"🧩 Model: {model}\n\n"
        f"[green]MCP Tools Available ({mcp_tools} total):[/green]\n"
        f"{endpoint_summary}\n\n"
        f"[green]Integration Status:[/green]\n"
        f"{mcp_status}\n"
        f"🔗 Config: {config_instance.config_file}{card_status}\n\n"
        f"[dim]Press Ctrl+C to stop server[/dim]",
        title="🔧 MCP Server Startup",
        border_style="green",
        expand=True
    ))
    
    # Store provider info in app state for later use
    app_instance.state.ai_provider = provider
    app_instance.state.ai_model = model
    app_instance.state.config = config_instance
    
    # Run server with better logging
    try:
        uvicorn.run(
            app_instance, 
            host=host, 
            port=port,
            log_level="info",
            access_log=False  # Reduce noise
        )
    except KeyboardInterrupt:
        console.print("\n[yellow]🛑 MCP Server stopped[/yellow]")
    except Exception as e:
        console.print(f"\n[red]❌ Server error: {e}[/red]")


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
def shell(
    data_dir: Optional[Path] = typer.Option(None, "--data-dir", "-d", help="Data directory"),
    model: Optional[str] = typer.Option(None, "--model", "-m", help="AI model to use"),
    provider: Optional[str] = typer.Option(None, "--provider", help="AI provider (ollama/openai)")
):
    """Interactive shell mode (ai.shell)"""
    persona = get_persona(data_dir)
    
    # Get defaults from config if not provided
    config_instance = Config()
    if not provider:
        provider = config_instance.get("default_provider", "ollama")
    if not model:
        if provider == "ollama":
            model = config_instance.get("providers.ollama.default_model", "qwen3:latest")
        elif provider == "openai":
            model = config_instance.get("providers.openai.default_model", "gpt-4o-mini")
        else:
            model = "qwen3:latest"  # fallback
    
    # Create AI provider
    ai_provider = None
    try:
        ai_provider = create_ai_provider(provider=provider, model=model)
        console.print(f"[dim]Using {provider} with model {model}[/dim]\n")
    except Exception as e:
        console.print(f"[yellow]Warning: Could not create AI provider: {e}[/yellow]")
        console.print("[yellow]Falling back to simple responses[/yellow]\n")
    
    # Welcome message
    console.print(Panel(
        "[cyan]Welcome to ai.shell[/cyan]\n\n"
        "Interactive AI-powered shell with command execution\n\n"
        "Commands:\n"
        "  help - Show available commands\n"
        "  exit/quit - Exit shell\n"
        "  !<command> - Execute shell command\n"
        "  chat <message> - Chat with AI\n"
        "  status - Show AI status\n"
        "  clear - Clear screen\n\n"
        "Type any message to interact with AI",
        title="ai.shell",
        border_style="green"
    ))
    
    # Custom completer for ai.shell
    class ShellCompleter(Completer):
        def __init__(self):
            # Slash commands (built-in)
            self.slash_commands = [
                '/help', '/exit', '/quit', '/status', '/clear', '/load',
                '/fortune', '/relationships'
            ]
            
            # AI commands
            self.ai_commands = [
                '/analyze', '/generate', '/explain', '/optimize', 
                '/refactor', '/test', '/document'
            ]
            
            # Project commands
            self.project_commands = [
                '/project-status', '/suggest-next', '/continuous'
            ]
            
            # Remote commands
            self.remote_commands = [
                '/remote', '/isolated', '/aibot-status'
            ]
            
            # Shell commands (with ! prefix)
            self.shell_commands = [
                '!ls', '!cd', '!pwd', '!cat', '!echo', '!grep', '!find', 
                '!mkdir', '!rm', '!cp', '!mv', '!git', '!python', '!pip', 
                '!npm', '!node', '!cargo', '!rustc', '!docker', '!kubectl'
            ]
            
            # All commands combined
            self.all_commands = (self.slash_commands + self.ai_commands + 
                               self.project_commands + self.remote_commands + 
                               self.shell_commands)
        
        def get_completions(self, document, complete_event):
            text = document.text_before_cursor
            
            # For slash commands
            if text.startswith('/'):
                for cmd in self.all_commands:
                    if cmd.startswith('/') and cmd.startswith(text):
                        yield Completion(cmd, start_position=-len(text))
            
            # For shell commands (!)
            elif text.startswith('!'):
                for cmd in self.shell_commands:
                    if cmd.startswith(text):
                        yield Completion(cmd, start_position=-len(text))
            
            # For regular text (AI chat)
            else:
                # Common AI prompts
                ai_prompts = [
                    'analyze this file', 'generate code for', 'explain how to',
                    'optimize this', 'refactor the', 'create tests for',
                    'document this code', 'help me with'
                ]
                for prompt in ai_prompts:
                    if prompt.startswith(text.lower()):
                        yield Completion(prompt, start_position=-len(text))
    
    completer = ShellCompleter()
    
    # History file
    actual_data_dir = data_dir if data_dir else DEFAULT_DATA_DIR
    history_file = actual_data_dir / "shell_history.txt"
    history = FileHistory(str(history_file))
    
    # Main shell loop
    current_user = "shell_user"  # Default user for shell sessions
    
    while True:
        try:
            # Get input with completion
            user_input = ptk_prompt(
                "ai.shell> ",
                completer=completer,
                history=history,
                auto_suggest=AutoSuggestFromHistory()
            ).strip()
            
            if not user_input:
                continue
            
            # Exit commands
            if user_input.lower() in ['exit', 'quit', '/exit', '/quit']:
                console.print("[cyan]Goodbye![/cyan]")
                break
            
            # Help command
            elif user_input.lower() in ['help', '/help', '/']:
                console.print(Panel(
                    "[cyan]ai.shell Commands:[/cyan]\n\n"
                    "  /help, /          - Show this help message\n"
                    "  /exit, /quit      - Exit the shell\n"
                    "  !<command>        - Execute a shell command (!ls, !git status)\n"
                    "  /status           - Show AI status\n"
                    "  /fortune          - Check AI fortune\n"
                    "  /relationships    - List all relationships\n"
                    "  /clear            - Clear the screen\n"
                    "  /load             - Load aishell.md project file\n\n"
                    "[cyan]AI Commands:[/cyan]\n"
                    "  /analyze <file>   - Analyze a file with AI\n"
                    "  /generate <desc>  - Generate code from description\n"
                    "  /explain <topic>  - Get AI explanation\n\n"
                    "[cyan]Remote Commands (ai.bot):[/cyan]\n"
                    "  /remote <command> - Execute command in isolated container\n"
                    "  /isolated <code>  - Run Python code in isolated environment\n"
                    "  /aibot-status     - Check ai.bot server status\n\n"
                    "[cyan]Project Commands (Claude Code-like):[/cyan]\n"
                    "  /project-status   - Analyze current project structure\n"
                    "  /suggest-next     - AI suggests next development steps\n"
                    "  /continuous       - Enable continuous development mode\n\n"
                    "[cyan]Tab Completion:[/cyan]\n"
                    "  /[Tab]            - Show all slash commands\n"
                    "  ![Tab]            - Show all shell commands\n"
                    "  <text>[Tab]       - AI prompt suggestions\n\n"
                    "Type any message to chat with AI",
                    title="Help",
                    border_style="yellow"
                ))
            
            # Clear command
            elif user_input.lower() in ['clear', '/clear']:
                console.clear()
            
            # Shell command execution
            elif user_input.startswith('!'):
                cmd = user_input[1:].strip()
                if cmd:
                    try:
                        # Execute command
                        result = subprocess.run(
                            shlex.split(cmd),
                            capture_output=True,
                            text=True,
                            shell=False
                        )
                        
                        if result.stdout:
                            console.print(result.stdout.rstrip())
                        if result.stderr:
                            console.print(f"[red]{result.stderr.rstrip()}[/red]")
                        
                        if result.returncode != 0:
                            console.print(f"[red]Command exited with code {result.returncode}[/red]")
                    except FileNotFoundError:
                        console.print(f"[red]Command not found: {cmd.split()[0]}[/red]")
                    except Exception as e:
                        console.print(f"[red]Error executing command: {e}[/red]")
            
            # Status command
            elif user_input.lower() in ['status', '/status']:
                state = persona.get_current_state()
                console.print(f"\nMood: {state.current_mood}")
                console.print(f"Fortune: {state.fortune.fortune_value}/10")
                
                rel = persona.relationships.get_or_create_relationship(current_user)
                console.print(f"\nRelationship Status: {rel.status.value}")
                console.print(f"Score: {rel.score:.2f} / {rel.threshold}")
            
            # Fortune command
            elif user_input.lower() in ['fortune', '/fortune']:
                fortune = persona.fortune_system.get_today_fortune()
                fortune_bar = "🌟" * fortune.fortune_value + "☆" * (10 - fortune.fortune_value)
                console.print(f"\n{fortune_bar}")
                console.print(f"Today's Fortune: {fortune.fortune_value}/10")
            
            # Relationships command
            elif user_input.lower() in ['relationships', '/relationships']:
                if persona.relationships.relationships:
                    console.print("\n[cyan]Relationships:[/cyan]")
                    for user_id, rel in persona.relationships.relationships.items():
                        console.print(f"  {user_id[:16]}... - {rel.status.value} ({rel.score:.2f})")
                else:
                    console.print("[yellow]No relationships yet[/yellow]")
            
            # Load aishell.md command
            elif user_input.lower() in ['load', '/load', 'load aishell.md', 'project']:
                # Try to find and load aishell.md
                search_paths = [
                    Path.cwd() / "aishell.md",
                    Path.cwd() / "docs" / "aishell.md",
                    actual_data_dir.parent / "aishell.md",
                    Path.cwd() / "claude.md",  # Also check for claude.md
                ]
                
                loaded = False
                for path in search_paths:
                    if path.exists():
                        console.print(f"[cyan]Loading project file: {path}[/cyan]")
                        with open(path, 'r', encoding='utf-8') as f:
                            content = f.read()
                        
                        # Process with AI to understand project
                        load_prompt = f"I've loaded the project specification. Please analyze it and understand the project goals:\n\n{content[:3000]}"
                        response, _ = persona.process_interaction(current_user, load_prompt, ai_provider)
                        console.print(f"\n[green]Project loaded successfully![/green]")
                        console.print(f"[cyan]AI Understanding:[/cyan]\n{response}")
                        loaded = True
                        break
                
                if not loaded:
                    console.print("[yellow]No aishell.md or claude.md found in project.[/yellow]")
                    console.print("Create aishell.md to define project goals and AI instructions.")
            
            # AI-powered commands
            elif user_input.lower().startswith(('analyze ', '/analyze ')):
                # Analyze file or code with project context
                target = user_input.split(' ', 1)[1].strip() if ' ' in user_input else ''
                if target and os.path.exists(target):
                    console.print(f"[cyan]Analyzing {target} with project context...[/cyan]")
                    try:
                        developer = ContinuousDeveloper(Path.cwd(), ai_provider)
                        analysis = developer.analyze_file(target)
                        console.print(f"\n[cyan]Analysis:[/cyan]\n{analysis}")
                    except Exception as e:
                        # Fallback to simple analysis
                        with open(target, 'r') as f:
                            content = f.read()
                        analysis_prompt = f"Analyze this file and provide insights:\n\n{content[:2000]}"
                        response, _ = persona.process_interaction(current_user, analysis_prompt, ai_provider)
                        console.print(f"\n[cyan]Analysis:[/cyan]\n{response}")
                else:
                    console.print(f"[red]Usage: /analyze <file_path>[/red]")
            
            elif user_input.lower().startswith(('generate ', '/generate ')):
                # Generate code with project context
                gen_prompt = user_input.split(' ', 1)[1].strip() if ' ' in user_input else ''
                if gen_prompt:
                    console.print("[cyan]Generating code with project context...[/cyan]")
                    try:
                        developer = ContinuousDeveloper(Path.cwd(), ai_provider)
                        generated_code = developer.generate_code(gen_prompt)
                        console.print(f"\n[cyan]Generated Code:[/cyan]\n{generated_code}")
                    except Exception as e:
                        # Fallback to simple generation
                        full_prompt = f"Generate code for: {gen_prompt}. Provide clean, well-commented code."
                        response, _ = persona.process_interaction(current_user, full_prompt, ai_provider)
                        console.print(f"\n[cyan]Generated Code:[/cyan]\n{response}")
                else:
                    console.print(f"[red]Usage: /generate <description>[/red]")
            
            elif user_input.lower().startswith(('explain ', '/explain ')):
                # Explain code or concept
                topic = user_input[8:].strip()
                if topic:
                    console.print(f"[cyan]Explaining {topic}...[/cyan]")
                    full_prompt = f"Explain this in detail: {topic}"
                    response, _ = persona.process_interaction(current_user, full_prompt, ai_provider)
                    console.print(f"\n[cyan]Explanation:[/cyan]\n{response}")
            
            # Remote execution commands (ai.bot integration)
            elif user_input.lower().startswith('remote '):
                # Execute command in ai.bot isolated container
                command = user_input[7:].strip()
                if command:
                    console.print(f"[cyan]Executing remotely:[/cyan] {command}")
                    try:
                        import httpx
                        import asyncio
                        
                        async def execute_remote():
                            async with httpx.AsyncClient(timeout=30.0) as client:
                                response = await client.post(
                                    "http://localhost:8080/sh",
                                    json={"command": command},
                                    headers={"Content-Type": "application/json"}
                                )
                                return response
                        
                        response = asyncio.run(execute_remote())
                        
                        if response.status_code == 200:
                            result = response.json()
                            console.print(f"[green]Output:[/green]\n{result.get('output', '')}")
                            if result.get('error'):
                                console.print(f"[red]Error:[/red] {result.get('error')}")
                            console.print(f"[dim]Exit code: {result.get('exit_code', 0)} | Execution time: {result.get('execution_time', 'N/A')}[/dim]")
                        else:
                            console.print(f"[red]ai.bot error: HTTP {response.status_code}[/red]")
                    except Exception as e:
                        console.print(f"[red]Failed to connect to ai.bot: {e}[/red]")
            
            elif user_input.lower().startswith('isolated '):
                # Execute Python code in isolated environment
                code = user_input[9:].strip()
                if code:
                    console.print(f"[cyan]Running Python code in isolated container...[/cyan]")
                    try:
                        import httpx
                        import asyncio
                        
                        async def execute_python():
                            python_command = f'python3 -c "{code.replace('"', '\\"')}"'
                            async with httpx.AsyncClient(timeout=30.0) as client:
                                response = await client.post(
                                    "http://localhost:8080/sh",
                                    json={"command": python_command},
                                    headers={"Content-Type": "application/json"}
                                )
                                return response
                        
                        response = asyncio.run(execute_python())
                        
                        if response.status_code == 200:
                            result = response.json()
                            console.print(f"[green]Python Output:[/green]\n{result.get('output', '')}")
                            if result.get('error'):
                                console.print(f"[red]Error:[/red] {result.get('error')}")
                        else:
                            console.print(f"[red]ai.bot error: HTTP {response.status_code}[/red]")
                    except Exception as e:
                        console.print(f"[red]Failed to execute Python code: {e}[/red]")
            
            elif user_input.lower() == 'aibot-status':
                # Check ai.bot server status
                console.print("[cyan]Checking ai.bot server status...[/cyan]")
                try:
                    import httpx
                    import asyncio
                    
                    async def check_status():
                        async with httpx.AsyncClient(timeout=10.0) as client:
                            response = await client.get("http://localhost:8080/status")
                            return response
                    
                    response = asyncio.run(check_status())
                    
                    if response.status_code == 200:
                        result = response.json()
                        console.print(f"[green]ai.bot is online![/green]")
                        console.print(f"Server info: {result}")
                    else:
                        console.print(f"[yellow]ai.bot responded with status {response.status_code}[/yellow]")
                except Exception as e:
                    console.print(f"[red]ai.bot is offline: {e}[/red]")
                    console.print("[dim]Make sure ai.bot is running on localhost:8080[/dim]")
            
            # Project management commands (Claude Code-like)
            elif user_input.lower() == 'project-status':
                # プロジェクト構造分析
                console.print("[cyan]Analyzing project structure...[/cyan]")
                try:
                    developer = ContinuousDeveloper(Path.cwd(), ai_provider)
                    analysis = developer.analyze_project_structure()
                    changes = developer.project_state.detect_changes()
                    
                    console.print(f"[green]Project Analysis:[/green]")
                    console.print(f"Language: {analysis['language']}")
                    console.print(f"Framework: {analysis['framework']}")
                    console.print(f"Structure: {analysis['structure']}")
                    console.print(f"Dependencies: {analysis['dependencies']}")
                    console.print(f"Code Patterns: {analysis['patterns']}")
                    
                    if changes:
                        console.print(f"\n[yellow]Recent Changes:[/yellow]")
                        for file_path, change_type in changes.items():
                            console.print(f"  {change_type}: {file_path}")
                    else:
                        console.print(f"\n[dim]No recent changes detected[/dim]")
                        
                except Exception as e:
                    console.print(f"[red]Error analyzing project: {e}[/red]")
            
            elif user_input.lower() == 'suggest-next':
                # 次のステップを提案
                console.print("[cyan]AI is analyzing project and suggesting next steps...[/cyan]")
                try:
                    developer = ContinuousDeveloper(Path.cwd(), ai_provider)
                    suggestions = developer.suggest_next_steps()
                    
                    console.print(f"[green]Suggested Next Steps:[/green]")
                    for i, suggestion in enumerate(suggestions, 1):
                        console.print(f"  {i}. {suggestion}")
                        
                except Exception as e:
                    console.print(f"[red]Error generating suggestions: {e}[/red]")
            
            elif user_input.lower().startswith('continuous'):
                # 継続開発モード
                console.print("[cyan]Enabling continuous development mode...[/cyan]")
                console.print("[yellow]Continuous mode is experimental. Type 'exit-continuous' to exit.[/yellow]")
                
                try:
                    developer = ContinuousDeveloper(Path.cwd(), ai_provider)
                    context = developer.load_project_context()
                    
                    console.print(f"[green]Project context loaded:[/green]")
                    console.print(f"Context: {len(context)} characters")
                    
                    # Add to session memory for continuous context
                    persona.process_interaction(current_user, f"Continuous development mode started for project: {context[:500]}", ai_provider)
                    console.print("[dim]Project context added to AI memory for continuous development.[/dim]")
                    
                except Exception as e:
                    console.print(f"[red]Error starting continuous mode: {e}[/red]")
            
            # Chat command or direct message
            else:
                # Remove 'chat' prefix if present
                if user_input.lower().startswith('chat '):
                    message = user_input[5:].strip()
                else:
                    message = user_input
                
                if message:
                    # Process interaction with AI
                    response, relationship_delta = persona.process_interaction(
                        current_user, message, ai_provider
                    )
                    
                    # Display response
                    console.print(f"\n[cyan]AI:[/cyan] {response}")
                    
                    # Show relationship change if significant
                    if abs(relationship_delta) >= 0.1:
                        if relationship_delta > 0:
                            console.print(f"[green](+{relationship_delta:.2f} relationship)[/green]")
                        else:
                            console.print(f"[red]({relationship_delta:.2f} relationship)[/red]")
        
        except KeyboardInterrupt:
            console.print("\n[yellow]Use 'exit' or 'quit' to leave the shell[/yellow]")
        except EOFError:
            console.print("\n[cyan]Goodbye![/cyan]")
            break
        except Exception as e:
            console.print(f"[red]Error: {e}[/red]")


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
        
        config_instance = Config()
        val = config_instance.get(key)
        if val is None:
            console.print(f"[yellow]Key '{key}' not found[/yellow]")
        else:
            console.print(f"[cyan]{key}[/cyan] = [green]{val}[/green]")
    
    elif action == "set":
        if not key or value is None:
            console.print("[red]Error: key and value required for set action[/red]")
            return
        
        config_instance = Config()
        # Special handling for sensitive keys
        if "password" in key or "api_key" in key:
            console.print(f"[cyan]Setting {key}[/cyan] = [dim]***hidden***[/dim]")
        else:
            console.print(f"[cyan]Setting {key}[/cyan] = [green]{value}[/green]")
        
        config_instance.set(key, value)
        console.print("[green]✓ Configuration saved[/green]")
    
    elif action == "delete":
        if not key:
            console.print("[red]Error: key required for delete action[/red]")
            return
        
        config_instance = Config()
        if config_instance.delete(key):
            console.print(f"[green]✓ Deleted {key}[/green]")
        else:
            console.print(f"[yellow]Key '{key}' not found[/yellow]")
    
    elif action == "list":
        config_instance = Config()
        keys = config_instance.list_keys(key or "")
        
        if not keys:
            console.print("[yellow]No configuration keys found[/yellow]")
            return
        
        table = Table(title="Configuration Settings")
        table.add_column("Key", style="cyan")
        table.add_column("Value", style="green")
        
        for k in sorted(keys):
            val = config_instance.get(k)
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


@app.command()
def import_chatgpt(
    file_path: Path = typer.Argument(..., help="Path to ChatGPT export JSON file"),
    user_id: str = typer.Option("chatgpt_user", "--user-id", "-u", help="User ID for imported conversations"),
    data_dir: Optional[Path] = typer.Option(None, "--data-dir", "-d", help="Data directory")
):
    """Import ChatGPT conversation data into ai.gpt memory system"""
    from .chatgpt_importer import ChatGPTImporter
    
    if data_dir is None:
        data_dir = DEFAULT_DATA_DIR
    
    data_dir.mkdir(parents=True, exist_ok=True)
    
    if not file_path.exists():
        console.print(f"[red]Error: File not found: {file_path}[/red]")
        raise typer.Exit(1)
    
    console.print(f"[cyan]Importing ChatGPT data from {file_path}[/cyan]")
    console.print(f"User ID: {user_id}")
    console.print(f"Data directory: {data_dir}")
    
    try:
        importer = ChatGPTImporter(data_dir)
        stats = importer.import_from_file(file_path, user_id)
        
        # Display results
        table = Table(title="Import Results")
        table.add_column("Metric", style="cyan")
        table.add_column("Count", style="green")
        
        table.add_row("Conversations imported", str(stats["conversations_imported"]))
        table.add_row("Total messages", str(stats["messages_imported"]))
        table.add_row("User messages", str(stats["user_messages"]))
        table.add_row("Assistant messages", str(stats["assistant_messages"]))
        table.add_row("Skipped messages", str(stats["skipped_messages"]))
        
        console.print(table)
        console.print(f"[green]✓ Import completed successfully![/green]")
        
        # Show next steps
        console.print("\n[cyan]Next steps:[/cyan]")
        console.print(f"- Check memories: [yellow]aigpt status[/yellow]")
        console.print(f"- Chat with AI: [yellow]aigpt chat {user_id} \"hello\"[/yellow]")
        console.print(f"- View relationships: [yellow]aigpt relationships[/yellow]")
        
    except Exception as e:
        console.print(f"[red]Error during import: {e}[/red]")
        raise typer.Exit(1)


@app.command()
def conversation(
    user_id: str = typer.Argument(..., help="User ID (atproto DID)"),
    data_dir: Optional[Path] = typer.Option(None, "--data-dir", "-d", help="Data directory"),
    model: Optional[str] = typer.Option(None, "--model", "-m", help="AI model to use"),
    provider: Optional[str] = typer.Option(None, "--provider", help="AI provider (ollama/openai)")
):
    """Simple continuous conversation mode with MCP support"""
    # Initialize MCP client
    mcp_client = MCPClient()
    persona = get_persona(data_dir)
    
    # Get defaults from config if not provided
    config_instance = Config()
    if not provider:
        provider = config_instance.get("default_provider", "ollama")
    if not model:
        if provider == "ollama":
            model = config_instance.get("providers.ollama.default_model", "qwen3:latest")
        elif provider == "openai":
            model = config_instance.get("providers.openai.default_model", "gpt-4o-mini")
        else:
            model = "qwen3:latest"  # fallback
    
    # Create AI provider with MCP client
    ai_provider = None
    try:
        ai_provider = create_ai_provider(provider=provider, model=model, mcp_client=mcp_client)
        console.print(f"[dim]Using {provider} with model {model}[/dim]")
    except Exception as e:
        console.print(f"[yellow]Warning: Could not create AI provider: {e}[/yellow]")
    
    # MCP status
    server_info = mcp_client.get_server_info()
    if server_info["available"]:
        console.print(f"[green]✓ MCP Server connected: {server_info['display_name']}[/green]")
        console.print(f"[dim]  URL: {server_info['base_url']} | Endpoints: {server_info['endpoints']}[/dim]")
    else:
        console.print(f"[yellow]⚠ MCP Server unavailable (running in local mode)[/yellow]")
    
    # Welcome message
    console.print(f"[cyan]Conversation with AI started. Type 'exit' or 'quit' to end.[/cyan]")
    if server_info["available"]:
        console.print(f"[dim]MCP commands: /memories, /search, /context, /relationship[/dim]\n")
    else:
        console.print()
    
    # History for conversation mode
    actual_data_dir = data_dir if data_dir else DEFAULT_DATA_DIR
    history_file = actual_data_dir / "conversation_history.txt"
    history = FileHistory(str(history_file))
    
    # Custom completer for slash commands and phrases with MCP support
    class ConversationCompleter(Completer):
        def __init__(self, mcp_available: bool = False):
            self.basic_commands = ['/status', '/help', '/clear', '/exit', '/quit']
            self.mcp_commands = ['/memories', '/search', '/context', '/relationship'] if mcp_available else []
            self.phrases = ['こんにちは', '今日は', 'ありがとう', 'お疲れ様',
                           'どう思う？', 'どうですか？', '教えて', 'わかりました']
            self.all_commands = self.basic_commands + self.mcp_commands
        
        def get_completions(self, document, complete_event):
            text = document.text_before_cursor
            
            # If text starts with '/', complete slash commands
            if text.startswith('/'):
                for cmd in self.all_commands:
                    if cmd.startswith(text):
                        yield Completion(cmd, start_position=-len(text))
            # For other text, complete phrases
            else:
                for phrase in self.phrases:
                    if phrase.startswith(text):
                        yield Completion(phrase, start_position=-len(text))
    
    completer = ConversationCompleter(mcp_client.available)
    
    while True:
        try:
            # Simple prompt with completion
            user_input = ptk_prompt(
                f"{user_id}> ",
                history=history,
                auto_suggest=AutoSuggestFromHistory(),
                completer=completer
            ).strip()
            
            if not user_input:
                continue
                
            # Exit commands
            if user_input.lower() in ['exit', 'quit', 'bye', '/exit', '/quit']:
                console.print("[cyan]Conversation ended.[/cyan]")
                break
            
            # Slash commands
            elif user_input.lower() == '/status':
                state = persona.get_current_state()
                rel = persona.relationships.get_or_create_relationship(user_id)
                console.print(f"\n[cyan]AI Status:[/cyan]")
                console.print(f"Mood: {state.current_mood}")
                console.print(f"Fortune: {state.fortune.fortune_value}/10")
                console.print(f"Relationship: {rel.status.value} ({rel.score:.2f})")
                console.print("")
                continue
            
            elif user_input.lower() in ['/help', '/']:
                console.print(f"\n[cyan]Conversation Commands:[/cyan]")
                console.print("  /status  - Show AI status and relationship")
                console.print("  /help    - Show this help")
                console.print("  /clear   - Clear screen")
                console.print("  /exit    - End conversation")
                console.print("  /        - Show commands (same as /help)")
                if mcp_client.available:
                    console.print(f"\n[cyan]MCP Commands:[/cyan]")
                    console.print("  /memories     - Show recent memories")
                    console.print("  /search <keywords> - Search memories")
                    console.print("  /context <query>   - Get contextual memories")
                    console.print("  /relationship      - Show relationship via MCP")
                    
                    if mcp_client.has_card_tools:
                        console.print(f"\n[cyan]Card Commands:[/cyan]")
                        console.print("  AI can answer questions about cards:")
                        console.print("  - 'Show my cards'")
                        console.print("  - 'Draw a card' / 'Gacha'")
                        console.print("  - 'Analyze my collection'")
                        console.print("  - 'Show gacha stats'")
                console.print("\n  <message> - Chat with AI\n")
                continue
                
            elif user_input.lower() == '/clear':
                console.clear()
                continue
            
            # MCP Commands
            elif user_input.lower() == '/memories' and mcp_client.available:
                memories = asyncio.run(mcp_client.get_memories(limit=5))
                if memories:
                    console.print(f"\n[cyan]Recent Memories (via MCP):[/cyan]")
                    for i, mem in enumerate(memories[:5], 1):
                        console.print(f"  {i}. [{mem.get('level', 'unknown')}] {mem.get('content', '')[:100]}...")
                    console.print("")
                else:
                    console.print("[yellow]No memories found[/yellow]")
                continue
            
            elif user_input.lower().startswith('/search ') and mcp_client.available:
                query = user_input[8:].strip()
                if query:
                    keywords = query.split()
                    results = asyncio.run(mcp_client.search_memories(keywords))
                    if results:
                        console.print(f"\n[cyan]Memory Search Results for '{query}' (via MCP):[/cyan]")
                        for i, mem in enumerate(results[:5], 1):
                            console.print(f"  {i}. {mem.get('content', '')[:100]}...")
                        console.print("")
                    else:
                        console.print(f"[yellow]No memories found for '{query}'[/yellow]")
                else:
                    console.print("[red]Usage: /search <keywords>[/red]")
                continue
            
            elif user_input.lower().startswith('/context ') and mcp_client.available:
                query = user_input[9:].strip()
                if query:
                    results = asyncio.run(mcp_client.get_contextual_memories(query, limit=5))
                    if results:
                        console.print(f"\n[cyan]Contextual Memories for '{query}' (via MCP):[/cyan]")
                        for i, mem in enumerate(results[:5], 1):
                            console.print(f"  {i}. {mem.get('content', '')[:100]}...")
                        console.print("")
                    else:
                        console.print(f"[yellow]No contextual memories found for '{query}'[/yellow]")
                else:
                    console.print("[red]Usage: /context <query>[/red]")
                continue
            
            elif user_input.lower() == '/relationship' and mcp_client.available:
                rel_data = asyncio.run(mcp_client.get_relationship(user_id))
                if rel_data:
                    console.print(f"\n[cyan]Relationship (via MCP):[/cyan]")
                    console.print(f"Status: {rel_data.get('status', 'unknown')}")
                    console.print(f"Score: {rel_data.get('score', 0):.2f}")
                    console.print(f"Interactions: {rel_data.get('total_interactions', 0)}")
                    console.print("")
                else:
                    console.print("[yellow]No relationship data found[/yellow]")
                continue
            
            # Process interaction - try MCP first, fallback to local
            if mcp_client.available:
                try:
                    mcp_result = asyncio.run(mcp_client.process_interaction(user_id, user_input))
                    if mcp_result and 'response' in mcp_result:
                        response = mcp_result['response']
                        console.print(f"AI> {response} [dim](via MCP)[/dim]\n")
                        continue
                except Exception as e:
                    console.print(f"[yellow]MCP failed, using local: {e}[/yellow]")
            
            # Fallback to local processing
            response, relationship_delta = persona.process_interaction(user_id, user_input, ai_provider)
            
            # Simple AI response display (no Panel, no extra info)
            console.print(f"AI> {response}\n")
            
        except KeyboardInterrupt:
            console.print("\n[yellow]Use 'exit' or 'quit' to end conversation[/yellow]")
        except EOFError:
            console.print("\n[cyan]Conversation ended.[/cyan]")
            break
        except Exception as e:
            console.print(f"[red]Error: {e}[/red]")


# Alias for conversation command
@app.command()
def conv(
    user_id: str = typer.Argument(..., help="User ID (atproto DID)"),
    data_dir: Optional[Path] = typer.Option(None, "--data-dir", "-d", help="Data directory"),
    model: Optional[str] = typer.Option(None, "--model", "-m", help="AI model to use"),
    provider: Optional[str] = typer.Option(None, "--provider", help="AI provider (ollama/openai)")
):
    """Alias for conversation command"""
    conversation(user_id, data_dir, model, provider)


if __name__ == "__main__":
    app()