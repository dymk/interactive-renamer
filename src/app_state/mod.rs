use crossterm::event::Event;

use self::app_transition::AppTransition;

pub mod app_transition;
pub mod configure_mapping_state;
pub mod mapping_state;
pub mod selecting_input_state;

pub trait AppState {
    fn on_event(&mut self, event: Event) -> AppTransition;
}
