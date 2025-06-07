"""Claude Code token usage and cost analysis commands."""

from pathlib import Path
from typing import Dict, List, Optional, Tuple
from datetime import datetime, timedelta
import json
import sqlite3

import typer
from rich.console import Console
from rich.panel import Panel
from rich.table import Table
from rich.progress import track

console = Console()
tokens_app = typer.Typer(help="Claude Code token usage and cost analysis")

# Claude Code pricing (estimated rates in USD)
CLAUDE_PRICING = {
    "input_tokens_per_1k": 0.003,   # $3 per 1M input tokens
    "output_tokens_per_1k": 0.015,  # $15 per 1M output tokens
    "usd_to_jpy": 150  # Exchange rate
}


def find_claude_data_dir() -> Optional[Path]:
    """Find Claude Code data directory."""
    possible_paths = [
        Path.home() / ".claude",
        Path.home() / ".config" / "claude",
        Path.cwd() / ".claude"
    ]
    
    for path in possible_paths:
        if path.exists() and (path / "projects").exists():
            return path
    
    return None


def parse_jsonl_files(claude_dir: Path) -> List[Dict]:
    """Parse Claude Code JSONL files safely."""
    records = []
    projects_dir = claude_dir / "projects"
    
    if not projects_dir.exists():
        return records
    
    # Find all .jsonl files recursively
    jsonl_files = list(projects_dir.rglob("*.jsonl"))
    
    for jsonl_file in track(jsonl_files, description="Reading Claude data..."):
        try:
            with open(jsonl_file, 'r', encoding='utf-8') as f:
                for line_num, line in enumerate(f, 1):
                    line = line.strip()
                    if not line:
                        continue
                    
                    try:
                        record = json.loads(line)
                        # Only include records with usage information
                        if (record.get('type') == 'assistant' and 
                            'message' in record and 
                            'usage' in record.get('message', {})):
                            records.append(record)
                    except json.JSONDecodeError:
                        # Skip malformed JSON lines
                        continue
                        
        except (IOError, PermissionError):
            # Skip files we can't read
            continue
    
    return records


def calculate_costs(records: List[Dict]) -> Dict[str, float]:
    """Calculate token costs from usage records."""
    total_input_tokens = 0
    total_output_tokens = 0
    total_cost_usd = 0
    
    for record in records:
        try:
            usage = record.get('message', {}).get('usage', {})
            
            input_tokens = int(usage.get('input_tokens', 0))
            output_tokens = int(usage.get('output_tokens', 0))
            
            # Calculate cost if not provided
            cost_usd = record.get('costUSD')
            if cost_usd is None:
                input_cost = (input_tokens / 1000) * CLAUDE_PRICING["input_tokens_per_1k"]
                output_cost = (output_tokens / 1000) * CLAUDE_PRICING["output_tokens_per_1k"]
                cost_usd = input_cost + output_cost
            else:
                cost_usd = float(cost_usd)
            
            total_input_tokens += input_tokens
            total_output_tokens += output_tokens
            total_cost_usd += cost_usd
            
        except (ValueError, TypeError, KeyError):
            # Skip records with invalid data
            continue
    
    return {
        'input_tokens': total_input_tokens,
        'output_tokens': total_output_tokens,
        'total_tokens': total_input_tokens + total_output_tokens,
        'cost_usd': total_cost_usd,
        'cost_jpy': total_cost_usd * CLAUDE_PRICING["usd_to_jpy"]
    }


def group_by_date(records: List[Dict]) -> Dict[str, Dict]:
    """Group records by date and calculate daily costs."""
    daily_stats = {}
    
    for record in records:
        try:
            timestamp = record.get('timestamp')
            if not timestamp:
                continue
            
            # Parse timestamp and convert to JST
            dt = datetime.fromisoformat(timestamp.replace('Z', '+00:00'))
            # Convert to JST (UTC+9)
            jst_dt = dt + timedelta(hours=9)
            date_key = jst_dt.strftime('%Y-%m-%d')
            
            if date_key not in daily_stats:
                daily_stats[date_key] = []
            
            daily_stats[date_key].append(record)
            
        except (ValueError, TypeError):
            continue
    
    # Calculate costs for each day
    daily_costs = {}
    for date_key, day_records in daily_stats.items():
        daily_costs[date_key] = calculate_costs(day_records)
    
    return daily_costs


