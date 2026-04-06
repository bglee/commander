use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, HighlightSpacing, List, ListDirection, ListItem, Paragraph},
    Frame, Terminal,
};
use std::collections::HashMap;
use std::io::{self, stderr, Write};
use std::process::{Command, Stdio};

use crate::filter_list::FilterableListState;
use crate::saved_commands::{SavedCommands, Template, TemplateParam};
use crate::trust::{check_trust, TrustStatus, TrustStore};

#[derive(PartialEq)]
enum ViewMode {
    All,
    SavedOnly,
}

struct TemplatePlaceholder {
    key: String,
    example: String,
    description: String,
}

enum CreatePhase {
    Selecting,
    EnteringExample,
    EnteringDescription,
}

enum AppState {
    TrustPrompt {
        file_path: String,
        file_contents: String,
        file_hash: String,
    },
    Normal,
    TemplateInput {
        template_index: usize,
        param_keys: Vec<String>,
        current_param: usize,
        values: Vec<String>,
        input: String,
    },
    TemplateCreate {
        command: String,
        cursor_pos: usize,
        selection_start: Option<usize>,
        placeholders: Vec<TemplatePlaceholder>,
        phase: CreatePhase,
        input: String,
    },
}

struct AppContext {
    list: FilterableListState,
    run_command: Option<String>,
    exit_next: bool,
    view_mode: ViewMode,
    all_commands: Vec<String>,
    saved_commands: SavedCommands,
    app_state: AppState,
}

// ─── Trust prompt UI ─────────────────────────────────────────────────────────

fn ui_trust_prompt(frame: &mut Frame, file_path: &str) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Percentage(100),
            Constraint::Min(5),
            Constraint::Min(1),
        ])
        .split(frame.area());

    let warning_style = Style::default()
        .fg(ratatui::style::Color::Yellow)
        .add_modifier(Modifier::BOLD);
    let path_style = Style::default().fg(ratatui::style::Color::Cyan);

    let text = vec![
        Line::from(Span::styled(
            "This folder contains a .commander.json file.",
            warning_style,
        )),
        Line::from(Span::styled(file_path, path_style)),
        Line::from(""),
        Line::from("Do you trust the authors of this file?"),
    ];

    frame.render_widget(
        Paragraph::new(text).block(
            Block::new()
                .borders(Borders::ALL)
                .title("Trust project commands?"),
        ),
        layout[1],
    );

    let key_style = Style::default()
        .add_modifier(Modifier::BOLD)
        .fg(ratatui::style::Color::White);
    let desc_style = Style::default().fg(ratatui::style::Color::DarkGray);
    let help_line = Line::from(vec![
        Span::styled("y", key_style),
        Span::styled(" trust & load  ", desc_style),
        Span::styled("n", key_style),
        Span::styled(" skip  ", desc_style),
        Span::styled("ctrl+q", key_style),
        Span::styled(" quit", desc_style),
    ]);
    frame.render_widget(
        Paragraph::new(help_line)
            .style(Style::default().fg(ratatui::style::Color::Gray))
            .block(Block::new().borders(Borders::NONE)),
        layout[2],
    );
}

// ─── Normal mode UI ──────────────────────────────────────────────────────────

fn ui_normal(frame: &mut Frame, app_context: &mut AppContext) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Percentage(100),
            Constraint::Min(3),
            Constraint::Min(1),
        ])
        .split(frame.area());

    let items: Vec<ListItem> = app_context
        .list
        .get_filtered_items()
        .iter()
        .map(|item| {
            if app_context.saved_commands.find_template(item).is_some() {
                ListItem::new(item.to_string())
                    .style(Style::default().fg(ratatui::style::Color::Magenta))
            } else if app_context.saved_commands.contains(item) {
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

    let key_style = Style::default()
        .add_modifier(Modifier::BOLD)
        .fg(ratatui::style::Color::White);
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
        Span::styled("ctrl+t", key_style),
        Span::styled(" template  ", desc_style),
        Span::styled("ctrl+v", key_style),
        Span::styled(" view  ", desc_style),
        Span::styled(mode_str, Style::default().fg(ratatui::style::Color::Cyan)),
    ]);
    frame.render_widget(
        Paragraph::new(help_line)
            .style(Style::default().fg(ratatui::style::Color::Gray))
            .block(Block::new().borders(Borders::NONE)),
        layout[2],
    );
}

