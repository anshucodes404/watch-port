use crate::app::App;
use crate::proc::State;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    // style::{Color, Modifier, Style, Stylize},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, HighlightSpacing, Paragraph, Row, Table},
};

pub fn render(frame: &mut Frame, app: &mut App) {
    let areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(frame.size());

    render_title(frame, app, areas[0]);

    render_table(frame, app, areas[1]);

    render_status(frame, app, areas[2]);
}

fn render_title(frame: &mut Frame, app: &mut App, area: Rect) {
    let title_line = Line::from(vec![
        Span::styled(
            "portwatch",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            " — real-time port monitor ",
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled(
            format!("(refresh #{})", app.refresh_count),
            Style::default().fg(Color::DarkGray),
        ),
    ]);

    let title = Paragraph::new(title_line).block(Block::default().borders(Borders::ALL));

    frame.render_widget(title, area);
}

fn render_table(frame: &mut Frame, app: &mut App, area: Rect) {
    let header = Row::new(vec![
        Cell::from("SL.").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("LOCAL_ADD").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("REMOTE_ADD").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("PROTOCOL").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("STATE").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("PID").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("PROCESS").style(Style::default().add_modifier(Modifier::BOLD)),
    ])
    .style(Style::default().bg(Color::Rgb(50, 50, 50)))
    .height(2);

    // the rows are filtered which are not having any pid, so make it toggle with a key press to see all the acquired ports, that dont have a pid or process name

    let rows: Vec<Row> = app
        .ports
        .iter()
        .filter(|p| p.pid.is_some())
        .enumerate()
        .map(|(i, port)| port_to_row(i, port))
        .collect();

    let widths = [
        Constraint::Length(3),
        Constraint::Length(20),
        Constraint::Length(20),
        Constraint::Length(7),
        Constraint::Length(14),
        Constraint::Length(5),
        Constraint::Min(0),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" {} connections ", app.ports.len())),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(40, 40, 40))
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ")
        .highlight_spacing(HighlightSpacing::Always);

    frame.render_stateful_widget(table, area, &mut app.table_state);
}

fn port_to_row(sl_no: usize, port: &crate::proc::TcpPorts) -> Row<'_> {
    // Color the state cell based on value
    let (state_str, state_color) = match &port.state {
        State::Listen => ("LISTEN", Color::Green),
        State::Established => ("ESTABLISHED", Color::Blue),
        State::TimeWait => ("TIME_WAIT", Color::Yellow),
        State::CloseWait => ("CLOSE_WAIT", Color::Red),
        State::SynSent => ("SYN_SENT", Color::Magenta),
        State::SynRecv => ("SYN_RECV", Color::Magenta),
        State::FinWait1 => ("FIN_WAIT1", Color::DarkGray),
        State::FinWait2 => ("FIN_WAIT2", Color::DarkGray),
        State::Close => ("CLOSE", Color::DarkGray),
        State::LastAck => ("LAST_ACK", Color::DarkGray),
        State::Closing => ("CLOSING", Color::DarkGray),
        State::Unknown(s) => (s.as_str(), Color::DarkGray),
    };

    let pid_str = port.pid.map(|p| p.to_string()).unwrap_or("-".into());

    // remove the processes having "?" name or no name
    let name_str = port.name.clone().unwrap_or("?".into());
    let proto_str = match port.protocol {
        crate::proc::Protocol::TCP => "TCP",
    };

    Row::new(vec![
        Cell::from(sl_no.to_string()),
        Cell::from(port.local_add.to_string()),
        Cell::from(port.remote_add.to_string()),
        Cell::from(proto_str),
        Cell::from(state_str).style(Style::default().fg(state_color)),
        Cell::from(pid_str),
        Cell::from(name_str),
    ])
    .height(1)
}

fn render_status(frame: &mut Frame, app: &mut App, area: Rect) {
    let status_text = if let Some(port) = app.selected_port() {
        let name = port.name.as_deref().unwrap_or("?");

        let pid = port.pid.map(|p| p.to_string()).unwrap_or("-".into());

        format!(
            " Selected: {}  (pid: {}) on port {:?}  {}",
            name, pid, port, app.status_msg
        )
    } else {
        app.status_msg.clone()
    };

    let status = Paragraph::new(status_text)
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::ALL));

    frame.render_widget(status, area);
}
