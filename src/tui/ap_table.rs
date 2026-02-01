use ratatui::prelude::*;
use ratatui::widgets::{Block, BorderType, Borders, Cell, Row, Table};

use crate::model::{AccessPoint, AppState};
use crate::tui::theme;

pub fn render_ap_table(frame: &mut Frame, area: Rect, state: &AppState) {
    let aps = state.sorted_aps();

    let header = Row::new(vec![
        Cell::from("SSID"),
        Cell::from("BSSID"),
        Cell::from("CH"),
        Cell::from("Enc"),
        Cell::from("Signal"),
        Cell::from("Bar"),
        Cell::from("Age"),
    ])
    .style(theme::TABLE_HEADER)
    .height(1);

    let rows: Vec<Row> = aps
        .iter()
        .skip(state.table_scroll)
        .map(|ap| {
            let style = if ap.seconds_since_seen() > 30 {
                theme::TABLE_ROW_DIM
            } else {
                theme::TABLE_ROW
            };

            Row::new(vec![
                Cell::from(ap.display_ssid().to_string()),
                Cell::from(ap.bssid_str()),
                Cell::from(format!("{:>3}", ap.channel)),
                Cell::from(ap.encryption.to_string()),
                Cell::from(format!("{}dBm", ap.signal_dbm)),
                Cell::from(signal_bar(ap)),
                Cell::from(format_age(ap.seconds_since_seen())),
            ])
            .style(style)
        })
        .collect();

    let widths = [
        Constraint::Min(16),
        Constraint::Length(17),
        Constraint::Length(4),
        Constraint::Length(6),
        Constraint::Length(8),
        Constraint::Length(10),
        Constraint::Length(6),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .title(" Access Points ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme::BORDER_COLOR)),
        )
        .highlight_style(theme::TABLE_HIGHLIGHT);

    frame.render_widget(table, area);
}

fn signal_bar(ap: &AccessPoint) -> String {
    let pct = ap.signal_quality_percent() as usize;
    let filled = pct / 12; // max ~8 chars
    let bar: String = "█".repeat(filled);
    let color_char = if ap.signal_dbm >= -50 {
        bar
    } else if ap.signal_dbm >= -70 {
        bar
    } else {
        bar
    };
    color_char
}

fn format_age(secs: u64) -> String {
    if secs < 60 {
        format!("{}s", secs)
    } else {
        format!("{}m", secs / 60)
    }
}
