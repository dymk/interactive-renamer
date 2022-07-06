use std::{cell::RefCell, rc::Rc};

use crate::{
    app_state::{
        app_transition::AppTransition, configure_mapping_state::ConfigureMappingState,
        selecting_input_state::SelectingInputState, AppState,
    },
    dao::Dao,
};

pub struct App {
    // dao: Rc<RefCell<Dao>>,
    pub selecting_input_state: SelectingInputState,
    pub configure_mapping_state: Option<ConfigureMappingState>,
}

impl App {
    pub fn new(db_path: &str, in_dir: &str, out_dir: &str) -> App {
        let dao = Rc::new(RefCell::new(Dao::new(db_path)));
        App {
            // dao: dao.clone(),
            selecting_input_state: SelectingInputState::new(
                dao.clone(),
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

                self.configure_mapping_state = Some(ConfigureMappingState::new(
                    mapping_idx,
                    mapping.to_mapped_dir(),
                ));
            }
            AppTransition::AbortConfiguration => {
                self.configure_mapping_state = None;
            }
            AppTransition::CommitConfiguration(idx, mapped_dir) => {
                self.configure_mapping_state = None;
                self.selecting_input_state.commit_mapping(idx, mapped_dir)
            }
        };
    }
}
