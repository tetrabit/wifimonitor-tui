use ratatui::prelude::*;
use ratatui::widgets::{Bar, BarChart, BarGroup, Block, BorderType, Borders};

use crate::model::AppState;
use crate::tui::theme;

pub fn render_channel_chart(frame: &mut Frame, area: Rect, state: &AppState) {
    let mut channels: Vec<(u8, u64)> = state
        .channel_packets
        .iter()
        .map(|(&ch, &count)| (ch, count))
        .collect();
    channels.sort_by_key(|(ch, _)| *ch);

    let bars: Vec<Bar> = channels
        .iter()
        .map(|(ch, count)| {
            let color = if *ch <= 14 {
                Color::Rgb(80, 160, 255) // 2.4GHz = blue
            } else {
                Color::Rgb(0, 220, 80) // 5GHz = green
            };
            Bar::default()
                .label(format!("{ch}").into())
                .value(*count)
                .style(Style::default().fg(color))
        })
        .collect();

    let group = BarGroup::default().bars(&bars);

    let chart = BarChart::default()
        .block(
            Block::default()
                .title(" Channel Utilization ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme::BORDER_COLOR)),
        )
        .data(group)
        .bar_width(3)
        .bar_gap(1)
        .max(channels.iter().map(|(_, c)| *c).max().unwrap_or(1));

    frame.render_widget(chart, area);
}
