use crate::app::App;

use super::{app_transition::AppTransition, mapping_state::NUM_CONFIG_INPUTS, AppState};

pub struct ConfigureMappingState {
    pub mapping_idx: usize,
    pub focused_input_idx: Option<usize>,
    pub inputs: [String; NUM_CONFIG_INPUTS],
}

impl ConfigureMappingState {
    pub fn is_active(&self, idx: usize) -> bool {
        match self.focused_input_idx {
            Some(this_idx) => idx == this_idx,
            None => false,
        }
    }

    pub fn input_val(&self, idx: usize) -> &str {
        if idx < NUM_CONFIG_INPUTS {
            self.inputs[idx].as_str()
        } else {
            ""
        }
    }
}

impl AppState for ConfigureMappingState {
    fn on_up(&mut self) -> AppTransition {
        match self.focused_input_idx {
            Some(idx) => {
                if idx > 0 {
                    self.focused_input_idx = Some(idx - 1);
                } else {
                    self.focused_input_idx = None;
                }
            }
            None => self.focused_input_idx = Some(NUM_CONFIG_INPUTS - 1),
        };
        AppTransition::None
    }

    fn on_down(&mut self) -> AppTransition {
        match self.focused_input_idx {
            Some(idx) => {
                if idx < NUM_CONFIG_INPUTS - 1 {
                    self.focused_input_idx = Some(idx + 1);
                } else {
                    self.focused_input_idx = None;
                }
            }
            None => self.focused_input_idx = Some(0),
        };
        AppTransition::None
    }

    fn requesting_input(&self) -> bool {
        match self.focused_input_idx {
            Some(_) => true,
            None => false,
        }
    }

    fn on_char(&mut self, c: char) -> AppTransition {
        if let Some(idx) = self.focused_input_idx {
            self.inputs[idx].push(c);
        };
        AppTransition::None
    }
    fn on_bksp(&mut self) -> AppTransition {
        if let Some(idx) = self.focused_input_idx {
            self.inputs[idx].pop();
        };
        AppTransition::None
    }

    fn on_tab(&mut self) -> AppTransition {
        self.on_down()
    }

    fn on_backtab(&mut self) -> AppTransition {
        self.on_up()
    }

    fn on_esc(&mut self) -> AppTransition {
        AppTransition::AbortConfiguration
    }
}
