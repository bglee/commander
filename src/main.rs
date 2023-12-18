use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use filter_list::FilterableListState;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Corner, Direction, Layout},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::{
    env,
    fs::File,
    io::{self, stdout, Write},
};

mod command_list;
mod filter;
mod filter_list;

struct AppContext<'a> {
    list: FilterableListState<'a>,
    run_command: Option<String>,
    exit_next: bool,
}

fn ui(frame: &mut Frame, app_context: &mut AppContext) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Percentage(100), Constraint::Min(3)])
        .split(frame.size());

    let items: Vec<ListItem> = app_context
        .list
        .get_filtered_items()
        .iter()
        .map(|item| ListItem::new(item.to_string()))
        .collect();
    let list = List::new(items)
        .highlight_symbol("> ")
        .start_corner(Corner::BottomLeft);
    frame.render_stateful_widget(
        list.block(Block::new().borders(Borders::NONE)),
        layout[0],
        &mut app_context.list.state,
    );

    frame.render_widget(
        Paragraph::new(format!(">{}", app_context.list.get_filter()))
            .block(Block::new().borders(Borders::ALL)),
        layout[1],
    );
}

fn event_handler(app_context: &mut AppContext) -> io::Result<()> {
    if event::poll(std::time::Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press {
                if key.modifiers == KeyModifiers::CONTROL {
                    match key.code {
                        KeyCode::Char('c') => {
                            app_context.exit_next = true;
                        }
                        KeyCode::Char('k') => {
                            app_context.list.next();
                        }
                        KeyCode::Char('j') => {
                            app_context.list.previous();
                        }

                        _ => return Ok(()),
                    }
                } else {
                    match key.code {
                        KeyCode::Char(c) => app_context.list.set_filter(format!(
                            "{}{}",
                            app_context.list.get_filter(),
                            c
                        )),
                        KeyCode::Backspace => {
                            let mut str = app_context.list.get_filter().to_string();
                            str.pop();
                            app_context.list.set_filter(format!("{}", str));
                        }
                        KeyCode::Enter => {
                            app_context.run_command = app_context.list.get_current_item();
                            app_context.exit_next = true;
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    Ok(())
}

fn main() -> io::Result<()> {
    let args: Vec<_> = env::args().collect();

    let file_name = if args.len() > 1 {
        Some(args[1].to_owned())
    } else {
        None
    };
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    let mut app_context = AppContext {
        list: FilterableListState::new(),
        exit_next: false,
        run_command: None,
    };

    loop {
        event_handler(&mut app_context)?;
        terminal.draw(|frame| ui(frame, &mut app_context))?;
        if app_context.exit_next == true {
            break;
        }
    }

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;

    if let (Some(command), Some(file_name)) = (app_context.run_command, file_name) {
        let mut file = File::create(file_name)?;
        file.write_all(command.as_bytes())?;
    }

    Ok(())
}
