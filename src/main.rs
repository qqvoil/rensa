mod app;
mod cleaner;
mod scanner;
mod ui;

use std::io::{self, stdout};
use std::time::Duration;

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use app::{App, AppMode};

fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Panic hook to ensure terminal restoration
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        original_hook(panic_info);
    }));

    // Create app
    let mut app = App::new();

    // Event loop
    let tick_rate = Duration::from_millis(80);

    while !app.should_quit {
        terminal.draw(|f| ui::draw(f, &app))?;

        if event::poll(tick_rate)? {
            if let Event::Key(key) = event::read()? {
                // Global interrupt
                if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                    app.should_quit = true;
                    break;
                }

                match app.mode {
                    AppMode::Select => match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            app.should_quit = true;
                        }
                        KeyCode::Char(' ') => {
                            app.toggle_current();
                        }
                        KeyCode::Char('a') => {
                            app.toggle_all();
                        }
                        KeyCode::Char('j') | KeyCode::Down => {
                            app.move_down();
                        }
                        KeyCode::Char('k') | KeyCode::Up => {
                            app.move_up();
                        }
                        KeyCode::Char('r') => {
                            app.start_scan();
                        }
                        KeyCode::Enter => {
                            app.start_clean();
                        }
                        _ => {}
                    },
                    AppMode::Executing => {
                        // User can press q or Esc to exit if needed
                        if key.code == KeyCode::Char('q') {
                            app.should_quit = true;
                        }
                    }
                    AppMode::Done => match key.code {
                        KeyCode::Char('q') => {
                            app.should_quit = true;
                        }
                        KeyCode::Enter | KeyCode::Esc => {
                            app.mode = AppMode::Select;
                            app.start_scan();
                        }
                        _ => {}
                    },
                }
            }
        }

        app.tick();
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
