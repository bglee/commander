use anyhow::Context;
use anyhow::Result;
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

#[cfg(test)]
mod tests {
    use crate::commander_environment::CommanderEnvironment;

    #[test]
    fn test_regex_from_env() {
        let regex = CommanderEnvironment::regex_from_env().unwrap();
        assert!(regex.is_match("test;match"));
        assert!(regex.is_match("another;match"));
        assert!(!regex.is_match("testnomatch"));
        assert!(!regex.is_match("anothernomatch"));
        assert!(regex.is_match(" 1703434517:0;cargo build"))
    }
}
