use ratatui::style::{Color, Modifier, Style};

pub const HEADER_BG: Color = Color::Rgb(40, 40, 60);
pub const HEADER_FG: Color = Color::Rgb(180, 200, 255);

pub const BORDER_COLOR: Color = Color::Rgb(80, 80, 120);
pub const BORDER_FOCUSED: Color = Color::Rgb(120, 140, 220);

pub const SIGNAL_STRONG: Color = Color::Rgb(0, 220, 80);
pub const SIGNAL_MEDIUM: Color = Color::Rgb(220, 200, 0);
pub const SIGNAL_WEAK: Color = Color::Rgb(220, 60, 60);

pub const TABLE_HEADER: Style = Style::new()
    .fg(Color::Rgb(180, 200, 255))
    .add_modifier(Modifier::BOLD);

pub const TABLE_ROW: Style = Style::new().fg(Color::Rgb(200, 200, 220));
pub const TABLE_ROW_DIM: Style = Style::new().fg(Color::Rgb(100, 100, 120));
pub const TABLE_HIGHLIGHT: Style = Style::new()
    .fg(Color::Rgb(255, 255, 255))
    .add_modifier(Modifier::BOLD);

/// Color palette for up to 8 AP signal lines on the chart.
pub const AP_COLORS: &[Color] = &[
    Color::Rgb(0, 220, 80),   // green
    Color::Rgb(80, 160, 255), // blue
    Color::Rgb(255, 160, 40), // orange
    Color::Rgb(220, 60, 220), // magenta
    Color::Rgb(0, 200, 200),  // cyan
    Color::Rgb(255, 100, 100),// red-ish
    Color::Rgb(180, 180, 60), // olive
    Color::Rgb(200, 140, 255),// purple
];

pub fn signal_color(dbm: i8) -> Color {
    if dbm >= -50 {
        SIGNAL_STRONG
    } else if dbm >= -70 {
        SIGNAL_MEDIUM
    } else {
        SIGNAL_WEAK
    }
}
