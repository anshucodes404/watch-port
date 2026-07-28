mod app;
mod proc;
mod tui;
mod ui;

use anyhow::Result;
use app::App;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

enum Message {
    PortData(Vec<proc::TcpPorts>),
    Error(String),
}

fn main() -> Result<()> {
    // initial setup of the UI and then passing the Result<Tui> to Guard for implemeting Drop on it
    let mut guard = tui::TerminalGuard(tui::setup()?);

    let (tx, rx) = mpsc::channel::<Message>();

    let producer = thread::spawn(move || {
        loop {
            let message = match proc::get_tcp_ports() {
                Ok(ports) => Message::PortData(ports),
                Err(e) => Message::Error(e.to_string()),
            };

            if tx.send(message).is_err() {
                break;
            }

            thread::sleep(Duration::from_secs(2));
        }
    });

    let mut app = App::new();

    loop {
        while let Ok(msg) = rx.try_recv() {
            match msg {
                Message::PortData(ports) => app.on_data_update(ports),
                Message::Error(e) => app.status_msg = format!("Error: {e}"),
            }
        }

        // Render UI
        guard.0.draw(|frame| ui::render(frame, &mut app))?;

        // Handle input events
        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    handle_key(&mut app, key.code);
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    std::mem::drop(rx);
    let _ = producer.join();

    Ok(())
}

fn handle_key(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
            app.should_quit = true;
        }

        // Navigation
        KeyCode::Down | KeyCode::Char('j') => app.select_next(),
        KeyCode::Up | KeyCode::Char('k') => app.select_prev(),
        KeyCode::Home | KeyCode::Char('g') => app.select_first(),
        KeyCode::End | KeyCode::Char('G') => app.select_last(),

        KeyCode::Char('K') => {
            if let Some(port) = app.selected_port() {
                app.status_msg = format!(
                    "  Kill stub: would kill {} (pid {:?}) — confirmation coming in week 3",
                    port.name.as_deref().unwrap_or("?"),
                    port.pid,
                );
            }
        }

        _ => {}
    }
}
