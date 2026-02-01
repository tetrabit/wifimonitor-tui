use ratatui::prelude::*;

use crate::model::AppState;
use crate::tui::{ap_table, channel_chart, header, packet_rate, signal_graph};

pub fn draw(frame: &mut Frame, state: &AppState) {
    let outer = Layout::vertical([
        Constraint::Length(1),       // Header bar
        Constraint::Percentage(50),  // Top section (graphs)
        Constraint::Min(8),          // Bottom section (AP table)
    ])
    .split(frame.area());

    // Header
    header::render_header(frame, outer[0], state);

    // Top section: signal graph (left) + right panel
    let top = Layout::horizontal([
        Constraint::Percentage(60), // Signal strength chart
        Constraint::Percentage(40), // Right panel
    ])
    .split(outer[1]);

    signal_graph::render_signal_graph(frame, top[0], state);

    // Right panel: packet rate (top) + channel chart (bottom)
    let right = Layout::vertical([
        Constraint::Percentage(40), // Packet rate sparkline
        Constraint::Percentage(60), // Channel utilization
    ])
    .split(top[1]);

    packet_rate::render_packet_rate(frame, right[0], state);
    channel_chart::render_channel_chart(frame, right[1], state);

    // Bottom: AP table
    ap_table::render_ap_table(frame, outer[2], state);
}
