use super::{
    app_transition::AppTransition,
    mapping_state::{MappedDir, NUM_CONFIGS},
    AppState,
};

pub struct ConfigureMappingState {
    pub mapping_idx: usize,
    pub focused_input_idx: Option<usize>,
    pub mapped_dir: MappedDir,
}

impl ConfigureMappingState {
    pub fn new(mapping_idx: usize, mapped_dir: MappedDir) -> ConfigureMappingState {
        ConfigureMappingState {
            mapping_idx,
            focused_input_idx: None,
            mapped_dir,
        }
    }

    pub fn is_active(&self, idx: usize) -> bool {
        match self.focused_input_idx {
            Some(this_idx) => idx == this_idx,
            None => false,
        }
    }

    pub fn input_val(&self, idx: usize) -> &str {
        if idx < NUM_CONFIGS {
            self.mapped_dir.config(idx).as_str()
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
            None => self.focused_input_idx = Some(NUM_CONFIGS - 1),
        };
        AppTransition::None
    }

    fn on_down(&mut self) -> AppTransition {
        match self.focused_input_idx {
            Some(idx) => {
                if idx < NUM_CONFIGS - 1 {
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
            self.mapped_dir.config_mut(idx, &|cfg| {
                cfg.push(c);
            })
        };
        AppTransition::None
    }

    fn on_bksp(&mut self) -> AppTransition {
        if let Some(idx) = self.focused_input_idx {
            self.mapped_dir.config_mut(idx, &|cfg| {
                cfg.pop();
            });
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

    fn on_enter(&mut self) -> AppTransition {
        AppTransition::CommitConfiguration(self.mapping_idx, self.mapped_dir.clone())
    }
}
