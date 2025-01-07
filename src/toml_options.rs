use serde::Deserialize;

#[derive(Deserialize)]
pub struct TomlOptions {
    pub tasks_config: TasksConfig
}

#[derive(Deserialize)]
pub struct TasksConfig {
    pub tasks_list_name: String,
    pub max_due_future_days: Option<u16>,
}
