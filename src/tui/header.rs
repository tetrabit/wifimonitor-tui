use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

use crate::model::AppState;
use crate::tui::theme;

pub fn render_header(frame: &mut Frame, area: Rect, state: &AppState) {
    let text = format!(
        " wifimonitor-tui  |  {}  |  CH: {}  |  Band: {}  |  Window: {}  |  Pkts: {}  |  APs: {}  |  q:quit  ↑↓:scroll  b:band  t:time",
        state.interface,
        state.current_channel,
        state.band_filter,
        state.time_window,
        state.total_packets,
        state.access_points.len(),
    );

    let header = Paragraph::new(text)
        .style(Style::default().fg(theme::HEADER_FG).bg(theme::HEADER_BG));

    frame.render_widget(header, area);
}
