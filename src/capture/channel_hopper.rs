use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::model::app_state::BandFilter;
use crate::model::AppState;

const CHANNELS_2_4: &[u8] = &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
const CHANNELS_5: &[u8] = &[36, 40, 44, 48, 149, 153, 157, 161, 165];

/// Dwell time per channel in milliseconds.
const DWELL_MS: u64 = 200;

fn channels_for_band(band: BandFilter) -> Vec<u8> {
    match band {
        BandFilter::Both => {
            let mut v = CHANNELS_2_4.to_vec();
            v.extend_from_slice(CHANNELS_5);
            v
        }
        BandFilter::Only2_4 => CHANNELS_2_4.to_vec(),
        BandFilter::Only5 => CHANNELS_5.to_vec(),
    }
}

/// Start channel hopping on a separate thread.
pub fn start_hopper(interface: &str, state: Arc<Mutex<AppState>>) -> thread::JoinHandle<()> {
    let iface = interface.to_string();
    thread::spawn(move || {
        let mut idx = 0;
        let mut current_band = BandFilter::Both;
        let mut channels = channels_for_band(current_band);

        while state.lock().unwrap().is_running() {
            // Check if band filter changed
            let band = state.lock().unwrap().band_filter;
            if band != current_band {
                current_band = band;
                channels = channels_for_band(current_band);
                idx = 0;
            }

            let channel = channels[idx % channels.len()];

            let result = Command::new("iw")
                .args(["dev", &iface, "set", "channel", &channel.to_string()])
                .output();

            if result.is_ok() {
                state.lock().unwrap().current_channel = channel;
            }

            idx = (idx + 1) % channels.len();
            thread::sleep(Duration::from_millis(DWELL_MS));
        }
    })
}