@tokens_app.command("summary")
def token_summary(
    period: str = typer.Option("all", help="Period: today, week, month, all"),
    claude_dir: Optional[Path] = typer.Option(None, "--claude-dir", help="Claude data directory"),
    show_details: bool = typer.Option(False, "--details", help="Show detailed breakdown"),
    format: str = typer.Option("table", help="Output format: table, json")
):
    """Show Claude Code token usage summary and estimated costs."""
    
    # Find Claude data directory
    if claude_dir is None:
        claude_dir = find_claude_data_dir()
    
    if claude_dir is None:
        console.print("[red]❌ Claude Code data directory not found[/red]")
        console.print("[dim]Looked in: ~/.claude, ~/.config/claude, ./.claude[/dim]")
        raise typer.Abort()
    
    if not claude_dir.exists():
        console.print(f"[red]❌ Directory not found: {claude_dir}[/red]")
        raise typer.Abort()
    
    console.print(f"[cyan]📊 Analyzing Claude Code usage from: {claude_dir}[/cyan]")
    
    # Parse data
    records = parse_jsonl_files(claude_dir)
    
    if not records:
        console.print("[yellow]⚠️ No usage data found[/yellow]")
        return
    
    # Filter by period
    now = datetime.now()
    filtered_records = []
    
    if period == "today":
        today = now.strftime('%Y-%m-%d')
        for record in records:
            try:
                timestamp = record.get('timestamp')
                if timestamp:
                    dt = datetime.fromisoformat(timestamp.replace('Z', '+00:00'))
                    jst_dt = dt + timedelta(hours=9)
                    if jst_dt.strftime('%Y-%m-%d') == today:
                        filtered_records.append(record)
            except (ValueError, TypeError):
                continue
    
    elif period == "week":
        week_ago = now - timedelta(days=7)
        for record in records:
            try:
                timestamp = record.get('timestamp')
                if timestamp:
                    dt = datetime.fromisoformat(timestamp.replace('Z', '+00:00'))
                    jst_dt = dt + timedelta(hours=9)
                    if jst_dt.date() >= week_ago.date():
                        filtered_records.append(record)
            except (ValueError, TypeError):
                continue
    
    elif period == "month":
        month_ago = now - timedelta(days=30)
        for record in records:
            try:
                timestamp = record.get('timestamp')
                if timestamp:
                    dt = datetime.fromisoformat(timestamp.replace('Z', '+00:00'))
                    jst_dt = dt + timedelta(hours=9)
                    if jst_dt.date() >= month_ago.date():
                        filtered_records.append(record)
            except (ValueError, TypeError):
                continue
    
    else:  # all
        filtered_records = records
    
    # Calculate total costs
    total_stats = calculate_costs(filtered_records)
    
    if format == "json":
        # JSON output
        output = {
            "period": period,
            "total_records": len(filtered_records),
            "input_tokens": total_stats['input_tokens'],
            "output_tokens": total_stats['output_tokens'],
            "total_tokens": total_stats['total_tokens'],
            "estimated_cost_usd": round(total_stats['cost_usd'], 2),
            "estimated_cost_jpy": round(total_stats['cost_jpy'], 0)
        }
        console.print(json.dumps(output, indent=2))
        return
    
    # Table output
    console.print(Panel(
        f"[bold cyan]Claude Code Token Usage Report[/bold cyan]\n\n"
        f"Period: {period.title()}\n"
        f"Data source: {claude_dir}",
        title="📊 Usage Analysis",
        border_style="cyan"
    ))
    
    # Summary table
    summary_table = Table(title="Token Summary")
    summary_table.add_column("Metric", style="cyan")
    summary_table.add_column("Value", style="green")
    
    summary_table.add_row("Input Tokens", f"{total_stats['input_tokens']:,}")
    summary_table.add_row("Output Tokens", f"{total_stats['output_tokens']:,}")
    summary_table.add_row("Total Tokens", f"{total_stats['total_tokens']:,}")
    summary_table.add_row("", "")  # Separator
    summary_table.add_row("Estimated Cost (USD)", f"${total_stats['cost_usd']:.2f}")
    summary_table.add_row("Estimated Cost (JPY)", f"¥{total_stats['cost_jpy']:,.0f}")
    summary_table.add_row("Records Analyzed", str(len(filtered_records)))
    
    console.print(summary_table)
    
    # Show daily breakdown if requested
    if show_details:
        daily_costs = group_by_date(filtered_records)
        
        if daily_costs:
            console.print("\n")
            daily_table = Table(title="Daily Breakdown")
            daily_table.add_column("Date", style="cyan")
            daily_table.add_column("Input Tokens", style="blue")
            daily_table.add_column("Output Tokens", style="green")
            daily_table.add_column("Total Tokens", style="yellow")
            daily_table.add_column("Cost (JPY)", style="red")
            
            for date in sorted(daily_costs.keys(), reverse=True):
                stats = daily_costs[date]
                daily_table.add_row(
                    date,
                    f"{stats['input_tokens']:,}",
                    f"{stats['output_tokens']:,}",
                    f"{stats['total_tokens']:,}",
                    f"¥{stats['cost_jpy']:,.0f}"
                )
            
            console.print(daily_table)
    
    # Warning about estimates
    console.print("\n[dim]💡 Note: Costs are estimates based on Claude API pricing.[/dim]")
    console.print("[dim]   Actual Claude Code subscription costs may differ.[/dim]")


