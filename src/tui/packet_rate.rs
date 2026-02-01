use ratatui::prelude::*;
use ratatui::widgets::{Block, BorderType, Borders, Sparkline};

use crate::model::AppState;
use crate::tui::theme;

pub fn render_packet_rate(frame: &mut Frame, area: Rect, state: &AppState) {
    let data: Vec<u64> = state.packet_rate_history.iter().copied().collect();

    let max_rate = data.iter().copied().max().unwrap_or(1).max(1);
    let current = data.last().copied().unwrap_or(0);

    let sparkline = Sparkline::default()
        .block(
            Block::default()
                .title(format!(" Packet Rate ({current} pkt/s) "))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme::BORDER_COLOR)),
        )
        .data(&data)
        .max(max_rate)
        .style(Style::default().fg(Color::Rgb(80, 160, 255)));

    frame.render_widget(sparkline, area);
}
