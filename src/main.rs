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
mod path_utils;
mod renamer;
mod ui;
mod widgets;

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
        terminal.draw(|f| ui(f, &app))?;
        let event = event::read()?;
        if let AppResult::Quit = app.on_event(event) {
            return Ok(());
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    let is_selecting = app.configure_mapping_state.is_none();
    ui::selecting_input(f, app, is_selecting);
    if let Some(state) = &app.configure_mapping_state {
        ui::configure_mapping(f, app, state);
    }
}
