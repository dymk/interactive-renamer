use crossterm::event::Event;
use tui::widgets::{InteractionOutcome, InteractiveWidgetState};

use crate::stdlib_utils::AndThenOrOption;

/*
 * Structs that implement `InputFormBacking` hooks will automatically get an
 * implementation of `InputForm`
*/
pub trait InputFormBacking {
    fn focused_state_idx(&self) -> Option<usize>;
    fn set_focused_state_idx(&mut self, idx: Option<usize>);
    fn input_states_len(&self) -> usize;
    fn input_state_at_mut(&mut self, idx: usize) -> Option<&mut dyn InteractiveWidgetState>;
}

pub trait InputForm {
    fn handle_input(&mut self, event: Event) -> InteractionOutcome;
    fn focus_next_input(&mut self);
    fn focus_prev_input(&mut self);
    fn any_inputs_focused(&self) -> bool;
    fn unfocus_inputs(&mut self);
}

impl<T: InputFormBacking> InputForm for T {
    fn handle_input(&mut self, event: Event) -> InteractionOutcome {
        self.focused_state_idx()
            .map_or(InteractionOutcome::Bubble, |idx| {
                self.input_state_at_mut(idx)
                    .map_or(InteractionOutcome::Bubble, |state| {
                        state.handle_event(event)
                    })
            })
    }

    fn focus_next_input(&mut self) {
        let input_states_len = self.input_states_len();
        let new_idx = focused_input_idx_mut(self).and_then_or(
            |(idx, state)| {
                state.unfocus();
                if idx < input_states_len - 1 {
                    Some(idx + 1)
                } else {
                    None
                }
            },
            Some(0),
        );

        self.set_focused_state_idx(new_idx);
        if let Some((_, state)) = focused_input_idx_mut(self) { state.focus() }
    }

    fn focus_prev_input(&mut self) {
        let input_states_len = self.input_states_len();
        let new_idx = focused_input_idx_mut(self).and_then_or(
            |(idx, state)| {
                state.unfocus();
                if idx == 0 {
                    None
                } else {
                    Some(idx - 1)
                }
            },
            Some(input_states_len - 1),
        );

        self.set_focused_state_idx(new_idx);
        if let Some((_, state)) = focused_input_idx_mut(self) { state.focus() }
    }

    fn any_inputs_focused(&self) -> bool {
        self.focused_state_idx().is_some()
    }

    fn unfocus_inputs(&mut self) {
        if let Some((_, state)) = focused_input_idx_mut(self) { state.unfocus() }
        self.set_focused_state_idx(None);
    }
}

fn focused_input_idx_mut(
    this: &mut dyn InputFormBacking,
) -> Option<(usize, &mut dyn InteractiveWidgetState)> {
    this.focused_state_idx()
        .and_then(|idx| this.input_state_at_mut(idx).map(|state| (idx, state)))
}
