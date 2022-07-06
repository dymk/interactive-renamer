use std::{cell::RefCell, rc::Rc};

use tui::widgets::ListState;

use crate::{
    dao::Dao,
    utils::{compute_prefix, dir_name, join_path},
};

use super::{
    app_transition::AppTransition,
    mapping_state::{FileMapping, MappedDir, MappingState},
    AppState,
};

pub struct SelectingInputState {
    dao: Rc<RefCell<Dao>>,
    in_dir_path: String,
    out_dir_path: String,
    mapping_states: Vec<MappingState>,
    selected_row_idx: usize,
    list_state: RefCell<ListState>,
    logs: Vec<String>,
}

impl SelectingInputState {
    pub fn new(
        dao: Rc<RefCell<Dao>>,
        in_dir_path: String,
        out_dir_path: String,
    ) -> SelectingInputState {
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        let mut ret = SelectingInputState {
            dao,
            in_dir_path,
            out_dir_path,
            mapping_states: vec![],
            selected_row_idx: 0,
            list_state: RefCell::new(list_state),
            logs: vec![],
        };
        ret.update_mappings_cache();
        ret
    }

    pub fn list_state(&self) -> &RefCell<ListState> {
        return &self.list_state;
    }

    pub fn in_dir(&self) -> &str {
        &self.in_dir_path
    }

    pub fn out_dir(&self) -> &str {
        &self.out_dir_path
    }

    fn update_mappings_cache(&mut self) {
        self.mapping_states = std::fs::read_dir(&self.in_dir_path)
            .unwrap()
            .filter(|entry| entry.as_ref().unwrap().metadata().unwrap().is_dir())
            .map(|entry| entry.unwrap().path().to_string_lossy().to_string())
            .map(|in_path| {
                if let Some(mapped_dir) = self
                    .dao
                    .borrow()
                    .get_mapped_dir_by_in_path(in_path.as_str())
                {
                    MappingState::HasMapping { mapped_dir }
                } else {
                    MappingState::Unmapped { in_path }
                }
            })
            .collect()
    }

    pub fn commit_mapping(&mut self, mapping_idx: usize, new_mapped_dir: MappedDir) {
        let old_mapping = &self.mapping_states[mapping_idx];
        if let MappingState::HasMapping { mapped_dir } = old_mapping {
            if mapped_dir.configs_eq(&new_mapped_dir) {
                self.add_log(format!("no change for `{}`", mapped_dir.in_dir_name()));
                return;
            }
        }

        {
            // remove the directory for the old mapping, if it exists
            if let MappingState::HasMapping { mapped_dir } = old_mapping {
                if let Some(out_dir_name) = mapped_dir.out_dir_name() {
                    let old_out_dir_path = join_path(&self.out_dir_path, &out_dir_name);
                    self.add_log(format!("delete dir `{}`", &old_out_dir_path));
                    std::fs::remove_dir_all(old_out_dir_path).unwrap();
                }
            }
        }

        {
            // create the new directory
            if let Some(out_dir_name) = new_mapped_dir.out_dir_name() {
                let new_out_dir_path = join_path(&self.out_dir_path, &out_dir_name);
                self.add_log(format!("create dir `{}`", &new_out_dir_path));
                std::fs::create_dir(&new_out_dir_path).unwrap();

                for file_mapping in new_mapped_dir.file_mappings().iter() {
                    if let FileMapping::MappedTo { from_name, to_name } = file_mapping {
                        let in_file_path =
                            format!("{}/{}", new_mapped_dir.in_dir_path(), &from_name);

                        let out_file_path = join_path(&new_out_dir_path, &to_name);

                        let in_file_rel_path =
                            compute_prefix(dir_name(&in_file_path), dir_name(&out_file_path))
                                + &from_name;

                        self.add_log(format!("in file:  {}", in_file_path));
                        self.add_log(format!("out file: {}", out_file_path));
                        self.add_log(format!("link:     {}", in_file_rel_path));

                        std::os::unix::fs::symlink(in_file_rel_path, out_file_path).unwrap();
                    }
                }
            }
        }

        self.dao.borrow().upsert_mapped_dir(&new_mapped_dir);
        self.mapping_states[mapping_idx] = MappingState::HasMapping {
            mapped_dir: new_mapped_dir,
        };
    }

    pub fn mappings(&self) -> &Vec<MappingState> {
        &self.mapping_states
    }

    fn add_log(&mut self, log: String) {
        self.logs.push(log);
    }

    pub fn get_logs(&self) -> &Vec<String> {
        return &self.logs;
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
        AppTransition::StartConfiguringIdx(self.selected_row_idx)
    }

    fn requesting_input(&self) -> bool {
        false
    }
}
