use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

const FILE_NAME: &str = ".commander.json";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TemplateParam {
    pub example: String,
    pub description: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Template {
    pub command: String,
    #[serde(flatten)]
    pub params: HashMap<String, TemplateParam>,
}

impl Template {
    pub fn placeholder_names(&self) -> Vec<String> {
        let mut keys: Vec<String> = self.params.keys().cloned().collect();
        keys.sort_by(|a, b| {
            let a_num: Result<usize, _> = a.parse();
            let b_num: Result<usize, _> = b.parse();
            match (a_num, b_num) {
                (Ok(a), Ok(b)) => a.cmp(&b),
                _ => a.cmp(b),
            }
        });
        keys
    }

    pub fn resolve(&self, values: &HashMap<String, String>) -> String {
        let mut result = self.command.clone();
        for (key, value) in values {
            result = result.replace(&format!("<{}>", key), value);
        }
        result
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct SavedEnvironment {
    #[serde(default)]
    default_view: String,
    commands: Vec<String>,
    #[serde(default)]
    templates: Vec<Template>,
}

impl SavedEnvironment {
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

    pub fn load_from_string(json: &str) -> Self {
        serde_json::from_str(json).unwrap_or_default()
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

    pub fn default_view(&self) -> &String{
        &self.default_view
    }

    pub fn commands(&self) -> &[String] {
        &self.commands
    }

    pub fn templates(&self) -> &[Template] {
        &self.templates
    }

    pub fn add_template(&mut self, template: Template) {
        // Replace if a template with the same command string already exists
        if let Some(pos) = self
            .templates
            .iter()
            .position(|t| t.command == template.command)
        {
            self.templates[pos] = template;
        } else {
            self.templates.push(template);
        }
        self.save();
    }

    pub fn find_template(&self, command: &str) -> Option<&Template> {
        self.templates.iter().find(|t| t.command == command)
    }
}
