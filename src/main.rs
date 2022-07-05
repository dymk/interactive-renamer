use std::{
    borrow::BorrowMut,
    error::Error,
    io::{self, Stdout},
    ops::DerefMut,
};

use app::App;
use app_state::{
    configure_mapping_state::ConfigureMappingState,
    mapping_state::{self, MappingState},
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Text},
    widgets::{Block, Borders, Cell, Clear, List, ListItem, Paragraph, Row, Table},
    Frame, Terminal,
};

type TTerminal = Terminal<CrosstermBackend<Stdout>>;

mod app;
mod app_state;
mod dao;
mod renamer;

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

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Up => app.on_up(),
                KeyCode::Down => app.on_down(),
                KeyCode::Enter => app.on_enter(),
                KeyCode::Esc => app.on_esc(),
                KeyCode::Backspace => app.on_bksp(),
                KeyCode::Tab => app.on_tab(),
                KeyCode::BackTab => app.on_backtab(),
                KeyCode::Char(c) => {
                    if app.requesting_input() {
                        app.on_char(c)
                    } else if c == 'q' {
                        return Ok(());
                    }
                }
                _ => {}
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    let is_selecting = if let Some(_) = app.configure_mapping_state {
        false
    } else {
        true
    };

    ui_selecting_input(f, app, is_selecting);
    if let Some(state) = &app.configure_mapping_state {
        ui_configure_mapping(f, app, &state);
    }
}

fn ui_selecting_input<B: Backend>(f: &mut Frame<B>, app: &App, active: bool) {
    let size = f.size();
    let deemph_or_style = |style: Style| {
        if active {
            style
        } else {
            Style::default()
        }
    };

    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(50), // input elements
                Constraint::Percentage(50), // mapped output elements
            ]
            .as_ref(),
        )
        .split(size);

    let mappings = app.selecting_input_state.mappings();

    let input_items: Vec<ListItem> = mappings
        .iter()
        .map(|mapping| {
            let span = match mapping {
                MappingState::Unmapped(path) => Span::styled(path, Style::default().fg(Color::Red)),
                MappingState::HasMapping(mapping) => {
                    Span::styled(mapping.in_path(), Style::default().fg(Color::Green))
                }
            };
            ListItem::new(span)
        })
        .collect();

    let in_dirs_highlight_style = if active {
        Style::default().bg(Color::Rgb(40, 40, 40))
    } else {
        Style::default()
    };

    let in_dirs_list = List::new(input_items)
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
        )
        .highlight_style(in_dirs_highlight_style);

    f.render_stateful_widget(
        in_dirs_list,
        layout[0],
        app.selecting_input_state
            .list_state()
            .borrow_mut()
            .deref_mut(),
    );

    let output_items: Vec<ListItem> = mappings
        .iter()
        .map(|mapping| {
            let span = match mapping {
                MappingState::HasMapping(mapping) => Span::raw(mapping.out_path()),
                MappingState::Unmapped(_) => Span::raw(""),
            };
            ListItem::new(span)
        })
        .collect();

    let out_dirs_block = List::new(output_items).block(
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
    f.render_widget(out_dirs_block, layout[1]);
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
            MappingState::HasMapping(committed_mapping) => {
                if *committed_mapping == configure_mapping_state.mapped_dir {
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
            MappingState::Unmapped(_) => Span::styled(
                "New",
                Style::default()
                    .fg(Color::LightBlue)
                    .add_modifier(Modifier::ITALIC | Modifier::BOLD),
            ),
        };

        let in_path_span = Span::styled(
            configure_mapping_state.mapped_dir.in_path(),
            Style::default().add_modifier(Modifier::BOLD),
        );
        let out_path_span = Span::styled(
            configure_mapping_state.mapped_dir.out_path(),
            Style::default().add_modifier(Modifier::BOLD),
        );

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

        let input_block = |f: &mut Frame<B>, rect: Rect, config_idx: usize, title: &str| {
            let config_input_is_active = configure_mapping_state.is_active(config_idx);
            let text = configure_mapping_state.input_val(config_idx);

            let border_style = if config_input_is_active {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            };
            let title_style = if config_input_is_active {
                border_style.add_modifier(Modifier::BOLD)
            } else {
                border_style
            };

            let dir_matcher_block = Block::default()
                .borders(Borders::ALL)
                .style(border_style)
                .title(Span::styled(title, title_style));
            let inside = dir_matcher_block.inner(rect);
            f.render_widget(dir_matcher_block, rect);

            let text_block = Paragraph::new(Text::raw(text));
            f.render_widget(text_block, inside);

            if config_input_is_active {
                f.set_cursor(rect.x + 1 + (text.len() as u16), rect.y + 1)
            }
        };

        input_block(f, config_input_rects[0][0], 0, "File Types");
        input_block(f, config_input_rects[1][0], 1, "Dir Matcher");
        input_block(f, config_input_rects[1][1], 2, "Dir Filter");
        input_block(f, config_input_rects[2][0], 3, "File Matcher");
        input_block(f, config_input_rects[2][1], 4, "File Filter");
    }

    // render file preview rect
    {
        let file_preview_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(file_preview_rect);

        let in_file_rect = file_preview_layout[0];
        let out_file_rect = file_preview_layout[1];

        let in_file_list = {
            let files_list = vec![];
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
            let files_list = vec![];
            let block = Block::default().borders(Borders::ALL).title(vec![
                Span::raw("Output Files - "),
                Span::styled(
                    format!("{} ", files_list.len()),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
            ]);
            List::new(files_list).block(block)
        };

        f.render_widget(in_file_list, in_file_rect);
        f.render_widget(out_file_list, out_file_rect);
    }
}
