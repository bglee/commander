use ratatui::widgets::ListState;

use crate::filter;

pub struct FilterableListState {
    all_items: Vec<String>,
    filter: String,
    pub list_state: ListState,
}

impl FilterableListState {
    pub fn new(all_items: Vec<String>) -> FilterableListState {
        FilterableListState {
            all_items,
            filter: "".to_string(),
            list_state: ListState::default(),
        }
    }
    pub fn set_filter(&mut self, new_filter: String) {
        self.filter = new_filter;
        self.reset_select();
    }
    pub fn get_filter(&self) -> &str {
        &self.filter
    }
    pub fn get_filtered_items(&self) -> Vec<&str> {
        let refs: Vec<&str> = self.all_items.iter().map(|s| s.as_str()).collect();
        filter::fuzzy_filter(&self.filter, &refs)
    }
    pub fn reset_select(&mut self) {
        let len = self.get_filtered_items().len();
        if len != 0 {
            self.list_state.select(Some(0));
        } else {
            self.list_state.select(None);
        }
    }
    pub fn next(&mut self) {
        let len = self.get_filtered_items().len();
        if len == 0 {
            self.list_state.select(None);
            return;
        }
        let next_index = match self.list_state.selected() {
            Some(current_index) => {
                if current_index >= len - 1 {
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
        let len = self.get_filtered_items().len();
        if len == 0 {
            self.list_state.select(None);
            return;
        }
        let previous_index = match self.list_state.selected() {
            Some(current_index) => {
                if current_index == 0 {
                    len - 1
                } else {
                    current_index - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(previous_index));
    }

    pub fn get_current_item(&self) -> Option<String> {
        self.list_state
            .selected()
            .map(|index| self.get_filtered_items()[index].to_owned())
    }

    pub fn swap_items(&mut self, new_items: Vec<String>) {
        self.all_items = new_items;
        self.reset_select();
    }
}
