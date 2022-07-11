use super::{app_transition::AppTransition, mapping_state::MappedDir, AppState};
use crossterm::event::{Event, KeyCode, KeyEvent};
use tui::{
    interactive_form::InteractiveForm,
    interactive_form_state,
    widgets::{InteractiveWidgetState, TextInputState},
};

interactive_form_state! {
    pub struct ConfigureMappingFormState {
        pub file_ext_input_state: TextInputState = "mkv,mp4,avi",
        pub dir_matcher_input_state: TextInputState = "(.+)",
        pub dir_replacer_input_state: TextInputState = "$1",
        pub file_matcher_input_state: TextInputState = "(.+)",
        pub file_replacer_input_state: TextInputState = "$1",
    }
}

pub struct ConfigureMappingState {
    pub mapping_idx: usize,
    pub mapped_dir: MappedDir,
    pub form: ConfigureMappingFormState,
}

impl ConfigureMappingState {
    pub fn new(mapping_idx: usize, mapped_dir: MappedDir) -> ConfigureMappingState {
        ConfigureMappingState {
            mapping_idx,
            mapped_dir,
            form: Default::default(),
        }
    }
}

macro_rules! update_if_changed {
    ($idx:expr, $self:ident, $field:ident) => {
        if $self.form.$field.changed() {
            $self
                .mapped_dir
                .set_config($idx, $self.form.$field.get_value());
        }
    };
}

impl AppState for ConfigureMappingState {
    fn on_event(&mut self, event: Event) -> AppTransition {
        if self.form.handle_event(event).is_consumed() {
            update_if_changed!(0, self, file_ext_input_state);
            update_if_changed!(1, self, dir_matcher_input_state);
            update_if_changed!(2, self, dir_replacer_input_state);
            update_if_changed!(3, self, file_matcher_input_state);
            update_if_changed!(4, self, file_replacer_input_state);
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
                if self.form.any_inputs_focused() {
                    self.form.unfocus_inputs();
                    AppTransition::None
                } else {
                    AppTransition::AbortConfiguration
                }
            }
            KeyCode::Enter => {
                AppTransition::CommitConfiguration(self.mapping_idx, self.mapped_dir.clone())
            }
            KeyCode::BackTab => {
                self.form.focus_prev_input();
                AppTransition::None
            }
            KeyCode::Tab => {
                self.form.focus_next_input();
                AppTransition::None
            }
            _ => AppTransition::None,
        }
    }
}
