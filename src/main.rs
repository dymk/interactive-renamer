use std::{
    error::Error,
    io::{self, Stdout},
    ops::DerefMut,
};

use app::{App, AppResult};
use app_state::{
    configure_mapping_state::ConfigureMappingState,
    mapping_state::{self, MappingState},
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Cell, Clear, List, ListItem, Row, Table, TextInput, TextInputState},
    Frame, Terminal,
};

type TTerminal = Terminal<CrosstermBackend<Stdout>>;

mod app;
mod app_state;
mod dao;
mod input_form;
mod path_utils;
mod renamer;
mod stdlib_utils;

fn main() -> Result<(), Box<dyn Error>> {
    let db_path = std::env::args().nth(1).expect("arg 1: db.sqlite");
    let in_dir = std::env::args().nth(2).expect("arg 2: in_dir");
    let out_dir = std::env::args().nth(3).expect("arg 3: out_dir");
    let app = App::new(&db_path, &in_dir, &out_dir);

    // setup terminal
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let res = run_app(&mut terminal, app);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
        return Err(Box::new(err));
    }

    Ok(())
}

fn run_app(terminal: &mut TTerminal, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;
        let event = event::read()?;
        if let AppResult::Quit = app.on_event(event) {
            return Ok(());
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    let is_selecting = !app.configure_mapping_state.is_some();
    ui_selecting_input(f, app, is_selecting);
    if let Some(state) = &app.configure_mapping_state {
        ui_configure_mapping(f, app, &state);
    }
}

fn ui_selecting_input<B: Backend>(f: &mut Frame<B>, app: &App, active: bool) {
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

fn ui_configure_mapping<B: Backend>(
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
            &configure_mapping_state.file_ext_input_state,
            "File Types",
            mapped_dir.has_valid_file_filter(),
        );
        input_block(
            f,
            config_input_rects[1][0],
            &configure_mapping_state.dir_matcher_input_state,
            "Dir Matcher",
            mapped_dir.has_valid_dir_renamer(),
        );
        input_block(
            f,
            config_input_rects[1][1],
            &configure_mapping_state.dir_replacer_input_state,
            "Dir Replacer",
            mapped_dir.has_valid_dir_renamer(),
        );
        input_block(
            f,
            config_input_rects[2][0],
            &configure_mapping_state.file_matcher_input_state,
            "File Matcher",
            mapped_dir.has_valid_file_renamer(),
        );
        input_block(
            f,
            config_input_rects[2][1],
            &configure_mapping_state.file_replacer_input_state,
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
