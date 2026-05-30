use anyhow::Result;
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, emable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend, terminal};
use std::io::{self, stdout};

pub type Tui = Terminal<CrosstermBackend<stdout>>;

pub fn setup() -> Result<Tui> {
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;
    let terminal: Terminal<CrosstermBackend<io::Stdout>> = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    Ok(terminal)
}

pub fn teardown(terminal: &mut Tui) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
} 


// i have used this for, even if main paincs teardown will absolutely run
pub struct TerminalGuard(pub Tui);

impl Drop for TerminalGuard {
    fn drop(&mut self){
        let _ = teardown(&mut self.0);
    }
}