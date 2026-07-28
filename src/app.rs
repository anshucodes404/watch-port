use crate::proc::{State, TcpPorts};
// use color_eyre::eyre::Ok;
use ratatui::widgets::TableState;

#[derive(Debug, PartialEq)]
pub enum AppMode {
    NORMAL,
    CONFORMING { port_idx: usize },
}

pub struct App {
    pub ports: Vec<TcpPorts>,
    pub table_state: TableState,
    pub mode: AppMode,
    pub status_msg: String,
    pub should_quit: bool,
    pub refresh_count: u64,
    pub last_action_msg: Option<String>,
}

impl App {
    pub fn new() -> Self {
        let mut table_state = TableState::default();
        table_state.select(Some(0));

        Self {
            ports: vec![],
            table_state,
            mode: AppMode::NORMAL,
            status_msg: "Loading...".into(),
            should_quit: false,
            refresh_count: 0,
            last_action_msg: None,
        }
    }

    pub fn on_data_update(&mut self, new_ports: Vec<TcpPorts>) {
        self.refresh_count += 1;

        let prev_selected = self.table_state.selected();

        self.ports = new_ports;

        // it holds which row to select when the UI will update
        let new_selection = match prev_selected {
            Some(i) => Some(i.min(self.ports.len().saturating_sub(1))),
            None if !self.ports.is_empty() => Some(0),
            None => None,
        };

        self.table_state.select(new_selection);

        let listen_count = self
            .ports
            .iter()
            .filter(|p| matches!(p.state, crate::proc::State::Listen))
            .count();

        self.status_msg = format!(
            " {}  connections   {}   listening  refresh #{}   ↑↓ navigate   K kill   Q quit",
            self.ports.len(),
            listen_count,
            self.refresh_count,
        );
    }

    // Navigation functions
    pub fn select_next(&mut self) {
        if self.ports.is_empty() {
            return;
        }
        let curr_selected = self.table_state.selected().unwrap_or(0);
        self.table_state
            .select(Some((curr_selected + 1).min(self.ports.len() - 1)));
    }

    pub fn select_prev(&mut self) {
        let curr_selected = self.table_state.selected().unwrap_or(0);
        self.table_state
            .select(Some(curr_selected.saturating_sub(1))); // saturating_sub will decrease by one but not pass 0, not used i - 1 b/c it might become -1 and u64 cant hold -ve, so it will panic or overflow
    }

    pub fn select_first(&mut self) {
        if !self.ports.is_empty() {
            self.table_state.select(Some(0));
        }
    }

    pub fn select_last(&mut self) {
        if !self.ports.is_empty() {
            self.table_state.select(Some(self.ports.len() - 1));
        }
    }

    // killing a process

    pub fn enter_kill_confirm(&mut self) {
        if let Some(i) = self.table_state.selected() {
            if i < self.ports.len() {
                self.mode = AppMode::CONFORMING { port_idx: i }
            }
        }
    }

    pub fn confirm_kill(&mut self) {
        if let AppMode::CONFORMING { port_idx } = self.mode {
            if let Some(port) = self.ports.get(port_idx) {
                match port.pid {
                    Some(pid) => {
                        let res = kill_process(pid);
                        self.last_action_msg = Some(match res {
                            Ok(_) => format!(
                                " Sent SIGTERM to {} (pid{})",
                                port.name.as_deref().unwrap_or("?"),
                                pid
                            ),
                            Err(e) => format!(" Kill failed: {e}"),
                        });
                    }
                    None => {
                        self.last_action_msg =
                            Some(" Cannot kill: PID unknown (try running with sudo)".into());
                    }
                }
            }
        }
        self.mode = AppMode::NORMAL;
    }

    pub fn cancel_kill(&mut self) {
        self.last_action_msg = Some(" Kill cancelled".into());
        self.mode = AppMode::NORMAL;
    }

    // getter for the selected port
    pub fn selected_port(&self) -> Option<&TcpPorts> {
        self.table_state.selected().and_then(|i| self.ports.get(i))
    }

    pub fn kill_target(&self) -> Option<&TcpPorts> {
        if let AppMode::CONFORMING { port_idx } = self.mode {
            self.ports.get(port_idx)
        } else {
            None
        }
    }
}

fn kill_process(pid: u32) -> Result<(), String> {
    let result = unsafe { libc::kill(pid as libc::pid_t, libc::SIGTERM) };

    if result == 0 {
        Ok(())
    } else {
        let errno = unsafe { *libc::__errno_location() };
        let msg = match errno {
            libc::EPERM => "permission denied (try sudo)".to_string(),
            libc::ESRCH => "process not found (already exited?)".to_string(),
            _ => format!("errno {errno}"),
        };
        Err(msg)
    }
}
