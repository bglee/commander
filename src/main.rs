use crossterm::{
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand, event::{self, Event, KeyModifiers, KeyCode},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Corner},
    widgets::{List, ListItem, ListState, Paragraph, Borders, Block},
    Frame, Terminal,
};
use std::{io::{self, stdout}, env} ;

mod command_list;
mod filter;

struct FilterableListOptions<'a> {
    all_item: Vec<&'a str>,
    filter: String,
    state: ListState,
}

impl<'a> FilterableListOptions<'a> {
    fn new() -> FilterableListOptions<'a> {
        FilterableListOptions {
            all_item: command_list::full(),
            filter: "".to_string(),
            state: ListState::default(),
        }
    }
    pub fn set_filter(&mut self, new_filer: String) {
        self.filter = new_filer;
        self.reset_select();
    }
    pub fn get_filter(&self) -> &str {
        &self.filter
    }
    pub fn get_filtered_items(&self) -> Vec<&str> {
        filter::fuzzy_filter(&self.filter, &self.all_item)
    }
    pub fn reset_select(&mut self) {
        let len = self.get_filtered_items().len();
        if len != 0 {
           self.state.select(Some(len-1));
        }else{
            self.state.select(None);
        }
    }
    pub fn next(&mut self) {
        let next_index = match self.state.selected() {
            Some(current_index) => {
                if current_index >= self.get_filtered_items().len() {
                    0
                } else {
                    current_index + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(next_index));
    }
    pub fn previous(&mut self) {
        let previous_index = match self.state.selected() {
            Some(current_index) => {
                if current_index == 0 {
                    self.get_filtered_items().len() - 1
                } else {
                    current_index - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(previous_index));
    }

    pub fn get_current_item(&self) -> Option<String> {
        match self.state.selected() {
            Some(index) => Some(self.get_filtered_items()[index].to_owned()),
            None => None,
        }
    }
}

struct AppContext<'a> {
    list: FilterableListOptions<'a>,
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
    let list = List::new(items).highlight_symbol("> ").start_corner(Corner::BottomLeft);
    frame.render_stateful_widget(list.block(Block::new().borders(Borders::NONE)), layout[0], &mut app_context.list.state);


    frame.render_widget(Paragraph::new(format!(">{}", app_context.list.get_filter()))
        .block(Block::new().borders(Borders::ALL)), layout[1]);
}

fn event_handler(app_context: &mut AppContext) -> io::Result<()> {

        if event::poll(std::time::Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {

        if key.kind == event::KeyEventKind::Press {
            if key.modifiers==KeyModifiers::CONTROL {
                match key.code {
                    KeyCode::Char('c') => {
                        app_context.exit_next=true;
                    }
                    KeyCode::Char('k') => {
                        app_context.list.next();
                    }
                    KeyCode::Char('j') => {
                        app_context.list.previous();
                    }

                    _ => return Ok(()),
                }
            }
            else {
               match key.code {
                   KeyCode::Char(c) => {
                    app_context.list.set_filter(format!("{}{}",app_context.list.get_filter(), c)) 
                   }
                   KeyCode::Backspace => {
                    let mut str = app_context.list.get_filter().to_string();
                    str.pop();
                    app_context.list.set_filter(format!("{}",str)); 
                   }
                   KeyCode::Enter => {
                     app_context.run_command=app_context.list.get_current_item();    
                     app_context.exit_next=true;
                   }
                   _=>{}
               }
            }
        }
        }}
    Ok(())
}

fn run_it(command: String) {
    env::set_var("COMMANDER_OUTPUT", format!("<<{}>>", command));
}

fn main() -> io::Result<()> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    let mut app_context = AppContext {
        list: FilterableListOptions::new(),
        exit_next: false,
        run_command: None
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

    if let Some(command) = app_context.run_command {
        run_it(command)
    }

    Ok(())
}
