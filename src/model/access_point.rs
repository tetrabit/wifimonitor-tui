use std::fmt;
use std::time::Instant;

use crate::util::ring_buffer::RingBuffer;

/// Signal history: 240 samples = 60 seconds at 4 samples/sec.
const SIGNAL_HISTORY_CAP: usize = 240;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncryptionType {
    Open,
    WEP,
    WPA,
    WPA2,
    WPA3,
    Unknown,
}

impl fmt::Display for EncryptionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EncryptionType::Open => write!(f, "Open"),
            EncryptionType::WEP => write!(f, "WEP"),
            EncryptionType::WPA => write!(f, "WPA"),
            EncryptionType::WPA2 => write!(f, "WPA2"),
            EncryptionType::WPA3 => write!(f, "WPA3"),
            EncryptionType::Unknown => write!(f, "???"),
        }
    }
}

pub struct AccessPoint {
    pub ssid: String,
    pub bssid: [u8; 6],
    pub channel: u8,
    pub encryption: EncryptionType,
    pub signal_dbm: i8,
    pub signal_history: RingBuffer<i8>,
    pub last_seen: Instant,
    pub beacon_count: u64,
}

impl AccessPoint {
    pub fn new(bssid: [u8; 6]) -> Self {
        Self {
            ssid: String::new(),
            bssid,
            channel: 0,
            encryption: EncryptionType::Unknown,
            signal_dbm: -100,
            signal_history: RingBuffer::new(SIGNAL_HISTORY_CAP),
            last_seen: Instant::now(),
            beacon_count: 0,
        }
    }

    pub fn bssid_str(&self) -> String {
        format!(
            "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            self.bssid[0], self.bssid[1], self.bssid[2],
            self.bssid[3], self.bssid[4], self.bssid[5],
        )
    }

    pub fn display_ssid(&self) -> &str {
        if self.ssid.is_empty() {
            "<hidden>"
        } else {
            &self.ssid
        }
    }

    pub fn signal_quality_percent(&self) -> u8 {
        // Map -100..-20 dBm to 0..100%
        let clamped = self.signal_dbm.max(-100).min(-20) as f32;
        ((clamped + 100.0) / 80.0 * 100.0) as u8
    }

    pub fn seconds_since_seen(&self) -> u64 {
        self.last_seen.elapsed().as_secs()
    }
}
