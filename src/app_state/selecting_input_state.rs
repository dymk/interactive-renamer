use std::{cell::RefCell, path::Path, rc::Rc};

use sqlite::{Connection, State};
use tui::widgets::ListState;

use super::{
    app_transition::AppTransition,
    mapping_state::{MappedDir, MappingState},
    AppState,
};

pub struct SelectingInputState {
    db: Rc<RefCell<Connection>>,
    in_dir: String,
    out_dir: String,
    mapping_states: Vec<MappingState>,
    selected_row_idx: usize,
    list_state: RefCell<ListState>,
}

impl SelectingInputState {
    pub fn new(
        db: Rc<RefCell<Connection>>,
        in_dir: String,
        out_dir: String,
    ) -> SelectingInputState {
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        SelectingInputState {
            db,
            in_dir,
            out_dir,
            mapping_states: vec![],
            selected_row_idx: 0,
            list_state: RefCell::new(list_state),
        }
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

    pub fn update_mappings_cache(&mut self) {
        self.mapping_states = std::fs::read_dir(&self.in_dir)
            .unwrap()
            .filter(|entry| entry.as_ref().unwrap().metadata().unwrap().is_dir())
            .map(|entry| entry.unwrap().path().to_string_lossy().to_string())
            .map(|path| path.as_str()[self.in_dir.len()..].to_string())
            .map(|in_path| {
                let db = self.db.borrow();
                let mut statement = db
                    .prepare(
                        r"
                    SELECT 
                        file_ext_filter,
                        in_dir_finder_regex, 
                        in_dir_replacer,
                        in_file_finder_regex,
                        in_file_replacer
                    FROM dir_mappings 
                    WHERE in_path = ?
                ",
                    )
                    .unwrap();

                statement.bind(1, in_path.as_str()).unwrap();

                if let State::Row = statement.next().unwrap() {
                    MappingState::HasMapping(MappedDir::from_inputs(
                        Path::new(&in_path).to_path_buf(),
                        [
                            statement.read::<String>(0).unwrap(),
                            statement.read::<String>(1).unwrap(),
                            statement.read::<String>(2).unwrap(),
                            statement.read::<String>(3).unwrap(),
                            statement.read::<String>(4).unwrap(),
                        ],
                    ))
                } else {
                    MappingState::Unmapped(in_path)
                }
            })
            .collect()
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
        // let mapping = self.mappings().get(self.in_dirs_selected_idx).unwrap();
        // let inputs = match mapping {
        //     MappingState::HasMapping(mapping) => mapping.to_inputs(),
        //     MappingState::Unmapped(_) => {
        //         [DEFAULT_FILE_FILTER, "(.+)", "$1", "(.+)", "$1"].map(ToString::to_string)
        //     }
        // };

        // let configure_mapping_state = ConfigureMappingState {
        //     mapping_idx: self.in_dirs_selected_idx,
        //     focused_input_idx: None,
        //     inputs,
        // };
        // self.app_state = AppState::ConfigureMapping(configure_mapping_state);

        AppTransition::StartConfiguringIdx(self.selected_row_idx)
    }

    fn requesting_input(&self) -> bool {
        false
    }
}
