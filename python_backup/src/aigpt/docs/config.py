"""Configuration management for documentation system."""

import json
from pathlib import Path
from typing import Any, Dict, List, Optional, Union

from pydantic import BaseModel, Field


class GitConfig(BaseModel):
    """Git configuration."""
    host: str = "git.syui.ai"
    protocol: str = "ssh"


class AtprotoConfig(BaseModel):
    """Atproto configuration."""
    host: str = "syu.is"
    protocol: str = "at"
    at_url: str = "at://ai.syu.is"
    did: str = "did:plc:6qyecktefllvenje24fcxnie"
    web: str = "https://web.syu.is/@ai"


class ProjectMetadata(BaseModel):
    """Project metadata."""
    last_updated: str
    structure_version: str
    domain: List[str]
    git: GitConfig
    atproto: AtprotoConfig


class ProjectInfo(BaseModel):
    """Individual project information."""
    type: Union[str, List[str]]  # Support both string and list
    text: str
    status: str
    branch: str = "main"
    git_url: Optional[str] = None
    detailed_specs: Optional[str] = None
    data_reference: Optional[str] = None
    features: Optional[str] = None


class AIConfig(BaseModel):
    """AI projects configuration."""
    ai: ProjectInfo
    gpt: ProjectInfo
    os: ProjectInfo
    game: ProjectInfo
    bot: ProjectInfo
    moji: ProjectInfo
    card: ProjectInfo
    api: ProjectInfo
    log: ProjectInfo
    verse: ProjectInfo
    shell: ProjectInfo


class DocsConfig(BaseModel):
    """Main documentation configuration model."""
    version: int = 2
    metadata: ProjectMetadata
    ai: AIConfig
    data: Dict[str, Any] = Field(default_factory=dict)
    deprecated: Dict[str, Any] = Field(default_factory=dict)

    @classmethod
    def load_from_file(cls, config_path: Path) -> "DocsConfig":
        """Load configuration from ai.json file."""
        if not config_path.exists():
            raise FileNotFoundError(f"Configuration file not found: {config_path}")
        
        with open(config_path, "r", encoding="utf-8") as f:
            data = json.load(f)
        
        return cls(**data)

    def get_project_info(self, project_name: str) -> Optional[ProjectInfo]:
        """Get project information by name."""
        return getattr(self.ai, project_name, None)

    def get_project_git_url(self, project_name: str) -> str:
        """Get git URL for project."""
        project = self.get_project_info(project_name)
        if project and project.git_url:
            return project.git_url
        
        # Construct URL from metadata
        host = self.metadata.git.host
        protocol = self.metadata.git.protocol
        
        if protocol == "ssh":
            return f"git@{host}:ai/{project_name}"
        else:
            return f"https://{host}/ai/{project_name}"

    def get_project_branch(self, project_name: str) -> str:
        """Get branch for project."""
        project = self.get_project_info(project_name)
        return project.branch if project else "main"

    def list_projects(self) -> List[str]:
        """List all available projects."""
        return list(self.ai.__fields__.keys())


def get_ai_root(custom_dir: Optional[Path] = None) -> Path:
    """Get AI ecosystem root directory.
    
    Priority order:
    1. --dir option (custom_dir parameter)
    2. AI_DOCS_DIR environment variable
    3. ai.gpt config file (docs.ai_root)
    4. Default relative path
    """
    if custom_dir:
        return custom_dir
    
    # Check environment variable
    import os
    env_dir = os.getenv("AI_DOCS_DIR")
    if env_dir:
        return Path(env_dir)
    
    # Check ai.gpt config file
    try:
        from ..config import Config
        config = Config()
        config_ai_root = config.get("docs.ai_root")
        if config_ai_root:
            return Path(config_ai_root).expanduser()
    except Exception:
        # If config loading fails, continue to default
        pass
    
    # Default: From gpt/src/aigpt/docs/config.py, go up to ai/ root
    return Path(__file__).parent.parent.parent.parent.parent


def get_claude_root(custom_dir: Optional[Path] = None) -> Path:
    """Get Claude documentation root directory."""
    return get_ai_root(custom_dir) / "claude"


def load_docs_config(custom_dir: Optional[Path] = None) -> DocsConfig:
    """Load documentation configuration."""
    config_path = get_ai_root(custom_dir) / "ai.json"
    return DocsConfig.load_from_file(config_path)