use std::ops::DerefMut;

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

use crate::{app::App, app_state::mapping_state::MappingState};

pub fn selecting_input<B: Backend>(f: &mut Frame<B>, app: &App, active: bool) {
    let deemph_or_style = |style: Style| {
        if active {
            style
        } else {
            Style::default()
        }
    };

    let max_log_lines = 16;
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(max_log_lines + 2)].as_ref())
        .split(f.size());

    let inputs_outputs_rect = layout[0];
    let logs_rect = layout[1];

    let inputs_outputs_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(50), // input elements
                Constraint::Percentage(50), // mapped output elements
            ]
            .as_ref(),
        )
        .split(inputs_outputs_rect);

    let mappings = app.selecting_input_state.mappings();

    let input_items: Vec<ListItem> = mappings
        .iter()
        .map(|mapping| {
            let style = match mapping {
                MappingState::Unmapped { in_path: _ } => Style::default().fg(Color::Red),
                MappingState::HasMapping { mapped_dir: _ } => Style::default().fg(Color::Green),
            };
            let span = Span::styled(mapping.in_dir_name(), style);
            ListItem::new(span)
        })
        .collect();

    let in_dirs_highlight_style = if active {
        Style::default().bg(Color::Rgb(40, 40, 40))
    } else {
        Style::default()
    };

    let in_dirs_list = List::new(input_items)
        .highlight_style(in_dirs_highlight_style)
        .block(
            Block::default()
                .title(vec![
                    Span::raw("Inputs - "),
                    Span::styled(
                        app.selecting_input_state.in_dir(),
                        deemph_or_style(Style::default().add_modifier(Modifier::BOLD)),
                    ),
                ])
                .borders(Borders::ALL),
        );

    f.render_stateful_widget(
        in_dirs_list,
        inputs_outputs_layout[0],
        app.selecting_input_state
            .list_state()
            .borrow_mut()
            .deref_mut(),
    );

    let output_items: Vec<ListItem> = mappings
        .iter()
        .map(|mapping| {
            let span = match mapping {
                MappingState::HasMapping { mapped_dir } => match mapped_dir.out_dir_name() {
                    Some(out_path) => Span::raw(out_path),
                    None => Span::styled("error", Style::default().fg(Color::Red)),
                },
                MappingState::Unmapped { in_path: _ } => Span::raw(""),
            };
            ListItem::new(span)
        })
        .collect();

    let out_dirs_block = List::new(output_items)
        .highlight_style(in_dirs_highlight_style)
        .block(
            Block::default()
                .title(vec![
                    Span::raw("Outputs - "),
                    Span::styled(
                        app.selecting_input_state.out_dir(),
                        deemph_or_style(Style::default().add_modifier(Modifier::BOLD)),
                    ),
                ])
                .borders(Borders::ALL),
        );
    f.render_stateful_widget(
        out_dirs_block,
        inputs_outputs_layout[1],
        app.selecting_input_state
            .list_state()
            .borrow_mut()
            .deref_mut(),
    );

    {
        let num_logs = app.selecting_input_state.get_logs().len();
        let all_logs = app.selecting_input_state.get_logs();
        let logs_window = &all_logs[num_logs.saturating_sub(max_log_lines.into())..num_logs];
        let log_lines: Vec<_> = logs_window
            .iter()
            .map(|log| ListItem::new(Span::raw(log)))
            .collect();
        let logs = List::new(log_lines).block(Block::default().borders(Borders::ALL).title("Logs"));
        f.render_widget(logs, logs_rect);
    }
}