// ─── TemplateInput mode UI ───────────────────────────────────────────────────

fn ui_template_input(
    frame: &mut Frame,
    app_context: &mut AppContext,
    param_keys: &[String],
    current_param: usize,
    values: &[String],
    input: &str,
    template_index: usize,
) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Percentage(100),
            Constraint::Min(3),
            Constraint::Min(3),
            Constraint::Min(1),
        ])
        .split(frame.area());

    // Show the template command with placeholders highlighted
    let templates = app_context.saved_commands.templates();
    let template = &templates[template_index];
    let mut cmd_spans: Vec<Span> = Vec::new();
    let cmd = &template.command;
    let mut pos = 0;
    for (i, key) in param_keys.iter().enumerate() {
        let placeholder = format!("<{}>", key);
        if let Some(idx) = cmd[pos..].find(&placeholder) {
            let abs_idx = pos + idx;
            if abs_idx > pos {
                cmd_spans.push(Span::raw(&cmd[pos..abs_idx]));
            }
            let style = if i < values.len() {
                // Already filled — show the value
                Style::default().fg(ratatui::style::Color::Green)
            } else if i == current_param {
                Style::default()
                    .fg(ratatui::style::Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(ratatui::style::Color::DarkGray)
            };
            let display = if i < values.len() {
                values[i].clone()
            } else {
                placeholder.clone()
            };
            cmd_spans.push(Span::styled(display, style));
            pos = abs_idx + placeholder.len();
        }
    }
    if pos < cmd.len() {
        cmd_spans.push(Span::raw(&cmd[pos..]));
    }

    frame.render_widget(
        Paragraph::new(Line::from(cmd_spans))
            .block(Block::new().borders(Borders::ALL).title("Template")),
        layout[1],
    );

    // Input prompt for current placeholder
    let key = &param_keys[current_param];
    let param = template.params.get(key);
    let prompt = if let Some(p) = param {
        format!(
            "<{}> — {} (e.g. {}): {}",
            key, p.description, p.example, input
        )
    } else {
        format!("<{}>: {}", key, input)
    };
    frame.render_widget(
        Paragraph::new(prompt).block(Block::new().borders(Borders::ALL).title("Input")),
        layout[2],
    );

    // Help bar
    let key_style = Style::default()
        .add_modifier(Modifier::BOLD)
        .fg(ratatui::style::Color::White);
    let desc_style = Style::default().fg(ratatui::style::Color::DarkGray);
    let help_line = Line::from(vec![
        Span::styled("enter", key_style),
        Span::styled(" confirm  ", desc_style),
        Span::styled("esc", key_style),
        Span::styled(" cancel", desc_style),
    ]);
    frame.render_widget(
        Paragraph::new(help_line)
            .style(Style::default().fg(ratatui::style::Color::Gray))
            .block(Block::new().borders(Borders::NONE)),
        layout[3],
    );
}

// ─── TemplateCreate mode UI ──────────────────────────────────────────────────

