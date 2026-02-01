use std::collections::HashMap;
use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use crate::model::AccessPoint;
use crate::util::ring_buffer::RingBuffer;

/// Packet rate history: 120 samples = 2 minutes at 1 sample/sec.
const PACKET_RATE_HISTORY_CAP: usize = 120;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BandFilter {
    Both,
    Only2_4,
    Only5,
}

impl BandFilter {
    pub fn next(self) -> Self {
        match self {
            BandFilter::Both => BandFilter::Only2_4,
            BandFilter::Only2_4 => BandFilter::Only5,
            BandFilter::Only5 => BandFilter::Both,
        }
    }
}

impl fmt::Display for BandFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BandFilter::Both => write!(f, "2.4+5 GHz"),
            BandFilter::Only2_4 => write!(f, "2.4 GHz"),
            BandFilter::Only5 => write!(f, "5 GHz"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeWindow {
    Sec5,
    Sec10,
    Sec15,
    Sec30,
    Sec60,
}

impl TimeWindow {
    pub fn next(self) -> Self {
        match self {
            TimeWindow::Sec5 => TimeWindow::Sec10,
            TimeWindow::Sec10 => TimeWindow::Sec15,
            TimeWindow::Sec15 => TimeWindow::Sec30,
            TimeWindow::Sec30 => TimeWindow::Sec60,
            TimeWindow::Sec60 => TimeWindow::Sec5,
        }
    }

    /// Number of samples this window covers (at 4 samples/sec).
    pub fn sample_count(self) -> usize {
        match self {
            TimeWindow::Sec5 => 20,
            TimeWindow::Sec10 => 40,
            TimeWindow::Sec15 => 60,
            TimeWindow::Sec30 => 120,
            TimeWindow::Sec60 => 240,
        }
    }

    pub fn seconds(self) -> u64 {
        match self {
            TimeWindow::Sec5 => 5,
            TimeWindow::Sec10 => 10,
            TimeWindow::Sec15 => 15,
            TimeWindow::Sec30 => 30,
            TimeWindow::Sec60 => 60,
        }
    }
}

impl fmt::Display for TimeWindow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}s", self.seconds())
    }
}

/// Shared application state between capture thread and TUI thread.
pub struct AppState {
    /// Map of BSSID → AccessPoint.
    pub access_points: HashMap<[u8; 6], AccessPoint>,
    /// Total packets captured.
    pub total_packets: u64,
    /// Packets captured in the current second (for rate calculation).
    pub packets_this_second: u64,
    /// Packet rate history (packets/sec).
    pub packet_rate_history: RingBuffer<u64>,
    /// Packets per channel for utilization chart.
    pub channel_packets: HashMap<u8, u64>,
    /// Current channel being monitored.
    pub current_channel: u8,
    /// Interface name.
    pub interface: String,
    /// Signal that the app should stop.
    pub running: AtomicBool,
    /// Last time the per-second counters were reset.
    pub last_rate_tick: Instant,
    /// Scroll offset for AP table.
    pub table_scroll: usize,
    /// AP expiry timeout (seconds).
    pub ap_expiry_secs: u64,
    /// Error message from capture thread (visible after TUI teardown).
    pub error: Option<String>,
    /// Which band(s) the channel hopper should scan.
    pub band_filter: BandFilter,
    /// Time window for the signal graph.
    pub time_window: TimeWindow,
}

impl AppState {
    pub fn new(interface: String) -> Self {
        Self {
            access_points: HashMap::new(),
            total_packets: 0,
            packets_this_second: 0,
            packet_rate_history: RingBuffer::new(PACKET_RATE_HISTORY_CAP),
            channel_packets: HashMap::new(),
            current_channel: 1,
            interface,
            running: AtomicBool::new(true),
            last_rate_tick: Instant::now(),
            table_scroll: 0,
            ap_expiry_secs: 120,
            error: None,
            band_filter: BandFilter::Both,
            time_window: TimeWindow::Sec60,
        }
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
    }

    /// Tick the per-second packet rate counter. Call this once per second.
    pub fn tick_rate(&mut self) {
        self.packet_rate_history.push(self.packets_this_second);
        self.packets_this_second = 0;
        self.last_rate_tick = Instant::now();
    }

    /// Remove APs not seen for longer than `ap_expiry_secs`.
    pub fn expire_aps(&mut self) {
        self.access_points
            .retain(|_, ap| ap.seconds_since_seen() < self.ap_expiry_secs);
    }

    /// Get APs sorted by signal strength (strongest first).
    pub fn sorted_aps(&self) -> Vec<&AccessPoint> {
        let mut aps: Vec<&AccessPoint> = self.access_points.values().collect();
        aps.sort_by(|a, b| b.signal_dbm.cmp(&a.signal_dbm));
        aps
    }
}
