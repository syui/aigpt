"""Wiki generation utilities for ai.wiki management."""

import re
from pathlib import Path
from typing import Dict, List, Optional, Tuple

from rich.console import Console

from .config import DocsConfig, get_ai_root
from .utils import find_project_directories
from .git_utils import pull_wiki_repository, push_wiki_repository

console = Console()


class WikiGenerator:
    """Generates wiki content from project documentation."""
    
    def __init__(self, config: DocsConfig, ai_root: Path):
        self.config = config
        self.ai_root = ai_root
        self.wiki_root = ai_root / "ai.wiki" if (ai_root / "ai.wiki").exists() else None
    
    def extract_project_summary(self, project_md_path: Path) -> Dict[str, str]:
        """Extract key information from claude/projects/${repo}.md file."""
        if not project_md_path.exists():
            return {"title": "No documentation", "summary": "Project documentation not found", "status": "Unknown"}
        
        try:
            content = project_md_path.read_text(encoding="utf-8")
            
            # Extract title (first # heading)
            title_match = re.search(r'^# (.+)$', content, re.MULTILINE)
            title = title_match.group(1) if title_match else "Unknown Project"
            
            # Extract project overview/summary (look for specific patterns)
            summary = self._extract_summary_section(content)
            
            # Extract status information
            status = self._extract_status_info(content)
            
            # Extract key features/goals
            features = self._extract_features(content)
            
            return {
                "title": title,
                "summary": summary,
                "status": status,
                "features": features,
                "last_updated": self._get_last_updated_info(content)
            }
            
        except Exception as e:
            console.print(f"[yellow]Warning: Failed to parse {project_md_path}: {e}[/yellow]")
            return {"title": "Parse Error", "summary": str(e), "status": "Error"}
    
    def _extract_summary_section(self, content: str) -> str:
        """Extract summary or overview section."""
        # Look for common summary patterns
        patterns = [
            r'## 概要\s*\n(.*?)(?=\n##|\n#|\Z)',
            r'## Overview\s*\n(.*?)(?=\n##|\n#|\Z)',
            r'## プロジェクト概要\s*\n(.*?)(?=\n##|\n#|\Z)',
            r'\*\*目的\*\*: (.+?)(?=\n|$)',
            r'\*\*中核概念\*\*:\s*\n(.*?)(?=\n##|\n#|\Z)',
        ]
        
        for pattern in patterns:
            match = re.search(pattern, content, re.DOTALL | re.MULTILINE)
            if match:
                summary = match.group(1).strip()
                # Clean up and truncate
                summary = re.sub(r'\n+', ' ', summary)
                summary = re.sub(r'\s+', ' ', summary)
                return summary[:300] + "..." if len(summary) > 300 else summary
        
        # Fallback: first paragraph after title
        lines = content.split('\n')
        summary_lines = []
        found_content = False
        
        for line in lines:
            line = line.strip()
            if not line:
                if found_content and summary_lines:
                    break
                continue
            if line.startswith('#'):
                found_content = True
                continue
            if found_content and not line.startswith('*') and not line.startswith('-'):
                summary_lines.append(line)
                if len(' '.join(summary_lines)) > 200:
                    break
        
        return ' '.join(summary_lines)[:300] + "..." if summary_lines else "No summary available"
    
    def _extract_status_info(self, content: str) -> str:
        """Extract status information."""
        # Look for status patterns
        patterns = [
            r'\*\*状況\*\*: (.+?)(?=\n|$)',
            r'\*\*Status\*\*: (.+?)(?=\n|$)',
            r'\*\*現在の状況\*\*: (.+?)(?=\n|$)',
            r'- \*\*状況\*\*: (.+?)(?=\n|$)',
        ]
        
        for pattern in patterns:
            match = re.search(pattern, content)
            if match:
                return match.group(1).strip()
        
        return "No status information"
    
    def _extract_features(self, content: str) -> List[str]:
        """Extract key features or bullet points."""
        features = []
        
        # Look for bullet point lists
        lines = content.split('\n')
        in_list = False
        
        for line in lines:
            line = line.strip()
            if line.startswith('- ') or line.startswith('* '):
                feature = line[2:].strip()
                if len(feature) > 10 and not feature.startswith('**'):  # Skip metadata
                    features.append(feature)
                    in_list = True
                    if len(features) >= 5:  # Limit to 5 features
                        break
            elif in_list and not line:
                break
        
        return features
    
    def _get_last_updated_info(self, content: str) -> str:
        """Extract last updated information."""
        patterns = [
            r'生成日時: (.+?)(?=\n|$)',
            r'最終更新: (.+?)(?=\n|$)',
            r'Last updated: (.+?)(?=\n|$)',
        ]
        
        for pattern in patterns:
            match = re.search(pattern, content)
            if match:
                return match.group(1).strip()
        
        return "Unknown"
    
    def generate_project_wiki_page(self, project_name: str, project_info: Dict[str, str]) -> str:
        """Generate wiki page for a single project."""
        config_info = self.config.get_project_info(project_name)
        
        content = f"""# {project_name}

## 概要
{project_info['summary']}

## プロジェクト情報
- **タイプ**: {config_info.type if config_info else 'Unknown'}
- **説明**: {config_info.text if config_info else 'No description'}
- **ステータス**: {config_info.status if config_info else project_info.get('status', 'Unknown')}
- **ブランチ**: {config_info.branch if config_info else 'main'}
- **最終更新**: {project_info.get('last_updated', 'Unknown')}

## 主な機能・特徴
"""
        
        features = project_info.get('features', [])
        if features:
            for feature in features:
                content += f"- {feature}\n"
        else:
            content += "- 情報なし\n"
        
        content += f"""
## リンク
- **Repository**: https://git.syui.ai/ai/{project_name}
- **Project Documentation**: [claude/projects/{project_name}.md](https://git.syui.ai/ai/ai/src/branch/main/claude/projects/{project_name}.md)
- **Generated Documentation**: [{project_name}/claude.md](https://git.syui.ai/ai/{project_name}/src/branch/main/claude.md)

---
*このページは claude/projects/{project_name}.md から自動生成されました*
"""
        
        return content
    
    def generate_wiki_home_page(self, project_summaries: Dict[str, Dict[str, str]]) -> str:
        """Generate the main Home.md page with all project summaries."""
        content = """# AI Ecosystem Wiki

AI生態系プロジェクトの概要とドキュメント集約ページです。

## プロジェクト一覧

"""
        
        # Group projects by type
        project_groups = {}
        for project_name, info in project_summaries.items():
            config_info = self.config.get_project_info(project_name)
            project_type = config_info.type if config_info else 'other'
            if isinstance(project_type, list):
                project_type = project_type[0]  # Use first type
            
            if project_type not in project_groups:
                project_groups[project_type] = []
            project_groups[project_type].append((project_name, info))
        
        # Generate sections by type
        type_names = {
            'ai': '🧠 AI・知能システム',
            'gpt': '🤖 自律・対話システム', 
            'os': '💻 システム・基盤',
            'card': '🎮 ゲーム・エンターテイメント',
            'shell': '⚡ ツール・ユーティリティ',
            'other': '📦 その他'
        }
        
        for project_type, projects in project_groups.items():
            type_display = type_names.get(project_type, f'📁 {project_type}')
            content += f"### {type_display}\n\n"
            
            for project_name, info in projects:
                content += f"#### [{project_name}](auto/{project_name}.md)\n"
                content += f"{info['summary'][:150]}{'...' if len(info['summary']) > 150 else ''}\n\n"
                
                # Add quick status
                config_info = self.config.get_project_info(project_name)
                if config_info:
                    content += f"**Status**: {config_info.status}  \n"
                content += f"**Links**: [Repo](https://git.syui.ai/ai/{project_name}) | [Docs](https://git.syui.ai/ai/{project_name}/src/branch/main/claude.md)\n\n"
        
        content += """
---

## ディレクトリ構成

- `auto/` - 自動生成されたプロジェクト概要
- `claude/` - Claude Code作業記録
- `manual/` - 手動作成ドキュメント

---

*このページは ai.json と claude/projects/ から自動生成されました*  
*最終更新: {last_updated}*
""".format(last_updated=self._get_current_timestamp())
        
        return content
    
    def _get_current_timestamp(self) -> str:
        """Get current timestamp."""
        from datetime import datetime
        return datetime.now().strftime("%Y-%m-%d %H:%M:%S")
    
    def update_wiki_auto_directory(self, auto_pull: bool = True) -> Tuple[bool, List[str]]:
        """Update the auto/ directory with project summaries."""
        if not self.wiki_root:
            return False, ["ai.wiki directory not found"]
        
        # Pull latest changes from wiki repository first
        if auto_pull:
            success, message = pull_wiki_repository(self.wiki_root)
            if not success:
                console.print(f"[yellow]⚠️ Wiki pull failed: {message}[/yellow]")
                console.print("[dim]Continuing with local wiki update...[/dim]")
            else:
                console.print(f"[green]✅ Wiki repository updated[/green]")
        
        auto_dir = self.wiki_root / "auto"
        auto_dir.mkdir(exist_ok=True)
        
        # Get claude/projects directory
        claude_projects_dir = self.ai_root / "claude" / "projects"
        if not claude_projects_dir.exists():
            return False, [f"claude/projects directory not found: {claude_projects_dir}"]
        
        project_summaries = {}
        updated_files = []
        
        console.print("[blue]📋 Extracting project summaries from claude/projects/...[/blue]")
        
        # Process all projects from ai.json
        for project_name in self.config.list_projects():
            project_md_path = claude_projects_dir / f"{project_name}.md"
            
            # Extract summary from claude/projects/${project}.md
            project_info = self.extract_project_summary(project_md_path)
            project_summaries[project_name] = project_info
            
            # Generate individual project wiki page
            wiki_content = self.generate_project_wiki_page(project_name, project_info)
            wiki_file_path = auto_dir / f"{project_name}.md"
            
            try:
                wiki_file_path.write_text(wiki_content, encoding="utf-8")
                updated_files.append(f"auto/{project_name}.md")
                console.print(f"[green]✓ Generated auto/{project_name}.md[/green]")
            except Exception as e:
                console.print(f"[red]✗ Failed to write auto/{project_name}.md: {e}[/red]")
        
        # Generate Home.md
        try:
            home_content = self.generate_wiki_home_page(project_summaries)
            home_path = self.wiki_root / "Home.md"
            home_path.write_text(home_content, encoding="utf-8")
            updated_files.append("Home.md")
            console.print(f"[green]✓ Generated Home.md[/green]")
        except Exception as e:
            console.print(f"[red]✗ Failed to write Home.md: {e}[/red]")
        
        return True, updated_files