fn ui_template_create(
    frame: &mut Frame,
    command: &str,
    cursor_pos: usize,
    selection_start: Option<usize>,
    phase: &CreatePhase,
    input: &str,
    placeholders: &[TemplatePlaceholder],
) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Percentage(100),
            Constraint::Min(3),
            Constraint::Min(3),
            Constraint::Min(1),
        ])
        .split(frame.area());

    // Render the command with cursor and selection
    let chars: Vec<char> = command.chars().collect();
    let cursor = cursor_pos.min(chars.len());

    match phase {
        CreatePhase::Selecting => {
            let mut spans: Vec<Span> = Vec::new();

            let (sel_start, sel_end) = if let Some(ss) = selection_start {
                let s = ss.min(chars.len());
                let e = cursor;
                (s.min(e), s.max(e))
            } else {
                (cursor, cursor)
            };

            for (i, ch) in chars.iter().enumerate() {
                if selection_start.is_some() && i >= sel_start && i < sel_end {
                    // Selected text
                    spans.push(Span::styled(
                        ch.to_string(),
                        Style::default()
                            .bg(ratatui::style::Color::Yellow)
                            .fg(ratatui::style::Color::Black),
                    ));
                } else if i == cursor {
                    // Cursor position
                    spans.push(Span::styled(
                        ch.to_string(),
                        Style::default()
                            .bg(ratatui::style::Color::White)
                            .fg(ratatui::style::Color::Black),
                    ));
                } else {
                    spans.push(Span::raw(ch.to_string()));
                }
            }
            // If cursor is at end, show a block cursor
            if cursor >= chars.len() {
                spans.push(Span::styled(
                    " ",
                    Style::default().bg(ratatui::style::Color::White),
                ));
            }

            frame.render_widget(
                Paragraph::new(Line::from(spans)).block(
                    Block::new()
                        .borders(Borders::ALL)
                        .title("Select text to make placeholder"),
                ),
                layout[1],
            );

            // Show existing placeholders
            let placeholder_info: Vec<String> = placeholders
                .iter()
                .map(|p| format!("<{}> {} — {}", p.key, p.example, p.description))
                .collect();
            let info_text = if placeholder_info.is_empty() {
                "Move cursor with ←/→, press Enter to start selection".to_string()
            } else {
                placeholder_info.join("  |  ")
            };
            frame.render_widget(
                Paragraph::new(info_text)
                    .block(Block::new().borders(Borders::ALL).title("Placeholders")),
                layout[2],
            );
        }
        CreatePhase::EnteringExample => {
            // Show the command
            frame.render_widget(
                Paragraph::new(command).block(Block::new().borders(Borders::ALL).title("Template")),
                layout[1],
            );

            let next_key = placeholders.len() + 1;
            frame.render_widget(
                Paragraph::new(format!("Example for <{}>: {}", next_key, input))
                    .block(Block::new().borders(Borders::ALL).title("Example")),
                layout[2],
            );
        }
        CreatePhase::EnteringDescription => {
            frame.render_widget(
                Paragraph::new(command).block(Block::new().borders(Borders::ALL).title("Template")),
                layout[1],
            );

            let next_key = placeholders.len() + 1;
            frame.render_widget(
                Paragraph::new(format!("Description for <{}>: {}", next_key, input))
                    .block(Block::new().borders(Borders::ALL).title("Description")),
                layout[2],
            );
        }
    }

    // Help bar
    let key_style = Style::default()
        .add_modifier(Modifier::BOLD)
        .fg(ratatui::style::Color::White);
    let desc_style = Style::default().fg(ratatui::style::Color::DarkGray);
    let help_spans = match phase {
        CreatePhase::Selecting => {
            if selection_start.is_some() {
                vec![
                    Span::styled("enter", key_style),
                    Span::styled(" confirm selection  ", desc_style),
                    Span::styled("esc", key_style),
                    Span::styled(" cancel selection", desc_style),
                ]
            } else if placeholders.is_empty() {
                vec![
                    Span::styled("←/→", key_style),
                    Span::styled(" move  ", desc_style),
                    Span::styled("enter", key_style),
                    Span::styled(" start selection  ", desc_style),
                    Span::styled("esc", key_style),
                    Span::styled(" cancel", desc_style),
                ]
            } else {
                vec![
                    Span::styled("←/→", key_style),
                    Span::styled(" move  ", desc_style),
                    Span::styled("enter", key_style),
                    Span::styled(" start selection  ", desc_style),
                    Span::styled("esc", key_style),
                    Span::styled(" save template", desc_style),
                ]
            }
        }
        _ => vec![
            Span::styled("enter", key_style),
            Span::styled(" confirm  ", desc_style),
            Span::styled("esc", key_style),
            Span::styled(" cancel", desc_style),
        ],
    };
    frame.render_widget(
        Paragraph::new(Line::from(help_spans))
            .style(Style::default().fg(ratatui::style::Color::Gray))
            .block(Block::new().borders(Borders::NONE)),
        layout[3],
    );
}

// ─── Main UI dispatch ────────────────────────────────────────────────────────

