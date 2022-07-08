use std::{cell::RefCell, rc::Rc};

use crossterm::event::Event;

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

pub enum AppResult {
    KeepGoing,
    Quit,
}

impl App {
    pub fn new(db_path: &str, in_dir: &str, out_dir: &str) -> App {
        let dao = Rc::new(RefCell::new(Dao::new(db_path)));
        App {
            // dao: dao.clone(),
            selecting_input_state: SelectingInputState::new(
                dao,
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
        &mut self.selecting_input_state
    }

    pub fn on_event(&mut self, event: Event) -> AppResult {
        let t = self.current_state_mut().on_event(event);
        self.handle_transition(t)
    }

    fn handle_transition(&mut self, transition: AppTransition) -> AppResult {
        match transition {
            AppTransition::None => AppResult::KeepGoing,
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
                AppResult::KeepGoing
            }
            AppTransition::AbortConfiguration => {
                self.configure_mapping_state = None;
                AppResult::KeepGoing
            }
            AppTransition::CommitConfiguration(idx, mapped_dir) => {
                self.configure_mapping_state = None;
                self.selecting_input_state.commit_mapping(idx, mapped_dir);
                AppResult::KeepGoing
            }
            AppTransition::Quit => AppResult::Quit,
        }
    }
}
