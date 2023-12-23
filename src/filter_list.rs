use ratatui::widgets::ListState;

use crate::{command_list, filter};

pub struct FilterableListState<'a> {
    all_item: Vec<&'a str>,
    filter: String,
    pub list_state: ListState,
}

impl<'a> FilterableListState<'a> {
    pub fn new() -> FilterableListState<'a> {
        FilterableListState {
            all_item: command_list::full(),
            filter: "".to_string(),
            list_state: ListState::default(),
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
            self.list_state.select(Some(len - 1));
        } else {
            self.list_state.select(None);
        }
    }
    pub fn next(&mut self) {
        let next_index = match self.list_state.selected() {
            Some(current_index) => {
                if current_index >= self.get_filtered_items().len() {
                    0
                } else {
                    current_index + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(next_index));
    }
    pub fn previous(&mut self) {
        let previous_index = match self.list_state.selected() {
            Some(current_index) => {
                if current_index == 0 {
                    self.get_filtered_items().len() - 1
                } else {
                    current_index - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(previous_index));
    }

    pub fn get_current_item(&self) -> Option<String> {
        match self.list_state.selected() {
            Some(index) => Some(self.get_filtered_items()[index].to_owned()),
            None => None,
        }
    }
}
