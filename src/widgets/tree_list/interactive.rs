use crossterm::event::{Event, KeyEvent};
use tui::widgets::{InteractionOutcome, InteractiveWidgetState};

use super::state::TreeListState;

impl InteractiveWidgetState for TreeListState {
    fn handle_event(&mut self, event: Event) -> InteractionOutcome {
        self.changed = false;
        if !self.focused {
            return InteractionOutcome::Bubble;
        }

        match event {
            Event::Key(key) => self.handle_key(key),
            _ => InteractionOutcome::Bubble,
        }
    }

    fn changed(&self) -> bool {
        self.changed
    }

    fn is_focused(&self) -> bool {
        self.focused
    }

    fn focus(&mut self) {
        self.focused = true;
    }

    fn unfocus(&mut self) {
        self.focused = false;
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl TreeListState {
    fn handle_key(&mut self, key: KeyEvent) -> InteractionOutcome {
        InteractionOutcome::Bubble
    }
}