fn ui(frame: &mut Frame, app_context: &mut AppContext) {
    match &app_context.app_state {
        AppState::TrustPrompt { file_path, .. } => {
            let file_path = file_path.clone();
            ui_trust_prompt(frame, &file_path);
        }
        AppState::Normal => ui_normal(frame, app_context),
        AppState::TemplateInput {
            template_index,
            param_keys,
            current_param,
            values,
            input,
            ..
        } => {
            let template_index = *template_index;
            let param_keys = param_keys.clone();
            let current_param = *current_param;
            let values = values.clone();
            let input = input.clone();
            ui_template_input(
                frame,
                app_context,
                &param_keys,
                current_param,
                &values,
                &input,
                template_index,
            );
        }
        AppState::TemplateCreate {
            command,
            cursor_pos,
            selection_start,
            placeholders,
            phase,
            input,
        } => {
            // Copy what we need to avoid borrow issues
            let command = command.clone();
            let cursor_pos = *cursor_pos;
            let selection_start = *selection_start;
            let input = input.clone();

            // We need to pass phase and placeholders by ref, but we can't borrow
            // from app_context while passing frame mutably. So we extract what we need.
            let phase_is_selecting = matches!(phase, CreatePhase::Selecting);
            let phase_is_example = matches!(phase, CreatePhase::EnteringExample);
            let placeholder_data: Vec<TemplatePlaceholder> = placeholders
                .iter()
                .map(|p| TemplatePlaceholder {
                    key: p.key.clone(),
                    example: p.example.clone(),
                    description: p.description.clone(),
                })
                .collect();

            let phase = if phase_is_selecting {
                CreatePhase::Selecting
            } else if phase_is_example {
                CreatePhase::EnteringExample
            } else {
                CreatePhase::EnteringDescription
            };

            ui_template_create(
                frame,
                &command,
                cursor_pos,
                selection_start,
                &phase,
                &input,
                &placeholder_data,
            );
        }
    }
}

// ─── Event handlers ──────────────────────────────────────────────────────────

fn event_handler_trust_prompt(app_context: &mut AppContext, key: event::KeyEvent) {
    if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('q') {
        app_context.exit_next = true;
        return;
    }
    match key.code {
        KeyCode::Char('y') => {
            if let AppState::TrustPrompt {
                file_path,
                file_contents,
                file_hash,
            } = std::mem::replace(&mut app_context.app_state, AppState::Normal)
            {
                let mut store = TrustStore::load();
                store.trust(&file_path, &file_hash);

                let saved = SavedCommands::load_from_string(&file_contents);
                // Merge saved commands into the list
                for cmd in saved.commands() {
                    if !app_context.all_commands.contains(cmd) {
                        app_context.all_commands.push(cmd.clone());
                    }
                }
                for tmpl in saved.templates() {
                    if !app_context.all_commands.contains(&tmpl.command) {
                        app_context.all_commands.push(tmpl.command.clone());
                    }
                }
                app_context
                    .list
                    .swap_items(app_context.all_commands.clone());
                app_context.saved_commands = saved;
            }
        }
        KeyCode::Char('n') => {
            app_context.app_state = AppState::Normal;
        }
        _ => {}
    }
}

fn event_handler_normal(app_context: &mut AppContext, key: event::KeyEvent) {
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
            KeyCode::Char('v') => match app_context.view_mode {
                ViewMode::All => {
                    let mut items: Vec<String> = app_context.saved_commands.commands().to_vec();
                    for t in app_context.saved_commands.templates() {
                        if !items.contains(&t.command) {
                            items.push(t.command.clone());
                        }
                    }
                    app_context.list.swap_items(items);
                    app_context.view_mode = ViewMode::SavedOnly;
                }
                ViewMode::SavedOnly => {
                    app_context
                        .list
                        .swap_items(app_context.all_commands.clone());
                    app_context.view_mode = ViewMode::All;
                }
            },
            KeyCode::Char('t') => {
                if let Some(item) = app_context.list.get_current_item() {
                    let len = item.chars().count();
                    app_context.app_state = AppState::TemplateCreate {
                        command: item,
                        cursor_pos: 0,
                        selection_start: None,
                        placeholders: Vec::new(),
                        phase: CreatePhase::Selecting,
                        input: String::new(),
                    };
                    // Ensure cursor is at start, which is already 0
                    let _ = len;
                }
            }
            _ => {}
        }
    } else {
        match key.code {
            KeyCode::Char(c) => {
                app_context
                    .list
                    .set_filter(format!("{}{}", app_context.list.get_filter(), c))
            }
            KeyCode::Backspace => {
                let mut str = app_context.list.get_filter().to_string();
                str.pop();
                app_context.list.set_filter(str);
            }
            KeyCode::Up => {
                app_context.list.next();
            }
            KeyCode::Down => {
                app_context.list.previous();
            }
            KeyCode::Enter => {
                if let Some(item) = app_context.list.get_current_item() {
                    // Check if this matches a template
                    if let Some(tmpl_idx) = app_context
                        .saved_commands
                        .templates()
                        .iter()
                        .position(|t| t.command == item)
                    {
                        let param_keys =
                            app_context.saved_commands.templates()[tmpl_idx].placeholder_names();
                        if param_keys.is_empty() {
                            // No placeholders, just run it
                            app_context.run_command = Some(item);
                            app_context.exit_next = true;
                        } else {
                            app_context.app_state = AppState::TemplateInput {
                                template_index: tmpl_idx,
                                param_keys,
                                current_param: 0,
                                values: Vec::new(),
                                input: String::new(),
                            };
                        }
                    } else {
                        app_context.run_command = Some(item);
                        app_context.exit_next = true;
                    }
                }
            }
            _ => {}
        }
    }
}

