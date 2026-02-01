use std::sync::{Arc, Mutex};
use std::time::Instant;

use libwifi::frame::components::{MacAddress, RsnAkmSuite};
use libwifi::parse_frame;
use libwifi::Frame;
use radiotap::Radiotap;

use crate::model::{AccessPoint, AppState, EncryptionType};

/// Parse a raw captured packet (with radiotap header) and update AppState.
pub fn handle_packet(raw: &[u8], state: &Arc<Mutex<AppState>>) {
    // Parse radiotap header
    let radiotap = match Radiotap::from_bytes(raw) {
        Ok(rt) => rt,
        Err(_) => return,
    };

    let signal_dbm = radiotap
        .antenna_signal
        .map(|s| s.value)
        .unwrap_or(-100);

    let channel_freq = radiotap.channel.map(|c| c.freq);

    // The 802.11 frame starts after the radiotap header
    let frame_start = radiotap.header.length as usize;
    if frame_start >= raw.len() {
        return;
    }
    let frame_bytes = &raw[frame_start..];

    // Update packet counters
    {
        let mut st = state.lock().unwrap();
        st.total_packets += 1;
        st.packets_this_second += 1;

        // Track channel usage
        if let Some(freq) = channel_freq {
            let ch = freq_to_channel(freq);
            if ch > 0 {
                *st.channel_packets.entry(ch).or_insert(0) += 1;
            }
        }
    }

    // Try to parse as 802.11 frame (assume no FCS at end)
    let frame = match parse_frame(frame_bytes, false) {
        Ok(f) => f,
        Err(_) => return,
    };

    // We only care about beacon and probe response frames for AP discovery
    match &frame {
        Frame::Beacon(beacon) => {
            let bssid = mac_to_bytes(&beacon.header.address_3);
            let ssid = beacon
                .station_info
                .ssid
                .as_ref()
                .map(|s| s.to_string())
                .unwrap_or_default();
            let channel = beacon
                .station_info
                .ds_parameter_set
                .unwrap_or(0);
            let encryption = determine_encryption_from_station_info(
                &beacon.station_info,
                beacon.capability_info,
            );

            update_ap(state, bssid, ssid, channel, encryption, signal_dbm);
        }
        Frame::ProbeResponse(probe_resp) => {
            let bssid = mac_to_bytes(&probe_resp.header.address_3);
            let ssid = probe_resp
                .station_info
                .ssid
                .as_ref()
                .map(|s| s.to_string())
                .unwrap_or_default();
            let channel = probe_resp
                .station_info
                .ds_parameter_set
                .unwrap_or(0);
            let encryption = determine_encryption_from_station_info(
                &probe_resp.station_info,
                probe_resp.capability_info,
            );

            update_ap(state, bssid, ssid, channel, encryption, signal_dbm);
        }
        _ => {}
    }
}

fn update_ap(
    state: &Arc<Mutex<AppState>>,
    bssid: [u8; 6],
    ssid: String,
    channel: u8,
    encryption: EncryptionType,
    signal_dbm: i8,
) {
    let mut st = state.lock().unwrap();
    let ap = st.access_points.entry(bssid).or_insert_with(|| AccessPoint::new(bssid));

    if !ssid.is_empty() {
        ap.ssid = ssid;
    }
    if channel > 0 {
        ap.channel = channel;
    }
    ap.encryption = encryption;
    ap.signal_dbm = signal_dbm;
    ap.signal_history.push(signal_dbm);
    ap.last_seen = Instant::now();
    ap.beacon_count += 1;
}

fn mac_to_bytes(mac: &MacAddress) -> [u8; 6] {
    mac.0
}

fn determine_encryption_from_station_info(
    station_info: &libwifi::frame::components::StationInfo,
    capability_info: u16,
) -> EncryptionType {
    if let Some(rsn) = &station_info.rsn_information {
        for akm in &rsn.akm_suites {
            if matches!(akm, RsnAkmSuite::SAE) {
                return EncryptionType::WPA3;
            }
        }
        return EncryptionType::WPA2;
    }

    if station_info.wpa_info.is_some() {
        return EncryptionType::WPA;
    }

    // Check capability info privacy bit (bit 4)
    if capability_info & 0x0010 != 0 {
        return EncryptionType::WEP;
    }

    EncryptionType::Open
}

/// Convert WiFi frequency (MHz) to channel number.
fn freq_to_channel(freq: u16) -> u8 {
    match freq {
        2412 => 1,
        2417 => 2,
        2422 => 3,
        2427 => 4,
        2432 => 5,
        2437 => 6,
        2442 => 7,
        2447 => 8,
        2452 => 9,
        2457 => 10,
        2462 => 11,
        2467 => 12,
        2472 => 13,
        2484 => 14,
        5180 => 36,
        5200 => 40,
        5220 => 44,
        5240 => 48,
        5260 => 52,
        5280 => 56,
        5300 => 60,
        5320 => 64,
        5500 => 100,
        5520 => 104,
        5540 => 108,
        5560 => 112,
        5580 => 116,
        5600 => 120,
        5620 => 124,
        5640 => 128,
        5660 => 132,
        5680 => 136,
        5700 => 140,
        5720 => 144,
        5745 => 149,
        5765 => 153,
        5785 => 157,
        5805 => 161,
        5825 => 165,
        _ => 0,
    }
}

/// Start the capture loop on a separate thread.
/// `use_rfmon`: if true, ask pcap to enable monitor mode (only when we haven't done it via iw).
pub fn start_capture(
    interface: &str,
    state: Arc<Mutex<AppState>>,
    use_rfmon: bool,
) -> std::thread::JoinHandle<()> {
    let iface = interface.to_string();
    std::thread::spawn(move || {
        let cap = match pcap::Capture::from_device(iface.as_str()) {
            Ok(c) => c,
            Err(e) => {
                let msg = format!("Failed to open capture device '{}': {}", iface, e);
                let mut st = state.lock().unwrap();
                st.error = Some(msg);
                st.stop();
                return;
            }
        };

        let cap = cap.promisc(true).timeout(100);
        let cap = if use_rfmon { cap.rfmon(true) } else { cap };

        let mut cap = match cap.open() {
            Ok(c) => c,
            Err(e) => {
                let msg = format!("Failed to activate capture on '{}': {}", iface, e);
                let mut st = state.lock().unwrap();
                st.error = Some(msg);
                st.stop();
                return;
            }
        };

        while state.lock().unwrap().is_running() {
            match cap.next_packet() {
                Ok(packet) => {
                    handle_packet(packet.data, &state);
                }
                Err(pcap::Error::TimeoutExpired) => continue,
                Err(e) => {
                    let mut st = state.lock().unwrap();
                    st.error = Some(format!("Capture error: {e}"));
                    st.stop();
                    break;
                }
            }
        }
    })
}
