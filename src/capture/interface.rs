use std::process::Command;

/// Detect wireless interfaces that support monitor mode.
pub fn detect_wireless_interfaces() -> Vec<String> {
    let output = Command::new("iw")
        .args(["dev"])
        .output()
        .expect("Failed to run 'iw dev'. Is iw installed?");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut interfaces = Vec::new();

    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("Interface ") {
            if let Some(name) = trimmed.strip_prefix("Interface ") {
                interfaces.push(name.to_string());
            }
        }
    }

    interfaces
}

/// Check if an interface is already in monitor mode.
pub fn is_monitor_mode(interface: &str) -> bool {
    let output = Command::new("iw")
        .args(["dev", interface, "info"])
        .output()
        .ok();

    if let Some(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        stdout.contains("type monitor")
    } else {
        false
    }
}

/// Enable monitor mode on an interface.
/// Returns the interface name (may be unchanged or a new mon interface).
pub fn enable_monitor_mode(interface: &str) -> Result<String, String> {
    if is_monitor_mode(interface) {
        return Ok(interface.to_string());
    }

    // Bring interface down
    let status = Command::new("ip")
        .args(["link", "set", interface, "down"])
        .status()
        .map_err(|e| format!("Failed to run 'ip link set down': {e}"))?;
    if !status.success() {
        return Err(format!("Failed to bring {interface} down"));
    }

    // Set monitor mode
    let status = Command::new("iw")
        .args(["dev", interface, "set", "monitor", "none"])
        .status()
        .map_err(|e| format!("Failed to run 'iw set monitor': {e}"))?;
    if !status.success() {
        return Err(format!("Failed to set monitor mode on {interface}"));
    }

    // Bring interface back up
    let status = Command::new("ip")
        .args(["link", "set", interface, "up"])
        .status()
        .map_err(|e| format!("Failed to run 'ip link set up': {e}"))?;
    if !status.success() {
        return Err(format!("Failed to bring {interface} up"));
    }

    Ok(interface.to_string())
}

/// Restore managed mode on an interface.
pub fn disable_monitor_mode(interface: &str) {
    let _ = Command::new("ip")
        .args(["link", "set", interface, "down"])
        .status();
    let _ = Command::new("iw")
        .args(["dev", interface, "set", "type", "managed"])
        .status();
    let _ = Command::new("ip")
        .args(["link", "set", interface, "up"])
        .status();
}