fn event_handler_template_input(app_context: &mut AppContext, key: event::KeyEvent) {
    // Extract state — we need to take ownership temporarily
    if let AppState::TemplateInput {
        template_index,
        ref param_keys,
        ref mut current_param,
        ref mut values,
        ref mut input,
    } = app_context.app_state
    {
        match key.code {
            KeyCode::Esc => {
                app_context.app_state = AppState::Normal;
            }
            KeyCode::Enter => {
                values.push(input.clone());
                *input = String::new();
                *current_param += 1;
                if *current_param >= param_keys.len() {
                    // All params filled — resolve and run
                    let mut value_map = HashMap::new();
                    for (i, key) in param_keys.iter().enumerate() {
                        value_map.insert(key.clone(), values[i].clone());
                    }
                    let resolved =
                        app_context.saved_commands.templates()[template_index].resolve(&value_map);
                    app_context.run_command = Some(resolved);
                    app_context.exit_next = true;
                    app_context.app_state = AppState::Normal;
                }
            }
            KeyCode::Backspace => {
                input.pop();
            }
            KeyCode::Char(c) => {
                input.push(c);
            }
            _ => {}
        }
    }
}

fn event_handler_template_create(app_context: &mut AppContext, key: event::KeyEvent) {
    if let AppState::TemplateCreate {
        ref mut command,
        ref mut cursor_pos,
        ref mut selection_start,
        ref mut placeholders,
        ref mut phase,
        ref mut input,
    } = app_context.app_state
    {
        match phase {
            CreatePhase::Selecting => match key.code {
                KeyCode::Left => {
                    if *cursor_pos > 0 {
                        *cursor_pos -= 1;
                    }
                }
                KeyCode::Right => {
                    let len = command.chars().count();
                    if *cursor_pos < len {
                        *cursor_pos += 1;
                    }
                }
                KeyCode::Enter => {
                    if let Some(ss) = *selection_start {
                        // Finalize selection
                        let chars: Vec<char> = command.chars().collect();
                        let start = ss.min(*cursor_pos);
                        let end = ss.max(*cursor_pos);
                        if start < end {
                            let next_key = placeholders.len() + 1;
                            let placeholder = format!("<{}>", next_key);
                            let before: String = chars[..start].iter().collect();
                            let after: String = chars[end..].iter().collect();
                            *command = format!("{}{}{}", before, placeholder, after);
                            *cursor_pos = start + placeholder.chars().count();
                            *selection_start = None;
                            *phase = CreatePhase::EnteringExample;
                            *input = String::new();
                        } else {
                            // Empty selection, just clear
                            *selection_start = None;
                        }
                    } else {
                        // Begin selection at cursor
                        *selection_start = Some(*cursor_pos);
                    }
                }
                KeyCode::Esc => {
                    if selection_start.is_some() {
                        *selection_start = None;
                    } else if !placeholders.is_empty() {
                        // Save template and return to Normal
                        let mut params = HashMap::new();
                        for p in placeholders.iter() {
                            params.insert(
                                p.key.clone(),
                                TemplateParam {
                                    example: p.example.clone(),
                                    description: p.description.clone(),
                                },
                            );
                        }
                        let template = Template {
                            command: command.clone(),
                            params,
                        };
                        app_context.saved_commands.add_template(template);

                        // Add the template command to the all_commands list
                        if !app_context.all_commands.contains(command) {
                            app_context.all_commands.push(command.clone());
                            app_context
                                .list
                                .swap_items(app_context.all_commands.clone());
                        }

                        app_context.app_state = AppState::Normal;
                    } else {
                        // Cancel without saving
                        app_context.app_state = AppState::Normal;
                    }
                }
                _ => {}
            },
            CreatePhase::EnteringExample => match key.code {
                KeyCode::Enter => {
                    let example = input.clone();
                    *input = String::new();
                    // Store example temporarily — we'll push the full placeholder after description
                    // Actually, stash example in input field trick: transition to description
                    // We need to store the example somewhere. Let's use a temp approach:
                    // push a partial placeholder and fill description next
                    let next_key = (placeholders.len() + 1).to_string();
                    placeholders.push(TemplatePlaceholder {
                        key: next_key,
                        example,
                        description: String::new(), // filled next
                    });
                    *phase = CreatePhase::EnteringDescription;
                }
                KeyCode::Backspace => {
                    input.pop();
                }
                KeyCode::Char(c) => {
                    input.push(c);
                }
                KeyCode::Esc => {
                    // Cancel this placeholder — revert command
                    // For simplicity, just go back to Selecting without adding
                    *phase = CreatePhase::Selecting;
                    *input = String::new();
                }
                _ => {}
            },
            CreatePhase::EnteringDescription => match key.code {
                KeyCode::Enter => {
                    let description = input.clone();
                    *input = String::new();
                    // Update the last placeholder's description
                    if let Some(last) = placeholders.last_mut() {
                        last.description = description;
                    }
                    *phase = CreatePhase::Selecting;
                }
                KeyCode::Backspace => {
                    input.pop();
                }
                KeyCode::Char(c) => {
                    input.push(c);
                }
                KeyCode::Esc => {
                    // Cancel — remove the partial placeholder
                    placeholders.pop();
                    *phase = CreatePhase::Selecting;
                    *input = String::new();
                }
                _ => {}
            },
        }
    }
}

