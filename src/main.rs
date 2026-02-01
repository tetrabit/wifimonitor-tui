mod app;
mod capture;
mod model;
mod tui;
mod util;

use std::sync::{Arc, Mutex};

use clap::Parser;

use capture::channel_hopper;
use capture::interface;
use capture::packet_handler;
use model::AppState;

#[derive(Parser)]
#[command(name = "wifimonitor-tui")]
#[command(about = "A btop-style WiFi monitor TUI using monitor mode")]
struct Cli {
    /// Wireless interface to use (auto-detected if not specified)
    #[arg(short, long)]
    interface: Option<String>,

    /// Skip enabling monitor mode (assume already in monitor mode)
    #[arg(long)]
    no_monitor: bool,
}

fn main() {
    let cli = Cli::parse();

    // Check for root privileges
    if unsafe { libc::geteuid() } != 0 {
        eprintln!("Error: wifimonitor-tui requires root privileges for monitor mode.");
        eprintln!("Run with: sudo wifimonitor-tui");
        std::process::exit(1);
    }

    // Determine interface
    let iface = match cli.interface {
        Some(i) => i,
        None => {
            let interfaces = interface::detect_wireless_interfaces();
            if interfaces.is_empty() {
                eprintln!("Error: No wireless interfaces found.");
                eprintln!("Make sure your USB WiFi adapter is connected.");
                std::process::exit(1);
            }
            // Prefer interfaces already in monitor mode
            if let Some(mon) = interfaces.iter().find(|i| interface::is_monitor_mode(i)) {
                mon.clone()
            } else {
                interfaces[0].clone()
            }
        }
    };

    // Enable monitor mode unless skipped
    let monitor_iface = if cli.no_monitor {
        if !interface::is_monitor_mode(&iface) {
            eprintln!("Warning: --no-monitor specified but {} is not in monitor mode", iface);
        }
        iface.clone()
    } else {
        match interface::enable_monitor_mode(&iface) {
            Ok(i) => {
                eprintln!("Monitor mode enabled on {}", i);
                i
            }
            Err(e) => {
                eprintln!("Error enabling monitor mode: {e}");
                std::process::exit(1);
            }
        }
    };

    let state = Arc::new(Mutex::new(AppState::new(monitor_iface.clone())));

    // If we already set monitor mode via iw, don't ask pcap to also set rfmon.
    // Only use pcap rfmon if --no-monitor was passed (user manages it themselves).
    let use_rfmon = false;

    // Start capture thread
    let capture_handle = packet_handler::start_capture(&monitor_iface, Arc::clone(&state), use_rfmon);

    // Start channel hopper thread
    let hopper_handle = channel_hopper::start_hopper(&monitor_iface, Arc::clone(&state));

    // Run TUI on main thread
    let result = app::run(Arc::clone(&state));

    // Signal threads to stop
    state.lock().unwrap().stop();

    // Wait for threads
    let _ = capture_handle.join();
    let _ = hopper_handle.join();

    // Restore managed mode unless --no-monitor was used
    if !cli.no_monitor {
        interface::disable_monitor_mode(&iface);
        eprintln!("Restored managed mode on {}", iface);
    }

    // Print any errors that occurred during capture
    if let Some(err) = &state.lock().unwrap().error {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }

    if let Err(e) = result {
        eprintln!("TUI error: {e}");
        std::process::exit(1);
    }
}
