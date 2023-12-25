use anyhow::Result;
use app::app;
use std::{env, fs::File, io::Write};

mod app;
mod command_list;
mod filter;
mod filter_list;

fn main() -> Result<()> {
    let args: Vec<_> = env::args().collect();

    let file_name = if args.len() > 1 {
        Some(args[1].to_owned())
    } else {
        None
    };
    let run_command = app()?;

    if let (Some(command), Some(file_name)) = (run_command, file_name) {
        let mut file = File::create(file_name)?;
        file.write_all(command.as_bytes())?;
    }

    Ok(())
}
