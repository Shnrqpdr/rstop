mod app;
mod config;
mod history;
mod metrics;
mod ui;

use std::io;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use crossterm::event::{
    DisableMouseCapture, EnableMouseCapture, Event, EventStream, KeyCode, KeyEventKind,
    KeyModifiers,
};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use futures::StreamExt;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use tokio::sync::RwLock;

use app::{AppState, render_interval, spawn_poller};
use config::{CliArgs, Config};

fn install_panic_hook() {
    let default = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
        default(info);
    }));
}

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() -> Result<()> {
    let args = CliArgs::parse();
    let config: Config = args.into();

    install_panic_hook();
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let state = Arc::new(RwLock::new(AppState::new(config)));
    let _poller = spawn_poller(Arc::clone(&state));

    let result = run_loop(&mut terminal, Arc::clone(&state)).await;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

async fn run_loop<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    state: Arc<RwLock<AppState>>,
) -> Result<()> {
    let mut events = EventStream::new();
    let mut ticker = tokio::time::interval(render_interval());

    loop {
        tokio::select! {
            _ = ticker.tick() => {
                let s = state.read().await;
                terminal.draw(|f| ui::render(f, &s))?;
            }
            Some(Ok(ev)) = events.next() => {
                if let Event::Key(k) = ev {
                    if k.kind != KeyEventKind::Press {
                        continue;
                    }
                    match k.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        KeyCode::Char('c') if k.modifiers.contains(KeyModifiers::CONTROL) => break,
                        KeyCode::Char(' ') => {
                            let mut s = state.write().await;
                            s.paused = !s.paused;
                        }
                        KeyCode::Char('+') | KeyCode::Char('=') => {
                            let mut s = state.write().await;
                            s.config.interval = (s.config.interval / 2).max(Duration::from_millis(50));
                        }
                        KeyCode::Char('-') | KeyCode::Char('_') => {
                            let mut s = state.write().await;
                            s.config.interval = (s.config.interval * 2).min(Duration::from_secs(10));
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    Ok(())
}
