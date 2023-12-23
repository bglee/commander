use std::{env::{self}, fs::File };

fn read_in_history ()-> Result<Vec<String>, > {
    let hist_file_name = env::var("HISTFILE")?;
    let hist_file = File::open(hist_file_name)?;

}

pub fn full() -> Vec<&'static str> {
    let mock_list = vec!["cd ..", "ls -la", "git commit -m \"test\""];
    return mock_list;
}

enum CommandType {
    Basic,
}
struct Command<'a> {
    raw: &'a str,
    command_type: CommandType,
}

enum SavedCommandUpdateDirective {
    Append(Vec<Command>),
    Replace,
}
struct CommandList<'a> {
    commands: Vec<Command<'a>>,
    write_directive: SavedCommandUpdateDirective,
}

impl CommandList {
    fn read_in_from_env () {}
    fn read_in_saved () {}
    fn merge_commands() {}
    fn save_commands () {}
    pub init()-> Self {

    }
}