fn event_handler(app_context: &mut AppContext) -> io::Result<()> {
    if event::poll(std::time::Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press {
                match &app_context.app_state {
                    AppState::TrustPrompt { .. } => {
                        event_handler_trust_prompt(app_context, key)
                    }
                    AppState::Normal => event_handler_normal(app_context, key),
                    AppState::TemplateInput { .. } => {
                        event_handler_template_input(app_context, key)
                    }
                    AppState::TemplateCreate { .. } => {
                        event_handler_template_create(app_context, key)
                    }
                }
            }
        }
    }
    Ok(())
}

fn copy_to_clipboard(text: &str) {
    if let Ok(mut child) = Command::new("pbcopy").stdin(Stdio::piped()).spawn() {
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(text.as_bytes());
        }
        let _ = child.wait();
    }
}

fn run_main_term_loop(commands: Vec<String>) -> Result<Option<String>> {
    let mut terminal = Terminal::new(CrosstermBackend::new(stderr()))?;
    terminal.clear()?;

    let (saved_commands, initial_state) = match check_trust() {
        TrustStatus::NoFile => (SavedCommands::load(), AppState::Normal),
        TrustStatus::Trusted { contents } => {
            (SavedCommands::load_from_string(&contents), AppState::Normal)
        }
        TrustStatus::Untrusted {
            path,
            contents,
            hash,
        } => (
            SavedCommands::default(),
            AppState::TrustPrompt {
                file_path: path,
                file_contents: contents,
                file_hash: hash,
            },
        ),
    };

    // Merge saved commands and template commands into the list, deduplicating
    let mut all_commands = commands;
    for cmd in saved_commands.commands() {
        if !all_commands.contains(cmd) {
            all_commands.push(cmd.clone());
        }
    }
    for tmpl in saved_commands.templates() {
        if !all_commands.contains(&tmpl.command) {
            all_commands.push(tmpl.command.clone());
        }
    }

    let mut app_context = AppContext {
        list: FilterableListState::new(all_commands.clone()),
        exit_next: false,
        run_command: None,
        view_mode: ViewMode::All,
        all_commands,
        saved_commands,
        app_state: initial_state,
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