@tokens_app.command("daily")
def daily_breakdown(
    days: int = typer.Option(7, help="Number of days to show"),
    claude_dir: Optional[Path] = typer.Option(None, "--claude-dir", help="Claude data directory"),
):
    """Show daily token usage breakdown."""
    
    # Find Claude data directory
    if claude_dir is None:
        claude_dir = find_claude_data_dir()
    
    if claude_dir is None:
        console.print("[red]❌ Claude Code data directory not found[/red]")
        raise typer.Abort()
    
    console.print(f"[cyan]📅 Daily token usage (last {days} days)[/cyan]")
    
    # Parse data
    records = parse_jsonl_files(claude_dir)
    
    if not records:
        console.print("[yellow]⚠️ No usage data found[/yellow]")
        return
    
    # Group by date
    daily_costs = group_by_date(records)
    
    # Get recent days
    recent_dates = sorted(daily_costs.keys(), reverse=True)[:days]
    
    if not recent_dates:
        console.print("[yellow]No recent usage data found[/yellow]")
        return
    
    # Create table
    table = Table(title=f"Daily Usage (Last {len(recent_dates)} days)")
    table.add_column("Date", style="cyan")
    table.add_column("Input", style="blue")
    table.add_column("Output", style="green") 
    table.add_column("Total", style="yellow")
    table.add_column("Cost (JPY)", style="red")
    
    total_cost = 0
    for date in recent_dates:
        stats = daily_costs[date]
        total_cost += stats['cost_jpy']
        
        table.add_row(
            date,
            f"{stats['input_tokens']:,}",
            f"{stats['output_tokens']:,}",
            f"{stats['total_tokens']:,}",
            f"¥{stats['cost_jpy']:,.0f}"
        )
    
    # Add total row
    table.add_row(
        "──────────",
        "────────",
        "────────", 
        "────────",
        "──────────"
    )
    table.add_row(
        "【Total】",
        "",
        "",
        "",
        f"¥{total_cost:,.0f}"
    )
    
    console.print(table)
    console.print(f"\n[green]Total estimated cost for {len(recent_dates)} days: ¥{total_cost:,.0f}[/green]")


@tokens_app.command("status")  
def token_status(
    claude_dir: Optional[Path] = typer.Option(None, "--claude-dir", help="Claude data directory"),
):
    """Check Claude Code data availability and basic stats."""
    
    # Find Claude data directory
    if claude_dir is None:
        claude_dir = find_claude_data_dir()
    
    console.print("[cyan]🔍 Claude Code Data Status[/cyan]")
    
    if claude_dir is None:
        console.print("[red]❌ Claude Code data directory not found[/red]")
        console.print("\n[yellow]Searched locations:[/yellow]")
        console.print("  • ~/.claude")
        console.print("  • ~/.config/claude")
        console.print("  • ./.claude")
        console.print("\n[dim]Make sure Claude Code is installed and has been used.[/dim]")
        return
    
    console.print(f"[green]✅ Found data directory: {claude_dir}[/green]")
    
    projects_dir = claude_dir / "projects"
    if not projects_dir.exists():
        console.print("[yellow]⚠️ No projects directory found[/yellow]")
        return
    
    # Count files
    jsonl_files = list(projects_dir.rglob("*.jsonl"))
    console.print(f"[blue]📂 Found {len(jsonl_files)} JSONL files[/blue]")
    
    if jsonl_files:
        # Parse sample to check data quality
        sample_records = []
        for jsonl_file in jsonl_files[:3]:  # Check first 3 files
            try:
                with open(jsonl_file, 'r') as f:
                    for line in f:
                        if line.strip():
                            try:
                                record = json.loads(line.strip())
                                sample_records.append(record)
                                if len(sample_records) >= 10:
                                    break
                            except json.JSONDecodeError:
                                continue
                if len(sample_records) >= 10:
                    break
            except IOError:
                continue
        
        usage_records = [r for r in sample_records 
                        if r.get('type') == 'assistant' and 
                           'usage' in r.get('message', {})]
        
        console.print(f"[green]📊 Found {len(usage_records)} usage records in sample[/green]")
        
        if usage_records:
            console.print("[blue]✅ Data appears valid for cost analysis[/blue]")
            console.print("\n[dim]Run 'aigpt tokens summary' for full analysis[/dim]")
        else:
            console.print("[yellow]⚠️ No usage data found in sample[/yellow]")
    else:
        console.print("[yellow]⚠️ No JSONL files found[/yellow]")


# Export the tokens app
__all__ = ["tokens_app"]