use ratatui::prelude::*;
use ratatui::symbols;
use ratatui::widgets::{Axis, Block, BorderType, Borders, Chart, Dataset, GraphType};

use crate::model::AppState;
use crate::tui::theme;

pub fn render_signal_graph(frame: &mut Frame, area: Rect, state: &AppState) {
    let aps = state.sorted_aps();
    let top_aps: Vec<_> = aps.into_iter().take(5).collect();

    // Build datasets with pre-collected data points
    let mut all_data: Vec<(String, Vec<(f64, f64)>, Color)> = Vec::new();

    for (i, ap) in top_aps.iter().enumerate() {
        let color = theme::AP_COLORS[i % theme::AP_COLORS.len()];
        let points = ap.signal_history.as_dataset(|&v| v as f64);
        let label = if ap.ssid.is_empty() {
            ap.bssid_str()[..8].to_string()
        } else if ap.ssid.len() > 12 {
            format!("{}...", &ap.ssid[..10])
        } else {
            ap.ssid.clone()
        };
        all_data.push((label, points, color));
    }

    let datasets: Vec<Dataset> = all_data
        .iter()
        .map(|(label, points, color)| {
            Dataset::default()
                .name(label.as_str())
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(*color))
                .data(points)
        })
        .collect();

    let x_max = 240.0; // max history samples

    let x_labels: Vec<Line> = vec!["60s".into(), "30s".into(), "now".into()];
    let y_labels: Vec<Line> = vec![
        "-100".into(),
        "-80".into(),
        "-60".into(),
        "-40".into(),
        "-20".into(),
    ];

    let chart = Chart::new(datasets)
        .block(
            Block::default()
                .title(" Signal Strength (dBm) ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme::BORDER_COLOR)),
        )
        .x_axis(
            Axis::default()
                .bounds([0.0, x_max])
                .labels(x_labels),
        )
        .y_axis(
            Axis::default()
                .bounds([-100.0, -20.0])
                .labels(y_labels),
        );

    frame.render_widget(chart, area);
}
