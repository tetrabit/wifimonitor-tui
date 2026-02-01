use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};

use crate::model::AppState;
use crate::tui;
use crate::tui::layout;

const TICK_RATE: Duration = Duration::from_millis(250);

pub fn run(state: Arc<Mutex<AppState>>) -> std::io::Result<()> {
    let mut terminal = tui::init_terminal()?;
    let mut last_rate_tick = Instant::now();

    loop {
        // Check if we should stop
        {
            let st = state.lock().unwrap();
            if !st.is_running() {
                break;
            }
        }

        // Tick packet rate counter every second
        if last_rate_tick.elapsed() >= Duration::from_secs(1) {
            let mut st = state.lock().unwrap();
            st.tick_rate();
            st.expire_aps();
            last_rate_tick = Instant::now();
        }

        // Render
        {
            let st = state.lock().unwrap();
            terminal.draw(|frame| {
                layout::draw(frame, &st);
            })?;
        }

        // Handle input with timeout
        if event::poll(TICK_RATE)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            state.lock().unwrap().stop();
                            break;
                        }
                        KeyCode::Up => {
                            let mut st = state.lock().unwrap();
                            if st.table_scroll > 0 {
                                st.table_scroll -= 1;
                            }
                        }
                        KeyCode::Down => {
                            let mut st = state.lock().unwrap();
                            let max = st.access_points.len().saturating_sub(1);
                            if st.table_scroll < max {
                                st.table_scroll += 1;
                            }
                        }
                        KeyCode::Char('b') => {
                            let mut st = state.lock().unwrap();
                            st.band_filter = st.band_filter.next();
                        }
                        KeyCode::Char('t') => {
                            let mut st = state.lock().unwrap();
                            st.time_window = st.time_window.next();
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    tui::restore_terminal(&mut terminal)?;
    Ok(())
}
