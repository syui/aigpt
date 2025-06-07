"""Template management for documentation generation."""

from datetime import datetime
from pathlib import Path
from typing import Dict, List, Optional

from jinja2 import Environment, FileSystemLoader

from .config import DocsConfig, get_claude_root


class DocumentationTemplateManager:
    """Manages Jinja2 templates for documentation generation."""
    
    def __init__(self, config: DocsConfig):
        self.config = config
        self.claude_root = get_claude_root()
        self.templates_dir = self.claude_root / "templates"
        self.core_dir = self.claude_root / "core"
        self.projects_dir = self.claude_root / "projects"
        
        # Setup Jinja2 environment
        self.env = Environment(
            loader=FileSystemLoader([
                str(self.templates_dir),
                str(self.core_dir),
                str(self.projects_dir),
            ]),
            trim_blocks=True,
            lstrip_blocks=True,
        )
        
        # Add custom filters
        self.env.filters["timestamp"] = self._timestamp_filter
    
    def _timestamp_filter(self, format_str: str = "%Y-%m-%d %H:%M:%S") -> str:
        """Jinja2 filter for timestamps."""
        return datetime.now().strftime(format_str)
    
    def get_template_context(self, project_name: str, components: List[str]) -> Dict:
        """Get template context for documentation generation."""
        project_info = self.config.get_project_info(project_name)
        
        return {
            "config": self.config,
            "project_name": project_name,
            "project_info": project_info,
            "components": components,
            "timestamp": datetime.now().strftime("%Y-%m-%d %H:%M:%S"),
            "ai_md_content": self._get_ai_md_content(),
        }
    
    def _get_ai_md_content(self) -> Optional[str]:
        """Get content from ai.md file."""
        ai_md_path = self.claude_root.parent / "ai.md"
        if ai_md_path.exists():
            return ai_md_path.read_text(encoding="utf-8")
        return None
    
    def render_component(self, component_name: str, context: Dict) -> str:
        """Render a specific component."""
        component_files = {
            "core": ["philosophy.md", "naming.md", "architecture.md"],
            "philosophy": ["philosophy.md"],
            "naming": ["naming.md"],
            "architecture": ["architecture.md"],
            "specific": [f"{context['project_name']}.md"],
        }
        
        if component_name not in component_files:
            raise ValueError(f"Unknown component: {component_name}")
        
        content_parts = []
        
        for file_name in component_files[component_name]:
            file_path = self.core_dir / file_name
            if component_name == "specific":
                file_path = self.projects_dir / file_name
            
            if file_path.exists():
                content = file_path.read_text(encoding="utf-8")
                content_parts.append(content)
        
        return "\n\n".join(content_parts)
    
    def generate_documentation(
        self,
        project_name: str,
        components: List[str],
        output_path: Optional[Path] = None,
    ) -> str:
        """Generate complete documentation."""
        context = self.get_template_context(project_name, components)
        
        # Build content sections
        content_sections = []
        
        # Add ai.md header if available
        if context["ai_md_content"]:
            content_sections.append(context["ai_md_content"])
            content_sections.append("---\n")
        
        # Add title and metadata
        content_sections.append("# エコシステム統合設計書（詳細版）\n")
        content_sections.append("このドキュメントは動的生成されました。修正は元ファイルで行ってください。\n")
        content_sections.append(f"生成日時: {context['timestamp']}")
        content_sections.append(f"対象プロジェクト: {project_name}")
        content_sections.append(f"含有コンポーネント: {','.join(components)}\n")
        
        # Add component content
        for component in components:
            try:
                component_content = self.render_component(component, context)
                if component_content.strip():
                    content_sections.append(component_content)
            except ValueError as e:
                print(f"Warning: {e}")
        
        # Add footer
        footer = """
# footer

© syui

# important-instruction-reminders
Do what has been asked; nothing more, nothing less.
NEVER create files unless they're absolutely necessary for achieving your goal.
ALWAYS prefer editing an existing file to creating a new one.
NEVER proactively create documentation files (*.md) or README files. Only create documentation files if explicitly requested by the User.
"""
        content_sections.append(footer)
        
        # Join all sections
        final_content = "\n".join(content_sections)
        
        # Write to file if output path provided
        if output_path:
            output_path.parent.mkdir(parents=True, exist_ok=True)
            output_path.write_text(final_content, encoding="utf-8")
        
        return final_content
    
    def list_available_components(self) -> List[str]:
        """List available components."""
        return ["core", "philosophy", "naming", "architecture", "specific"]
    
    def validate_components(self, components: List[str]) -> List[str]:
        """Validate and return valid components."""
        available = self.list_available_components()
        valid_components = []
        
        for component in components:
            if component in available:
                valid_components.append(component)
            else:
                print(f"Warning: Unknown component '{component}' (available: {available})")
        
        return valid_components or ["core", "specific"]  # Default fallback