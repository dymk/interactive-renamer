use self::app_transition::AppTransition;

pub mod app_transition;
pub mod configure_mapping_state;
pub mod mapping_state;
pub mod selecting_input_state;

pub trait AppState {
    fn on_up(&mut self) -> AppTransition {
        AppTransition::None
    }
    fn on_down(&mut self) -> AppTransition {
        AppTransition::None
    }
    fn on_tab(&mut self) -> AppTransition {
        AppTransition::None
    }
    fn on_backtab(&mut self) -> AppTransition {
        AppTransition::None
    }
    fn on_enter(&mut self) -> AppTransition {
        AppTransition::None
    }
    fn on_esc(&mut self) -> AppTransition {
        AppTransition::None
    }
    fn on_bksp(&mut self) -> AppTransition {
        AppTransition::None
    }
    fn on_char(&mut self, _c: char) -> AppTransition {
        AppTransition::None
    }
    fn requesting_input(&self) -> bool;
}
