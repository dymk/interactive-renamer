use crossterm::event::{Event, KeyCode, KeyEvent};
use tui::widgets::{InteractiveWidgetState, TextInputState};

use crate::input_form::{InputForm, InputFormHooks};

use super::{app_transition::AppTransition, mapping_state::MappedDir, AppState};

pub struct ConfigureMappingState {
    pub mapping_idx: usize,
    pub mapped_dir: MappedDir,

    focused_idx: Option<usize>,
    pub file_ext_input_state: TextInputState,
    pub dir_matcher_input_state: TextInputState,
    pub dir_replacer_input_state: TextInputState,
    pub file_matcher_input_state: TextInputState,
    pub file_replacer_input_state: TextInputState,
}

impl InputFormHooks for ConfigureMappingState {
    fn input_states_len(&self) -> usize {
        5
    }
    fn input_state_at_mut(&mut self, idx: usize) -> Option<&mut dyn InteractiveWidgetState> {
        if idx >= self.input_states_len() {
            None
        } else {
            Some(self.input_states_mut()[idx])
        }
    }

    fn focused_state_idx(&self) -> Option<usize> {
        self.focused_idx
    }

    fn set_focused_state_idx(&mut self, idx: Option<usize>) {
        self.focused_idx = idx;
    }
}

impl ConfigureMappingState {
    pub fn new(mapping_idx: usize, mapped_dir: MappedDir) -> ConfigureMappingState {
        ConfigureMappingState {
            mapping_idx,
            mapped_dir,
            focused_idx: Default::default(),
            file_ext_input_state: TextInputState::with_value("mkv,mp4,avi"),
            dir_matcher_input_state: TextInputState::with_value("(.+)"),
            dir_replacer_input_state: TextInputState::with_value("$1"),
            file_matcher_input_state: TextInputState::with_value("(.+)"),
            file_replacer_input_state: TextInputState::with_value("$1"),
        }
    }
}

impl AppState for ConfigureMappingState {
    fn on_event(&mut self, event: Event) -> AppTransition {
        let new_state_value =
            self.input_states_mut()
                .iter_mut()
                .enumerate()
                .find_map(|(idx, state)| {
                    if state.handle_event(event).is_consumed() {
                        Some((idx, state.get_value().clone()))
                    } else {
                        None
                    }
                });

        if let Some((idx, new_value)) = new_state_value {
            self.mapped_dir.set_config(idx, &new_value);
            return AppTransition::None;
        }

        match event {
            Event::Key(key) => self.on_key(key),
            _ => AppTransition::None,
        }
    }
}

impl ConfigureMappingState {
    fn on_key(&mut self, key: KeyEvent) -> AppTransition {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                if self.any_inputs_focused() {
                    self.unfocus_inputs();
                    AppTransition::None
                } else {
                    AppTransition::AbortConfiguration
                }
            }
            KeyCode::Enter => {
                AppTransition::CommitConfiguration(self.mapping_idx, self.mapped_dir.clone())
            }
            KeyCode::BackTab => {
                self.focus_prev_input();
                AppTransition::None
            }
            KeyCode::Tab => {
                self.focus_next_input();
                AppTransition::None
            }
            _ => AppTransition::None,
        }
    }

    fn input_states_mut(&mut self) -> [&mut TextInputState; 5] {
        [
            &mut self.file_ext_input_state,
            &mut self.dir_matcher_input_state,
            &mut self.dir_replacer_input_state,
            &mut self.file_matcher_input_state,
            &mut self.file_replacer_input_state,
        ]
    }
}
