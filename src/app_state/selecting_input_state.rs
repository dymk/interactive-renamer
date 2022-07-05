use std::{cell::RefCell, rc::Rc};

use tui::widgets::ListState;

use crate::dao::Dao;

use super::{
    app_transition::AppTransition,
    mapping_state::{MappedDir, MappingState},
    AppState,
};

pub struct SelectingInputState {
    dao: Rc<RefCell<Dao>>,
    in_dir: String,
    out_dir: String,
    mapping_states: Vec<MappingState>,
    selected_row_idx: usize,
    list_state: RefCell<ListState>,
}

impl SelectingInputState {
    pub fn new(dao: Rc<RefCell<Dao>>, in_dir: String, out_dir: String) -> SelectingInputState {
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        let mut ret = SelectingInputState {
            dao,
            in_dir,
            out_dir,
            mapping_states: vec![],
            selected_row_idx: 0,
            list_state: RefCell::new(list_state),
        };
        ret.update_mappings_cache();
        ret
    }

    pub fn list_state(&self) -> &RefCell<ListState> {
        return &self.list_state;
    }

    pub fn in_dir(&self) -> &str {
        &self.in_dir
    }

    pub fn out_dir(&self) -> &str {
        &self.out_dir
    }

    fn update_mappings_cache(&mut self) {
        self.mapping_states = std::fs::read_dir(&self.in_dir)
            .unwrap()
            .filter(|entry| entry.as_ref().unwrap().metadata().unwrap().is_dir())
            .map(|entry| entry.unwrap().path().to_string_lossy().to_string())
            .map(|path| path.as_str()[self.in_dir.len()..].to_string())
            .map(|in_path| {
                if let Some(mapped_dir) = self
                    .dao
                    .borrow()
                    .get_mapped_dir_by_in_path(in_path.as_str())
                {
                    MappingState::HasMapping(mapped_dir)
                } else {
                    MappingState::Unmapped(in_path)
                }
            })
            .collect()
    }

    pub fn set_mapping(&mut self, mapping_idx: usize, mapped_dir: MappedDir) {
        self.dao.borrow().upsert_mapped_dir(&mapped_dir);
        self.mapping_states[mapping_idx] = MappingState::HasMapping(mapped_dir);
    }

    pub fn mappings(&self) -> &Vec<MappingState> {
        &self.mapping_states
    }
}

impl AppState for SelectingInputState {
    fn on_up(&mut self) -> AppTransition {
        if self.selected_row_idx > 0 {
            self.selected_row_idx -= 1;
        }
        self.list_state
            .get_mut()
            .select(Some(self.selected_row_idx));
        AppTransition::None
    }

    fn on_down(&mut self) -> AppTransition {
        if self.selected_row_idx < self.mappings().len() - 1 {
            self.selected_row_idx += 1;
        }
        self.list_state
            .get_mut()
            .select(Some(self.selected_row_idx));
        AppTransition::None
    }

    fn on_enter(&mut self) -> AppTransition {
        let mapping = self.mappings().get(self.selected_row_idx).unwrap();
        AppTransition::StartConfiguringIdx(self.selected_row_idx)
    }

    fn requesting_input(&self) -> bool {
        false
    }
}
