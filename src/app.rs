use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, HighlightSpacing, List, ListDirection, ListItem, Paragraph},
    style::{Modifier, Style},
    text::{Line, Span},
    Frame, Terminal
};
use std::io::{self, stderr, Write};
use std::process::{Command, Stdio};

use crate::filter_list::FilterableListState;
use crate::saved_commands::SavedCommands;

#[derive(PartialEq)]
enum ViewMode {
    All,
    SavedOnly,
}

struct AppContext {
    list: FilterableListState,
    run_command: Option<String>,
    exit_next: bool,
    view_mode: ViewMode,
    all_commands: Vec<String>,
    saved_commands: SavedCommands,
}

fn ui(frame: &mut Frame, app_context: &mut AppContext) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Percentage(100), Constraint::Min(3),Constraint::Min(1)])
        .split(frame.area());

    let items: Vec<ListItem> = app_context
        .list
        .get_filtered_items()
        .iter()
        .map(|item| {
            if app_context.saved_commands.contains(item) {
                ListItem::new(item.to_string())
                    .style(Style::default().fg(ratatui::style::Color::Cyan))
            } else {
                ListItem::new(item.to_string())
            }
        })
        .collect();
    let highlight_color = ratatui::style::Color::Green;
    let list = List::new(items)
        .highlight_symbol("❯ ")
        .highlight_style(Style::default().fg(highlight_color))
        .highlight_spacing(HighlightSpacing::Always)
        .direction(ListDirection::BottomToTop);
    frame.render_stateful_widget(
        list.block(Block::new().borders(Borders::NONE)),
        layout[0],
        &mut app_context.list.list_state,
    );

    frame.render_widget(
        Paragraph::new(format!("❯ {}", app_context.list.get_filter()))
            .block(Block::new().borders(Borders::ALL)),
        layout[1],
    );
    let key_style = Style::default().add_modifier(Modifier::BOLD).fg(ratatui::style::Color::White);
    let desc_style = Style::default().fg(ratatui::style::Color::DarkGray);
    let mode_str = match app_context.view_mode {
        ViewMode::All => "[all]",
        ViewMode::SavedOnly => "[saved]",
    };
    let help_line = Line::from(vec![
            Span::styled("enter", key_style),
        Span::styled(" select  ", desc_style),
        Span::styled("ctrl+q", key_style),
        Span::styled(" quit  ", desc_style),
        Span::styled("ctrl+j", key_style),
        Span::styled(" ↑  ", desc_style),
        Span::styled("ctrl+k", key_style),
        Span::styled(" ↓  ", desc_style),
        Span::styled("ctrl+c", key_style),
        Span::styled(" copy  ", desc_style),
        Span::styled("ctrl+s", key_style),
        Span::styled(" save  ", desc_style),
        Span::styled("ctrl+v", key_style),
        Span::styled(" view  ", desc_style),
        Span::styled(mode_str, Style::default().fg(ratatui::style::Color::Yellow)),
    ]);
    frame.render_widget(
        Paragraph::new(help_line)
            .style(Style::default().fg(ratatui::style::Color::Gray))
            .block(Block::new().borders(Borders::NONE)),
        layout[2],
    );
}

fn event_handler(app_context: &mut AppContext) -> io::Result<()> {
    if event::poll(std::time::Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press {
                if key.modifiers == KeyModifiers::CONTROL {
                    match key.code {
                        KeyCode::Char('q') => {
                            app_context.exit_next = true;
                        }
                        KeyCode::Char('k') => {
                            app_context.list.next();
                        }
                        KeyCode::Char('j') => {
                            app_context.list.previous();
                        }
                        KeyCode::Char('c') => {
                            if let Some(item) = app_context.list.get_current_item() {
                                copy_to_clipboard(&item);
                            }
                        }
                        KeyCode::Char('s') => {
                            if let Some(item) = app_context.list.get_current_item() {
                                app_context.saved_commands.add(item);
                            }
                        }
                        KeyCode::Char('v') => {
                            match app_context.view_mode {
                                ViewMode::All => {
                                    let saved = app_context.saved_commands.commands().to_vec();
                                    app_context.list.swap_items(saved);
                                    app_context.view_mode = ViewMode::SavedOnly;
                                }
                                ViewMode::SavedOnly => {
                                    app_context.list.swap_items(app_context.all_commands.clone());
                                    app_context.view_mode = ViewMode::All;
                                }
                            }
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
                            app_context.list.set_filter(str.to_string());
                        }
                        KeyCode::Up => {
                            app_context.list.next();
                        }
                        KeyCode::Down => {
                            app_context.list.previous();
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

fn copy_to_clipboard(text: &str) {
    if let Ok(mut child) = Command::new("pbcopy")
        .stdin(Stdio::piped())
        .spawn()
    {
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(text.as_bytes());
        }
        let _ = child.wait();
    }
}

fn run_main_term_loop(commands: Vec<String>) -> Result<Option<String>> {
    let mut terminal = Terminal::new(CrosstermBackend::new(stderr()))?;
    terminal.clear()?;

    let saved_commands = SavedCommands::load();

    // Merge saved commands into the list, deduplicating
    let mut all_commands = commands;
    for cmd in saved_commands.commands() {
        if !all_commands.contains(cmd) {
            all_commands.push(cmd.clone());
        }
    }

    let mut app_context = AppContext {
        list: FilterableListState::new(all_commands.clone()),
        exit_next: false,
        run_command: None,
        view_mode: ViewMode::All,
        all_commands,
        saved_commands,
    };

    loop {
        event_handler(&mut app_context)?;
        terminal.draw(|frame| ui(frame, &mut app_context))?;
        if app_context.exit_next {
            break;
        }
    }
    Ok(app_context.run_command)
}

pub fn app(commands: Vec<String>) -> Result<Option<String>> {
    stderr().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;

    let result_or_error = run_main_term_loop(commands);
    stderr().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;

    result_or_error
}
