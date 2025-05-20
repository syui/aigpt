#[derive(Debug, Serialize, Deserialize)]
pub struct RelationalAutonomousAI {
    pub system_name: String,
    pub description: String,
    pub core_components: CoreComponents,
    pub extensions: Extensions,
    pub note: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CoreComponents {
    pub personality: Personality,
    pub relationship: Relationship,
    pub environment: Environment,
    pub memory: Memory,
    pub message_trigger: MessageTrigger,
    pub message_generation: MessageGeneration,
    pub state_transition: StateTransition,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Personality {
    pub r#type: String,
    pub variants: Vec<String>,
    pub parameters: PersonalityParameters,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PersonalityParameters {
    pub message_trigger_style: String,
    pub decay_rate_modifier: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Relationship {
    pub parameters: Vec<String>,
    pub properties: RelationshipProperties,
    pub decay_function: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RelationshipProperties {
    pub persistent: bool,
    pub hidden: bool,
    pub irreversible: bool,
    pub decay_over_time: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Environment {
    pub daily_luck: DailyLuck,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DailyLuck {
    pub r#type: String,
    pub range: Vec<f32>,
    pub update: String,
    pub streak_mechanism: StreakMechanism,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StreakMechanism {
    pub trigger: String,
    pub effect: String,
    pub chance: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Memory {
    pub long_term_memory: String,
    pub short_term_context: String,
    pub usage_in_generation: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageTrigger {
    pub condition: TriggerCondition,
    pub timing: TriggerTiming,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TriggerCondition {
    pub relationship_threshold: String,
    pub time_decay: bool,
    pub environment_luck: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TriggerTiming {
    pub based_on: Vec<String>,
    pub modifiers: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageGeneration {
    pub style_variants: Vec<String>,
    pub influenced_by: Vec<String>,
    pub llm_integration: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StateTransition {
    pub states: Vec<String>,
    pub transitions: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Extensions {
    pub persistence: Persistence,
    pub api: Api,
    pub scheduler: Scheduler,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Persistence {
    pub database: String,
    pub storage_items: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Api {
    pub llm: String,
    pub mode: String,
    pub external_event_trigger: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Scheduler {
    pub async_event_loop: bool,
    pub interval_check: i32,
    pub time_decay_check: bool,
}
