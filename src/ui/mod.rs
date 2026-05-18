mod widgets;

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout as TLayout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::app::AppState;

pub fn render(frame: &mut Frame, state: &AppState) {
    let size = frame.area();
    let chunks = TLayout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(size);

    render_header(frame, chunks[0], state);
    render_body(frame, chunks[1], state);
    render_footer(frame, chunks[2]);
}

fn render_header(frame: &mut Frame, area: Rect, state: &AppState) {
    let d = &state.config.detail;
    let detailed = [
        ("cpu", d.cpu),
        ("mem", d.mem),
        ("gpu", d.gpu),
        ("temp", d.temp),
    ]
    .into_iter()
    .filter(|(_, on)| *on)
    .map(|(n, _)| n)
    .collect::<Vec<_>>()
    .join(",");
    let detailed = if detailed.is_empty() {
        "none".to_string()
    } else {
        detailed
    };
    let paused = if state.paused { "  •  [PAUSED]" } else { "" };
    let title = format!(
        " crabtop  •  interval {}ms  •  history {}  •  detailed: {}{}",
        state.config.interval.as_millis(),
        state.config.history,
        detailed,
        paused
    );
    let p = Paragraph::new(title).style(Style::default().add_modifier(Modifier::BOLD));
    frame.render_widget(p, area);
}

fn render_footer(frame: &mut Frame, area: Rect) {
    let hints = Line::from(vec![
        Span::styled("q", Style::default().fg(Color::Yellow)),
        Span::raw(" quit  "),
        Span::styled("space", Style::default().fg(Color::Yellow)),
        Span::raw(" pause  "),
        Span::styled("+/-", Style::default().fg(Color::Yellow)),
        Span::raw(" interval"),
    ]);
    frame.render_widget(Paragraph::new(hints), area);
}

/// Single stacked screen: CPU, Memory, GPU and Thermals line charts (or their
/// detailed views when the corresponding flag is set), top to bottom.
fn render_body(frame: &mut Frame, area: Rect, state: &AppState) {
    let rows = TLayout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Ratio(1, 4),
            Constraint::Ratio(1, 4),
            Constraint::Ratio(1, 4),
            Constraint::Ratio(1, 4),
        ])
        .split(area);

    widgets::cpu_panel(frame, rows[0], state);
    widgets::mem_panel(frame, rows[1], state);
    widgets::gpu_panel(frame, rows[2], state);
    widgets::temp_panel(frame, rows[3], state);
}
