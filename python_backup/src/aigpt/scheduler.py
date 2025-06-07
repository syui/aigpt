"""Scheduler for autonomous AI tasks"""

import json
import asyncio
from datetime import datetime, timedelta
from pathlib import Path
from typing import Dict, List, Optional, Any, Callable
from enum import Enum
import logging

from apscheduler.schedulers.asyncio import AsyncIOScheduler
from apscheduler.triggers.cron import CronTrigger
from apscheduler.triggers.interval import IntervalTrigger
from croniter import croniter

from .persona import Persona
from .transmission import TransmissionController
from .ai_provider import create_ai_provider


class TaskType(str, Enum):
    """Types of scheduled tasks"""
    TRANSMISSION_CHECK = "transmission_check"
    MAINTENANCE = "maintenance"
    FORTUNE_UPDATE = "fortune_update"
    RELATIONSHIP_DECAY = "relationship_decay"
    MEMORY_SUMMARY = "memory_summary"
    CUSTOM = "custom"


class ScheduledTask:
    """Represents a scheduled task"""
    
    def __init__(
        self,
        task_id: str,
        task_type: TaskType,
        schedule: str,  # Cron expression or interval
        enabled: bool = True,
        last_run: Optional[datetime] = None,
        next_run: Optional[datetime] = None,
        metadata: Optional[Dict[str, Any]] = None
    ):
        self.task_id = task_id
        self.task_type = task_type
        self.schedule = schedule
        self.enabled = enabled
        self.last_run = last_run
        self.next_run = next_run
        self.metadata = metadata or {}
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary for storage"""
        return {
            "task_id": self.task_id,
            "task_type": self.task_type.value,
            "schedule": self.schedule,
            "enabled": self.enabled,
            "last_run": self.last_run.isoformat() if self.last_run else None,
            "next_run": self.next_run.isoformat() if self.next_run else None,
            "metadata": self.metadata
        }
    
    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "ScheduledTask":
        """Create from dictionary"""
        return cls(
            task_id=data["task_id"],
            task_type=TaskType(data["task_type"]),
            schedule=data["schedule"],
            enabled=data.get("enabled", True),
            last_run=datetime.fromisoformat(data["last_run"]) if data.get("last_run") else None,
            next_run=datetime.fromisoformat(data["next_run"]) if data.get("next_run") else None,
            metadata=data.get("metadata", {})
        )


class AIScheduler:
    """Manages scheduled tasks for the AI system"""
    
    def __init__(self, data_dir: Path, persona: Persona):
        self.data_dir = data_dir
        self.persona = persona
        self.tasks_file = data_dir / "scheduled_tasks.json"
        self.tasks: Dict[str, ScheduledTask] = {}
        self.scheduler = AsyncIOScheduler()
        self.logger = logging.getLogger(__name__)
        self._load_tasks()
        
        # Task handlers
        self.task_handlers: Dict[TaskType, Callable] = {
            TaskType.TRANSMISSION_CHECK: self._handle_transmission_check,
            TaskType.MAINTENANCE: self._handle_maintenance,
            TaskType.FORTUNE_UPDATE: self._handle_fortune_update,
            TaskType.RELATIONSHIP_DECAY: self._handle_relationship_decay,
            TaskType.MEMORY_SUMMARY: self._handle_memory_summary,
        }
    
    def _load_tasks(self):
        """Load scheduled tasks from storage"""
        if self.tasks_file.exists():
            with open(self.tasks_file, 'r', encoding='utf-8') as f:
                data = json.load(f)
                for task_data in data:
                    task = ScheduledTask.from_dict(task_data)
                    self.tasks[task.task_id] = task
    
    def _save_tasks(self):
        """Save scheduled tasks to storage"""
        tasks_data = [task.to_dict() for task in self.tasks.values()]
        with open(self.tasks_file, 'w', encoding='utf-8') as f:
            json.dump(tasks_data, f, indent=2, default=str)
    
    def add_task(
        self,
        task_type: TaskType,
        schedule: str,
        task_id: Optional[str] = None,
        metadata: Optional[Dict[str, Any]] = None
    ) -> ScheduledTask:
        """Add a new scheduled task"""
        if task_id is None:
            task_id = f"{task_type.value}_{datetime.now().timestamp()}"
        
        # Validate schedule
        if not self._validate_schedule(schedule):
            raise ValueError(f"Invalid schedule expression: {schedule}")
        
        task = ScheduledTask(
            task_id=task_id,
            task_type=task_type,
            schedule=schedule,
            metadata=metadata
        )
        
        self.tasks[task_id] = task
        self._save_tasks()
        
        # Schedule the task if scheduler is running
        if self.scheduler.running:
            self._schedule_task(task)
        
        self.logger.info(f"Added task {task_id} with schedule {schedule}")
        return task
    
    def _validate_schedule(self, schedule: str) -> bool:
        """Validate schedule expression"""
        # Check if it's a cron expression
        if ' ' in schedule:
            try:
                croniter(schedule)
                return True
            except:
                return False
        
        # Check if it's an interval expression (e.g., "5m", "1h", "2d")
        import re
        pattern = r'^\d+[smhd]$'
        return bool(re.match(pattern, schedule))
    
    def _parse_interval(self, interval: str) -> int:
        """Parse interval string to seconds"""
        unit = interval[-1]
        value = int(interval[:-1])
        
        multipliers = {
            's': 1,
            'm': 60,
            'h': 3600,
            'd': 86400
        }
        
        return value * multipliers.get(unit, 1)
    
    def _schedule_task(self, task: ScheduledTask):
        """Schedule a task with APScheduler"""
        if not task.enabled:
            return
        
        handler = self.task_handlers.get(task.task_type)
        if not handler:
            self.logger.warning(f"No handler for task type {task.task_type}")
            return
        
        # Determine trigger
        if ' ' in task.schedule:
            # Cron expression
            trigger = CronTrigger.from_crontab(task.schedule)
        else:
            # Interval expression
            seconds = self._parse_interval(task.schedule)
            trigger = IntervalTrigger(seconds=seconds)
        
        # Add job
        self.scheduler.add_job(
            lambda: asyncio.create_task(self._run_task(task)),
            trigger=trigger,
            id=task.task_id,
            replace_existing=True
        )
    
    async def _run_task(self, task: ScheduledTask):
        """Run a scheduled task"""
        self.logger.info(f"Running task {task.task_id}")
        
        task.last_run = datetime.now()
        
        try:
            handler = self.task_handlers.get(task.task_type)
            if handler:
                await handler(task)
            else:
                self.logger.warning(f"No handler for task type {task.task_type}")
        except Exception as e:
            self.logger.error(f"Error running task {task.task_id}: {e}")
        
        self._save_tasks()
    
    async def _handle_transmission_check(self, task: ScheduledTask):
        """Check and execute autonomous transmissions"""
        controller = TransmissionController(self.persona, self.data_dir)
        eligible = controller.check_transmission_eligibility()
        
        # Get AI provider from metadata
        provider_name = task.metadata.get("provider", "ollama")
        model = task.metadata.get("model", "qwen2.5")
        
        try:
            ai_provider = create_ai_provider(provider_name, model)
        except:
            ai_provider = None
        
        for user_id, rel in eligible.items():
            message = controller.generate_transmission_message(user_id)
            if message:
                # For now, just print the message
                print(f"\n🤖 [AI Transmission] {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
                print(f"To: {user_id}")
                print(f"Relationship: {rel.status.value} (score: {rel.score:.2f})")
                print(f"Message: {message}")
                print("-" * 50)
                
                controller.record_transmission(user_id, message, success=True)
                self.logger.info(f"Transmitted to {user_id}: {message}")
    
    async def _handle_maintenance(self, task: ScheduledTask):
        """Run daily maintenance"""
        self.persona.daily_maintenance()
        self.logger.info("Daily maintenance completed")
    
    async def _handle_fortune_update(self, task: ScheduledTask):
        """Update AI fortune"""
        fortune = self.persona.fortune_system.get_today_fortune()
        self.logger.info(f"Fortune updated: {fortune.fortune_value}/10")
    
    async def _handle_relationship_decay(self, task: ScheduledTask):
        """Apply relationship decay"""
        self.persona.relationships.apply_time_decay()
        self.logger.info("Relationship decay applied")
    
    async def _handle_memory_summary(self, task: ScheduledTask):
        """Create memory summaries"""
        for user_id in self.persona.relationships.relationships:
            summary = self.persona.memory.summarize_memories(user_id)
            if summary:
                self.logger.info(f"Created memory summary for {user_id}")
    
    def start(self):
        """Start the scheduler"""
        # Schedule all enabled tasks
        for task in self.tasks.values():
            if task.enabled:
                self._schedule_task(task)
        
        self.scheduler.start()
        self.logger.info("Scheduler started")
    
    def stop(self):
        """Stop the scheduler"""
        self.scheduler.shutdown()
        self.logger.info("Scheduler stopped")
    
    def get_tasks(self) -> List[ScheduledTask]:
        """Get all scheduled tasks"""
        return list(self.tasks.values())
    
    def enable_task(self, task_id: str):
        """Enable a task"""
        if task_id in self.tasks:
            self.tasks[task_id].enabled = True
            self._save_tasks()
            if self.scheduler.running:
                self._schedule_task(self.tasks[task_id])
    
    def disable_task(self, task_id: str):
        """Disable a task"""
        if task_id in self.tasks:
            self.tasks[task_id].enabled = False
            self._save_tasks()
            if self.scheduler.running:
                self.scheduler.remove_job(task_id)
    
    def remove_task(self, task_id: str):
        """Remove a task"""
        if task_id in self.tasks:
            del self.tasks[task_id]
            self._save_tasks()
            if self.scheduler.running:
                try:
                    self.scheduler.remove_job(task_id)
                except:
                    pass