use std::fs::{self};

use anyhow::{Context, Result};

use regex::Regex;

pub struct CommanderEnvironment {
    pub history_file_path: String,
    pub saved_commands_file_path: String,
    pub history_file_regex: Regex,
}

impl CommanderEnvironment {
    fn regex_from_env() -> Result<Regex> {
        Regex::new(r".*;(.*)$").context("Bad Regex")
    }
    fn history_file_path_from_env() -> String {
        "/home/blee/.zsh_history".to_string()
    }
    fn saved_commands_file_path_from_config() -> String {
        "./.commander_history".to_string()
    }
    pub fn new() -> Result<Self> {
        Ok(CommanderEnvironment {
            history_file_path: Self::history_file_path_from_env(),
            saved_commands_file_path: Self::saved_commands_file_path_from_config(),
            history_file_regex: Self::regex_from_env()?,
        })
    }
}

pub struct CommandList {
    pub commands: Vec<String>,
}

impl CommandList {
    fn read_in_from_env(hist_file_path: String, raw_hist_regex: Regex) -> Result<Vec<String>> {
        let mut return_commands = Vec::new();
        let content = fs::read_to_string(hist_file_path).context("Failed to read env history");
        for i in content?.split('\n') {
            if let Some(caps) = raw_hist_regex.captures(i) {
                let (_, captures): (&str, [&str; 1]) = caps.extract();
                return_commands.push(captures[0].to_owned());
            }
        }
        return Ok(return_commands);
    }

    fn read_in_saved(saved_commands_file_path: String) -> Result<Vec<String>> {
        let mut return_commands = Vec::new();
        let content = fs::read_to_string(saved_commands_file_path)
            .context("Failed to read in saved commands");
        for i in content?.split('\n') {
            return_commands.push(i.to_owned());
        }
        return Ok(return_commands);
    }
    fn merge_commands(
        base_commands: &mut Vec<String>,
        insert_commands: &mut Vec<String>,
    ) -> Vec<String> {
        insert_commands.sort_unstable();
        insert_commands.dedup();

        let mut merged_commands = Vec::new();
        let mut base_iter = base_commands.iter().peekable();
        let mut insert_iter = insert_commands.iter().peekable();

        loop {
            match (base_iter.peek(), insert_iter.peek()) {
                (Some(base), Some(insert)) => {
                    if base < insert {
                        merged_commands.push(base_iter.next().unwrap().to_string());
                    } else if base > insert {
                        merged_commands.push(insert_iter.next().unwrap().to_string());
                    } else {
                        insert_iter.next();
                    }
                }
                (None, Some(_)) => {
                    merged_commands.push(insert_iter.next().unwrap().to_string());
                }
                (Some(_), None) => {
                    merged_commands.push(base_iter.next().unwrap().to_string());
                }
                (None, None) => break,
            }
        }
        merged_commands
    }
    fn save_commands() -> Result<()> {
        Ok(())
    }
    pub fn new(env: CommanderEnvironment) -> Result<Self> {
        let mut hist_commands =
            Self::read_in_from_env(env.history_file_path, env.history_file_regex)?;

        let mut saved_commands = Self::read_in_saved(env.saved_commands_file_path)?;

        Ok(CommandList {
            commands: Self::merge_commands(&mut saved_commands, &mut hist_commands),
        })
    }
}
