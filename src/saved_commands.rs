use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

const FILE_NAME: &str = ".commander.json";

#[derive(Serialize, Deserialize, Default)]
pub struct SavedCommands {
    commands: Vec<String>,
}

impl SavedCommands {
    pub fn load() -> Self {
        let path = Path::new(FILE_NAME);
        if !path.exists() {
            return Self::default();
        }
        fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(FILE_NAME, json);
        }
    }

    pub fn add(&mut self, command: String) {
        if !self.commands.contains(&command) {
            self.commands.push(command);
            self.save();
        }
    }

    pub fn contains(&self, command: &str) -> bool {
        self.commands.iter().any(|c| c == command)
    }

    pub fn commands(&self) -> &[String] {
        &self.commands
    }
}
