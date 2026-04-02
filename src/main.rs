use anyhow::Result;
use app::app;
use std::collections::HashSet;
use std::io::{self, BufRead};

mod app;
mod filter;
mod filter_list;

fn main() -> Result<()> {
    let stdin = io::stdin();
    let mut seen = HashSet::new();
    let commands: Vec<String> = stdin
        .lock()
        .lines()
        .map_while(Result::ok)
        .filter(|line| !line.is_empty() && seen.insert(line.clone()))
        .collect();

    match app(commands)? {
        Some(command) => {
            println!("{}", command);
            Ok(())
        }
        None => {
            std::process::exit(1);
        }
    }
}
