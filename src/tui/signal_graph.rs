use ratatui::prelude::*;
use ratatui::symbols;
use ratatui::widgets::{Axis, Block, BorderType, Borders, Chart, Dataset, GraphType};

use crate::model::AppState;
use crate::tui::theme;

pub fn render_signal_graph(frame: &mut Frame, area: Rect, state: &AppState) {
    let aps = state.sorted_aps();
    let top_aps: Vec<_> = aps.into_iter().take(5).collect();

    let window_samples = state.time_window.sample_count();
    let window_secs = state.time_window.seconds();

    // Build datasets, only taking samples within the time window
    let mut all_data: Vec<(String, Vec<(f64, f64)>, Color)> = Vec::new();
    let mut global_min: f64 = 0.0;
    let mut global_max: f64 = -120.0;
    let mut has_data = false;

    for (i, ap) in top_aps.iter().enumerate() {
        let color = theme::AP_COLORS[i % theme::AP_COLORS.len()];
        let points = ap.signal_history.as_dataset_last_n(window_samples, |&v| v as f64);

        // Track min/max across all visible data points
        for &(_, y) in &points {
            if y < global_min {
                global_min = y;
            }
            if y > global_max {
                global_max = y;
            }
            has_data = true;
        }

        let label = if ap.ssid.is_empty() {
            ap.bssid_str()[..8].to_string()
        } else if ap.ssid.len() > 12 {
            format!("{}...", &ap.ssid[..10])
        } else {
            ap.ssid.clone()
        };
        all_data.push((label, points, color));
    }

    // Dynamic Y bounds: pad by 5 dBm on each side, fall back to -100..-20 if no data
    let (y_min, y_max) = if has_data {
        let lo = (global_min - 5.0).max(-120.0);
        let hi = (global_max + 5.0).min(0.0);
        // Ensure at least 10 dBm range so the chart isn't too compressed
        if hi - lo < 10.0 {
            let mid = (lo + hi) / 2.0;
            ((mid - 5.0).max(-120.0), (mid + 5.0).min(0.0))
        } else {
            (lo, hi)
        }
    } else {
        (-100.0, -20.0)
    };

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

    let x_max = window_samples as f64;
    let half_label = format!("{}s", window_secs / 2);
    let full_label = format!("{}s", window_secs);

    let x_labels: Vec<Line> = vec![full_label.into(), half_label.into(), "now".into()];

    // Generate Y axis labels: min, 1-3 intermediate values, max
    let y_labels: Vec<Line> = {
        let range = y_max - y_min;
        let steps = if range >= 40.0 { 4 } else if range >= 20.0 { 3 } else { 2 };
        let step = range / steps as f64;
        (0..=steps)
            .map(|i| format!("{:.0}", y_min + step * i as f64).into())
            .collect()
    };

    let chart = Chart::new(datasets)
        .block(
            Block::default()
                .title(format!(" Signal Strength (dBm) [{window_secs}s] "))
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
                .bounds([y_min, y_max])
                .labels(y_labels),
        );

    frame.render_widget(chart, area);
}
