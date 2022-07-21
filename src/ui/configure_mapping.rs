use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Cell, Clear, List, ListItem, Row, Table, TextInput, TextInputState},
    Frame,
};

use crate::{
    app::App,
    app_state::{
        configure_mapping_state::ConfigureMappingState,
        mapping_state::{self, MappingState},
    },
};
pub fn configure_mapping<B: Backend>(
    f: &mut Frame<B>,
    app: &App,
    configure_mapping_state: &ConfigureMappingState,
) {
    // render parent popup + clear background
    let popup_rect = {
        let popup_rect = Layout::default()
            .direction(Direction::Horizontal)
            .horizontal_margin(10)
            .vertical_margin(1)
            .constraints([Constraint::Percentage(100)].as_ref())
            .split(f.size())[0];

        let block = Block::default()
            .title(Span::styled(
                "Configure Mapping",
                Style::default().add_modifier(Modifier::BOLD),
            ))
            .borders(Borders::ALL);

        let popup_rect_inner = block.inner(popup_rect);
        f.render_widget(Clear, popup_rect);
        f.render_widget(block, popup_rect);
        popup_rect_inner
    };

    // compute main layout within the popup
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(4), // status / input / output dir
                Constraint::Length(9), // input configurations
                Constraint::Min(1),    // file rename preview
            ]
            .as_ref(),
        )
        .split(popup_rect);

    let status_rect = main_layout[0];
    let config_rect = main_layout[1];
    let file_preview_rect = main_layout[2];

    // render status rect
    {
        let existing_mapping = app
            .selecting_input_state
            .mappings()
            .get(configure_mapping_state.mapping_idx)
            .unwrap();

        let status_span = match existing_mapping {
            MappingState::HasMapping { mapped_dir } => {
                if mapped_dir.configs_eq(&configure_mapping_state.mapped_dir) {
                    Span::styled(
                        "Saved",
                        Style::default()
                            .fg(Color::LightGreen)
                            .add_modifier(Modifier::BOLD),
                    )
                } else {
                    Span::styled(
                        "Changed",
                        Style::default()
                            .fg(Color::LightYellow)
                            .add_modifier(Modifier::BOLD),
                    )
                }
            }
            MappingState::Unmapped { in_path: _ } => Span::styled(
                "New",
                Style::default()
                    .fg(Color::LightBlue)
                    .add_modifier(Modifier::ITALIC | Modifier::BOLD),
            ),
        };

        let in_path_span = Span::styled(
            configure_mapping_state.mapped_dir.in_dir_name(),
            Style::default().add_modifier(Modifier::BOLD),
        );
        let out_path_span = match configure_mapping_state.mapped_dir.out_dir_name() {
            Some(out_path) => Span::styled(out_path, Style::default().add_modifier(Modifier::BOLD)),
            None => Span::styled("error", Style::default().fg(Color::Red)),
        };

        let table = Table::new(vec![
            Row::new(vec![
                Cell::from(Span::raw("Status")),
                Cell::from(status_span),
            ]),
            Row::new(vec![
                Cell::from(Span::raw("Input Dir")),
                Cell::from(in_path_span),
            ]),
            Row::new(vec![
                Cell::from(Span::raw("Output Dir")),
                Cell::from(out_path_span),
            ]),
        ])
        .widths([Constraint::Length(16), Constraint::Length(100)].as_ref());
        f.render_widget(table, status_rect);
    }

    // render config input rect
    {
        let config_parent_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(3), // 0 - file ext filter
                    Constraint::Length(3), // 1 - dir matcher / replacer
                    Constraint::Length(3), // 2 - file matcher / replacer
                    Constraint::Min(1),    // x - rest of padding
                ]
                .as_ref(),
            )
            .split(config_rect);

        let config_input_rects = vec![
            // file ext filter
            vec![config_parent_layout[0]],
            // dir matcher / replacer
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(config_parent_layout[1]),
            // file matcher / replacer
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(config_parent_layout[2]),
        ];

        let input_block = |f: &mut Frame<B>,
                           rect: Rect,
                           input_state: &TextInputState,
                           title: &str,
                           is_valid: bool| {
            let border_style = if !is_valid {
                Style::default().fg(Color::Red)
            } else if input_state.is_focused() {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            };

            let title_style = if input_state.is_focused() {
                border_style.add_modifier(Modifier::BOLD)
            } else {
                border_style
            };

            let text_input = TextInput::new().focused_style(border_style).block(
                Block::default()
                    .borders(Borders::ALL)
                    .style(border_style)
                    .title(Span::styled(title, title_style)),
            );
            f.render_interactive(text_input, rect, input_state);
        };

        let mapped_dir = &configure_mapping_state.mapped_dir;
        input_block(
            f,
            config_input_rects[0][0],
            &configure_mapping_state.form.file_ext_input_state,
            "File Types",
            mapped_dir.has_valid_file_filter(),
        );
        input_block(
            f,
            config_input_rects[1][0],
            &configure_mapping_state.form.dir_matcher_input_state,
            "Dir Matcher",
            mapped_dir.has_valid_dir_renamer(),
        );
        input_block(
            f,
            config_input_rects[1][1],
            &configure_mapping_state.form.dir_replacer_input_state,
            "Dir Replacer",
            mapped_dir.has_valid_dir_renamer(),
        );
        input_block(
            f,
            config_input_rects[2][0],
            &configure_mapping_state.form.file_matcher_input_state,
            "File Matcher",
            mapped_dir.has_valid_file_renamer(),
        );
        input_block(
            f,
            config_input_rects[2][1],
            &configure_mapping_state.form.file_replacer_input_state,
            "File Replacer",
            mapped_dir.has_valid_file_renamer(),
        );
    }

    // render file preview rect
    {
        let file_preview_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(file_preview_rect);

        let mapped_dir = &configure_mapping_state.mapped_dir;
        let in_file_rect = file_preview_layout[0];
        let out_file_rect = file_preview_layout[1];

        let file_mappings = mapped_dir.file_mappings();

        let in_file_list = {
            let files_list: Vec<_> = file_mappings
                .iter()
                .map(|mapping| {
                    let span = match mapping {
                        mapping_state::FileMapping::MappedTo {
                            from_name: from_path,
                            to_name: _,
                        } => Span::styled(from_path, Style::default().add_modifier(Modifier::BOLD)),
                        mapping_state::FileMapping::Filtered { name: path } => Span::raw(path),
                    };
                    ListItem::new(span)
                })
                .collect();

            let block = Block::default().borders(Borders::ALL).title(vec![
                Span::raw("Input Files - "),
                Span::styled(
                    format!("{} ", files_list.len()),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
            ]);
            List::new(files_list).block(block)
        };

        let out_file_list = {
            let mut num_files = 0;
            let files_list: Vec<_> = file_mappings
                .iter()
                .map(|mapping| {
                    let span = match mapping {
                        mapping_state::FileMapping::MappedTo {
                            from_name: _,
                            to_name,
                        } => {
                            num_files += 1;
                            Span::raw(to_name)
                        }
                        mapping_state::FileMapping::Filtered { name: _ } => {
                            Span::styled("", Style::default().add_modifier(Modifier::ITALIC))
                        }
                    };
                    ListItem::new(span)
                })
                .collect();
            let block = Block::default().borders(Borders::ALL).title(vec![
                Span::raw("Output Files - "),
                Span::styled(
                    format!("{} ", num_files),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
            ]);
            List::new(files_list).block(block)
        };

        f.render_widget(in_file_list, in_file_rect);
        f.render_widget(out_file_list, out_file_rect);
    }
}
