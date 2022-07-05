use std::{
    borrow::BorrowMut,
    error::Error,
    io::{self, Stdout},
    ops::DerefMut,
};

use app::App;
use app_state::{configure_mapping_state::ConfigureMappingState, mapping_state::MappingState};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Text},
    widgets::{Block, Borders, Cell, Clear, List, ListItem, Paragraph, Row, Table},
    Frame, Terminal,
};

type TTerminal = Terminal<CrosstermBackend<Stdout>>;

mod app;
// mod dir_renamer;
mod app_state;
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
        app.selecting_input_state.update_mappings_cache();
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

    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
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
                        Style::default().add_modifier(Modifier::BOLD),
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
                    Style::default().add_modifier(Modifier::BOLD),
                ),
            ])
            .borders(Borders::ALL),
    );
    f.render_widget(out_dirs_block, layout[1]);
}

fn ui_configure_mapping<B: Backend>(f: &mut Frame<B>, app: &App, state: &ConfigureMappingState) {
    let mapping = app
        .selecting_input_state
        .mappings()
        .get(state.mapping_idx)
        .unwrap();

    let size = f.size();
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .horizontal_margin(10)
        .vertical_margin(5)
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(size);

    let (in_path, out_path) = match mapping {
        MappingState::HasMapping(mapping) => {
            (mapping.in_path().to_string(), Some(mapping.out_path()))
        }
        MappingState::Unmapped(in_path) => (in_path.clone(), None),
    };

    let in_path = Span::styled(in_path, Style::default().add_modifier(Modifier::BOLD));
    let out_path = match out_path {
        Some(out_path) => Span::styled(out_path, Style::default().add_modifier(Modifier::BOLD)),
        None => Span::styled(
            "(none)",
            Style::default()
                .add_modifier(Modifier::ITALIC)
                .fg(Color::DarkGray),
        ),
    };

    let block = Block::default()
        .title(Span::raw("Configure Mapping"))
        .borders(Borders::ALL);
    let size = block.inner(layout[0]);
    f.render_widget(Clear, layout[0]);
    f.render_widget(block, layout[0]);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(4), // input / output dir display
                Constraint::Min(1),    // configuration block
            ]
            .as_ref(),
        )
        .split(size);

    let table = Table::new(vec![
        Row::new(vec![
            Cell::from(Span::raw("Input Dir")),
            Cell::from(in_path),
        ]),
        Row::new(vec![
            Cell::from(Span::raw("Output Dir")),
            Cell::from(out_path),
        ]),
    ])
    .widths([Constraint::Length(10), Constraint::Length(100)].as_ref());
    f.render_widget(table, layout[0]);

    let size = layout[1];
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3), // 0 - file type fileter
                Constraint::Length(3), // 1 - dir finder
                Constraint::Length(3), // 2 - dir replacer
                Constraint::Length(3), // 3 - file finder
                Constraint::Length(3), // 4 - file replacer
                Constraint::Min(1),    // x - rest of padding
            ]
            .as_ref(),
        )
        .split(size);

    let input_block = |f: &mut Frame<B>, idx: usize, title: &str| {
        let is_active = state.is_active(idx);
        let rect = layout[idx];
        let text = state.input_val(idx);

        let style = if is_active {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        let dir_matcher_block = Block::default()
            .borders(Borders::ALL)
            .style(style)
            .title(title);
        let inside = dir_matcher_block.inner(rect);
        f.render_widget(dir_matcher_block, rect);

        let text_block = Paragraph::new(Text::raw(text));
        f.render_widget(text_block, inside);

        if is_active {
            f.set_cursor(rect.x + 1 + (text.len() as u16), rect.y + 1)
        }
    };

    input_block(f, 0, "File Filter");
    input_block(f, 1, "Dir Matcher");
    input_block(f, 2, "File Filter");
    input_block(f, 3, "File Filter");
    input_block(f, 4, "File Filter");

    // match mapping {
    //     MappingState::HasMapping(mapping) => todo!(),
    //     MappingState::Unmapped(in_path) => todo!(),
    // }
}
