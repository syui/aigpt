use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};
use chrono::{DateTime, Utc, Duration};

use crate::config::Config;
use crate::persona::Persona;
use crate::transmission::TransmissionController;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTask {
    pub id: String,
    pub task_type: TaskType,
    pub next_run: DateTime<Utc>,
    pub interval_hours: Option<i64>,
    pub enabled: bool,
    pub last_run: Option<DateTime<Utc>>,
    pub run_count: u32,
    pub max_runs: Option<u32>,
    pub created_at: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskType {
    DailyMaintenance,
    AutoTransmission,
    RelationshipDecay,
    BreakthroughCheck,
    MaintenanceTransmission,
    Custom(String),
}

impl std::fmt::Display for TaskType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskType::DailyMaintenance => write!(f, "daily_maintenance"),
            TaskType::AutoTransmission => write!(f, "auto_transmission"),
            TaskType::RelationshipDecay => write!(f, "relationship_decay"),
            TaskType::BreakthroughCheck => write!(f, "breakthrough_check"),
            TaskType::MaintenanceTransmission => write!(f, "maintenance_transmission"),
            TaskType::Custom(name) => write!(f, "custom_{}", name),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskExecution {
    pub task_id: String,
    pub execution_time: DateTime<Utc>,
    pub duration_ms: u64,
    pub success: bool,
    pub result: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIScheduler {
    config: Config,
    tasks: HashMap<String, ScheduledTask>,
    execution_history: Vec<TaskExecution>,
    last_check: Option<DateTime<Utc>>,
}

impl AIScheduler {
    pub fn new(config: &Config) -> Result<Self> {
        let (tasks, execution_history) = Self::load_scheduler_data(config)?;
        
        let mut scheduler = AIScheduler {
            config: config.clone(),
            tasks,
            execution_history,
            last_check: None,
        };
        
        // Initialize default tasks if none exist
        if scheduler.tasks.is_empty() {
            scheduler.create_default_tasks()?;
        }
        
        Ok(scheduler)
    }
    
    pub async fn run_scheduled_tasks(&mut self, persona: &mut Persona, transmission_controller: &mut TransmissionController) -> Result<Vec<TaskExecution>> {
        let now = Utc::now();
        let mut executions = Vec::new();
        
        // Find tasks that are due to run
        let due_task_ids: Vec<String> = self.tasks
            .iter()
            .filter(|(_, task)| task.enabled && task.next_run <= now)
            .filter(|(_, task)| {
                // Check if task hasn't exceeded max runs
                if let Some(max_runs) = task.max_runs {
                    task.run_count < max_runs
                } else {
                    true
                }
            })
            .map(|(id, _)| id.clone())
            .collect();
        
        for task_id in due_task_ids {
            let execution = self.execute_task(&task_id, persona, transmission_controller).await?;
            executions.push(execution);
        }
        
        self.last_check = Some(now);
        self.save_scheduler_data()?;
        
        Ok(executions)
    }
    
    async fn execute_task(&mut self, task_id: &str, persona: &mut Persona, transmission_controller: &mut TransmissionController) -> Result<TaskExecution> {
        let start_time = Utc::now();
        let mut execution = TaskExecution {
            task_id: task_id.to_string(),
            execution_time: start_time,
            duration_ms: 0,
            success: false,
            result: None,
            error: None,
        };
        
        // Get task type without borrowing mutably
        let task_type = {
            let task = self.tasks.get(task_id)
                .ok_or_else(|| anyhow::anyhow!("Task not found: {}", task_id))?;
            task.task_type.clone()
        };
        
        // Execute the task based on its type
        let result = match &task_type {
            TaskType::DailyMaintenance => self.execute_daily_maintenance(persona, transmission_controller).await,
            TaskType::AutoTransmission => self.execute_auto_transmission(persona, transmission_controller).await,
            TaskType::RelationshipDecay => self.execute_relationship_decay(persona).await,
            TaskType::BreakthroughCheck => self.execute_breakthrough_check(persona, transmission_controller).await,
            TaskType::MaintenanceTransmission => self.execute_maintenance_transmission(persona, transmission_controller).await,
            TaskType::Custom(name) => self.execute_custom_task(name, persona, transmission_controller).await,
        };
        
        let end_time = Utc::now();
        execution.duration_ms = (end_time - start_time).num_milliseconds() as u64;
        
        // Now update the task state with mutable borrow
        match result {
            Ok(message) => {
                execution.success = true;
                execution.result = Some(message);
                
                // Update task state
                if let Some(task) = self.tasks.get_mut(task_id) {
                    task.last_run = Some(start_time);
                    task.run_count += 1;
                    
                    // Schedule next run if recurring
                    if let Some(interval_hours) = task.interval_hours {
                        task.next_run = start_time + Duration::hours(interval_hours);
                    } else {
                        // One-time task, disable it
                        task.enabled = false;
                    }
                }
            }
            Err(e) => {
                execution.error = Some(e.to_string());
                
                // For failed tasks, retry in a shorter interval
                if let Some(task) = self.tasks.get_mut(task_id) {
                    if task.interval_hours.is_some() {
                        task.next_run = start_time + Duration::minutes(15); // Retry in 15 minutes
                    }
                }
            }
        }
        
        self.execution_history.push(execution.clone());
        
        // Keep only recent execution history (last 1000 executions)
        if self.execution_history.len() > 1000 {
            self.execution_history.drain(..self.execution_history.len() - 1000);
        }
        
        Ok(execution)
    }
    
    async fn execute_daily_maintenance(&self, persona: &mut Persona, transmission_controller: &mut TransmissionController) -> Result<String> {
        // Run daily maintenance
        persona.daily_maintenance()?;
        
        // Check for maintenance transmissions
        let transmissions = transmission_controller.check_maintenance_transmissions(persona).await?;
        
        Ok(format!("Daily maintenance completed. {} maintenance transmissions sent.", transmissions.len()))
    }
    
    async fn execute_auto_transmission(&self, _persona: &mut Persona, transmission_controller: &mut TransmissionController) -> Result<String> {
        let transmissions = transmission_controller.check_autonomous_transmissions(_persona).await?;
        Ok(format!("Autonomous transmission check completed. {} transmissions sent.", transmissions.len()))
    }
    
    async fn execute_relationship_decay(&self, persona: &mut Persona) -> Result<String> {
        persona.daily_maintenance()?;
        Ok("Relationship time decay applied.".to_string())
    }
    
    async fn execute_breakthrough_check(&self, persona: &mut Persona, transmission_controller: &mut TransmissionController) -> Result<String> {
        let transmissions = transmission_controller.check_breakthrough_transmissions(persona).await?;
        Ok(format!("Breakthrough check completed. {} transmissions sent.", transmissions.len()))
    }
    
    async fn execute_maintenance_transmission(&self, persona: &mut Persona, transmission_controller: &mut TransmissionController) -> Result<String> {
        let transmissions = transmission_controller.check_maintenance_transmissions(persona).await?;
        Ok(format!("Maintenance transmission check completed. {} transmissions sent.", transmissions.len()))
    }
    
    async fn execute_custom_task(&self, _name: &str, _persona: &mut Persona, _transmission_controller: &mut TransmissionController) -> Result<String> {
        // Placeholder for custom task execution
        Ok("Custom task executed.".to_string())
    }
    
    pub fn create_task(&mut self, task_type: TaskType, next_run: DateTime<Utc>, interval_hours: Option<i64>) -> Result<String> {
        let task_id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();
        
        let task = ScheduledTask {
            id: task_id.clone(),
            task_type,
            next_run,
            interval_hours,
            enabled: true,
            last_run: None,
            run_count: 0,
            max_runs: None,
            created_at: now,
            metadata: HashMap::new(),
        };
        
        self.tasks.insert(task_id.clone(), task);
        self.save_scheduler_data()?;
        
        Ok(task_id)
    }
    
    pub fn enable_task(&mut self, task_id: &str) -> Result<()> {
        if let Some(task) = self.tasks.get_mut(task_id) {
            task.enabled = true;
            self.save_scheduler_data()?;
        }
        Ok(())
    }
    
    pub fn disable_task(&mut self, task_id: &str) -> Result<()> {
        if let Some(task) = self.tasks.get_mut(task_id) {
            task.enabled = false;
            self.save_scheduler_data()?;
        }
        Ok(())
    }
    
    pub fn delete_task(&mut self, task_id: &str) -> Result<()> {
        self.tasks.remove(task_id);
        self.save_scheduler_data()?;
        Ok(())
    }
    
    pub fn get_task(&self, task_id: &str) -> Option<&ScheduledTask> {
        self.tasks.get(task_id)
    }
    
    pub fn list_tasks(&self) -> &HashMap<String, ScheduledTask> {
        &self.tasks
    }
    
    pub fn get_due_tasks(&self) -> Vec<&ScheduledTask> {
        let now = Utc::now();
        self.tasks
            .values()
            .filter(|task| task.enabled && task.next_run <= now)
            .collect()
    }
    
    pub fn get_execution_history(&self, limit: Option<usize>) -> Vec<&TaskExecution> {
        let mut executions: Vec<_> = self.execution_history.iter().collect();
        executions.sort_by(|a, b| b.execution_time.cmp(&a.execution_time));
        
        match limit {
            Some(limit) => executions.into_iter().take(limit).collect(),
            None => executions,
        }
    }
    
    pub fn get_scheduler_stats(&self) -> SchedulerStats {
        let total_tasks = self.tasks.len();
        let enabled_tasks = self.tasks.values().filter(|task| task.enabled).count();
        let due_tasks = self.get_due_tasks().len();
        
        let total_executions = self.execution_history.len();
        let successful_executions = self.execution_history.iter()
            .filter(|exec| exec.success)
            .count();
        
        let today = Utc::now().date_naive();
        let today_executions = self.execution_history.iter()
            .filter(|exec| exec.execution_time.date_naive() == today)
            .count();
        
        let avg_duration = if total_executions > 0 {
            self.execution_history.iter()
                .map(|exec| exec.duration_ms)
                .sum::<u64>() as f64 / total_executions as f64
        } else {
            0.0
        };
        
        SchedulerStats {
            total_tasks,
            enabled_tasks,
            due_tasks,
            total_executions,
            successful_executions,
            today_executions,
            success_rate: if total_executions > 0 {
                successful_executions as f64 / total_executions as f64
            } else {
                0.0
            },
            avg_duration_ms: avg_duration,
        }
    }
    
    fn create_default_tasks(&mut self) -> Result<()> {
        let now = Utc::now();
        
        // Daily maintenance task - run every day at 3 AM
        let mut daily_maintenance_time = now.date_naive().and_hms_opt(3, 0, 0).unwrap().and_utc();
        if daily_maintenance_time <= now {
            daily_maintenance_time = daily_maintenance_time + Duration::days(1);
        }
        
        self.create_task(
            TaskType::DailyMaintenance,
            daily_maintenance_time,
            Some(24), // 24 hours = 1 day
        )?;
        
        // Auto transmission check - every 4 hours
        self.create_task(
            TaskType::AutoTransmission,
            now + Duration::hours(1),
            Some(4),
        )?;
        
        // Breakthrough check - every 2 hours
        self.create_task(
            TaskType::BreakthroughCheck,
            now + Duration::minutes(30),
            Some(2),
        )?;
        
        // Maintenance transmission - once per day
        let mut maintenance_time = now.date_naive().and_hms_opt(12, 0, 0).unwrap().and_utc();
        if maintenance_time <= now {
            maintenance_time = maintenance_time + Duration::days(1);
        }
        
        self.create_task(
            TaskType::MaintenanceTransmission,
            maintenance_time,
            Some(24), // 24 hours = 1 day
        )?;
        
        Ok(())
    }
    
    fn load_scheduler_data(config: &Config) -> Result<(HashMap<String, ScheduledTask>, Vec<TaskExecution>)> {
        let tasks_file = config.scheduler_tasks_file();
        let history_file = config.scheduler_history_file();
        
        let tasks = if tasks_file.exists() {
            let content = std::fs::read_to_string(tasks_file)
                .context("Failed to read scheduler tasks file")?;
            serde_json::from_str(&content)
                .context("Failed to parse scheduler tasks file")?
        } else {
            HashMap::new()
        };
        
        let history = if history_file.exists() {
            let content = std::fs::read_to_string(history_file)
                .context("Failed to read scheduler history file")?;
            serde_json::from_str(&content)
                .context("Failed to parse scheduler history file")?
        } else {
            Vec::new()
        };
        
        Ok((tasks, history))
    }
    
    fn save_scheduler_data(&self) -> Result<()> {
        // Save tasks
        let tasks_content = serde_json::to_string_pretty(&self.tasks)
            .context("Failed to serialize scheduler tasks")?;
        std::fs::write(&self.config.scheduler_tasks_file(), tasks_content)
            .context("Failed to write scheduler tasks file")?;
        
        // Save execution history
        let history_content = serde_json::to_string_pretty(&self.execution_history)
            .context("Failed to serialize scheduler history")?;
        std::fs::write(&self.config.scheduler_history_file(), history_content)
            .context("Failed to write scheduler history file")?;
        
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct SchedulerStats {
    pub total_tasks: usize,
    pub enabled_tasks: usize,
    pub due_tasks: usize,
    pub total_executions: usize,
    pub successful_executions: usize,
    pub today_executions: usize,
    pub success_rate: f64,
    pub avg_duration_ms: f64,
}