use std::{cell::RefCell, rc::Rc};

use crate::app_state::{
    app_transition::AppTransition, configure_mapping_state::ConfigureMappingState,
    selecting_input_state::SelectingInputState, AppState,
};

pub struct App {
    db: Rc<RefCell<sqlite::Connection>>,
    pub selecting_input_state: SelectingInputState,
    pub configure_mapping_state: Option<ConfigureMappingState>,
}

impl App {
    pub fn new(db_path: &str, in_dir: &str, out_dir: &str) -> App {
        let db = Rc::new(RefCell::new(sqlite::open(db_path).unwrap()));
        db.borrow()
            .execute(
                r"
            DROP TABLE IF EXISTS dir_mappings;
        ",
            )
            .unwrap();

        db.borrow()
            .execute(
                r"
            CREATE TABLE IF NOT EXISTS dir_mappings (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                in_path TEXT,

                in_dir_finder_regex TEXT,
                in_dir_replacer TEXT,
                in_file_finder_regex TEXT,
                in_file_replacer TEXT,

                file_ext_filter TEXT
            );
        ",
            )
            .unwrap();

        App {
            db: db.clone(),
            selecting_input_state: SelectingInputState::new(
                db.clone(),
                in_dir.to_string(),
                out_dir.to_string(),
            ),
            configure_mapping_state: None,
        }
    }

    fn current_state_mut(&mut self) -> &mut dyn AppState {
        if let Some(cms) = self.configure_mapping_state.as_mut() {
            return cms;
        }
        return &mut self.selecting_input_state;
    }
    fn current_state(&self) -> &dyn AppState {
        if let Some(cms) = self.configure_mapping_state.as_ref() {
            return cms;
        }
        return &self.selecting_input_state;
    }

    pub fn on_up(&mut self) {
        let t = self.current_state_mut().on_up();
        self.handle_transition(t);
    }

    pub fn on_down(&mut self) {
        let t = self.current_state_mut().on_down();
        self.handle_transition(t);
    }

    pub fn on_tab(&mut self) {
        let t = self.current_state_mut().on_tab();
        self.handle_transition(t);
    }

    pub fn on_backtab(&mut self) {
        let t = self.current_state_mut().on_backtab();
        self.handle_transition(t);
    }

    pub fn on_enter(&mut self) {
        let t = self.current_state_mut().on_enter();
        self.handle_transition(t);
    }

    pub fn on_esc(&mut self) {
        let t = self.current_state_mut().on_esc();
        self.handle_transition(t)
    }

    pub fn on_bksp(&mut self) {
        let t = self.current_state_mut().on_bksp();
        self.handle_transition(t);
    }

    pub fn on_char(&mut self, c: char) {
        let t = self.current_state_mut().on_char(c);
        self.handle_transition(t);
    }

    pub fn requesting_input(&self) -> bool {
        self.current_state().requesting_input()
    }

    fn handle_transition(&mut self, transition: AppTransition) {
        match transition {
            AppTransition::None => {}
            AppTransition::StartConfiguringIdx(mapping_idx) => {
                let mapping = self
                    .selecting_input_state
                    .mappings()
                    .get(mapping_idx)
                    .unwrap();

                self.configure_mapping_state = Some(ConfigureMappingState {
                    mapping_idx,
                    focused_input_idx: None,
                    inputs: mapping.to_inputs(),
                })
            }
            AppTransition::AbortConfiguration => {
                self.configure_mapping_state = None;
            }
            AppTransition::CommitConfiguration => {}
        };
    }
}
