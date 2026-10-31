use crate::battery::Battery;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Availability {
    Ready,
    NoAdapter,
    PoweredOff,
    ServiceUnavailable,
    PermissionDenied,
    BusUnavailable,
    UnsupportedPlatform,
}

impl Availability {
    pub fn description(&self) -> &'static str {
        match self {
            Self::Ready => "Bluetooth ready",
            Self::NoAdapter => "BlueZ is running, but no Bluetooth adapter is available",
            Self::PoweredOff => "Bluetooth adapters are powered off; check KDE Bluetooth and rfkill",
            Self::ServiceUnavailable => "BlueZ service unavailable; check bluetooth.service",
            Self::PermissionDenied => "Bluetooth access denied by system policy",
            Self::BusUnavailable => "System D-Bus unavailable",
            Self::UnsupportedPlatform => "Live Bluetooth requires Linux and system BlueZ",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Adapter {
    pub powered: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Device {
    /// Session-local selector, never a raw Bluetooth address or D-Bus path.
    pub id: String,
    pub name: String,
    pub connected: bool,
    pub airpods_name_hint: bool,
    pub battery: Battery,
}

impl Device {
    pub fn description(&self, now: u64) -> String {
        let connection = if self.connected { "Connected" } else { "Disconnected" };
        let aggregate = self.battery.aggregate.as_ref().map(|r| r.display(now, self.connected))
            .unwrap_or_else(|| "Unavailable".into());
        format!("{connection}\nModel: unverified{}\nAggregate: {aggregate}\nLeft: unavailable · Right: unavailable · Case: unavailable\nNoise / in-ear / audio output: unavailable",
            if self.airpods_name_hint { " (AirPods name hint only)" } else { "" })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Snapshot {
    pub availability: Availability,
    pub adapters: Vec<Adapter>,
    pub devices: Vec<Device>,
}

impl Snapshot {
    pub fn unavailable(availability: Availability) -> Self {
        Self { availability, adapters: vec![], devices: vec![] }
    }
}

pub fn adapter_state(powered: &[bool]) -> Availability {
    if powered.is_empty() { Availability::NoAdapter }
    else if powered.iter().any(|p| *p) { Availability::Ready }
    else { Availability::PoweredOff }
}

pub fn name_hint(name: &str) -> bool { name.to_lowercase().contains("airpods") }

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Capabilities {
    pub noise_control: bool,
    pub in_ear: bool,
    pub component_battery: bool,
}

/// No verified model identifiers or command acknowledgements are bundled yet.
/// A user-controlled alias cannot grant protocol capabilities.
pub fn capabilities(_name: &str) -> Capabilities { Capabilities::default() }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn adapters_and_model_gating() {
        assert_eq!(adapter_state(&[]), Availability::NoAdapter);
        assert_eq!(adapter_state(&[false]), Availability::PoweredOff);
        assert_eq!(adapter_state(&[false, true]), Availability::Ready);
        for name in ["AirPods Pro 3", "AirPods Max", "future model", ""] {
            assert_eq!(capabilities(name), Capabilities::default());
        }
        assert!(name_hint("Marko’s AIRPODS"));
        assert!(!name_hint("Headset"));
    }
}
