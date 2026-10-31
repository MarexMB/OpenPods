use crate::{model::{Availability, Snapshot}, settings::Settings};
use std::collections::HashMap;

pub const COOLDOWN: u64 = 1800;

#[derive(Default)]
struct DeviceAlerts {
    connected: Option<bool>,
    low: bool,
    last_battery: Option<u64>,
    last_connection: Option<u64>,
}

#[derive(Default)]
pub struct Rules { devices: HashMap<String, DeviceAlerts> }

#[derive(Debug, PartialEq, Eq)]
pub struct Alert { pub title: &'static str, pub body: String }

fn elapsed(last: Option<u64>, now: u64, cooldown: u64) -> bool {
    last.is_none_or(|last| now.checked_sub(last).is_some_and(|n| n >= cooldown))
}

impl Rules {
    /// No initial connection notifications. Only fresh, connected battery data
    /// can trigger a low alert; a recovery above threshold+5 re-arms the rule.
    pub fn evaluate(&mut self, snapshot: &Snapshot, settings: &Settings, now: u64) -> Vec<Alert> {
        let mut alerts = Vec::new();
        if snapshot.availability != Availability::Ready { return alerts; }
        for device in &snapshot.devices {
            let state = self.devices.entry(device.id.clone()).or_default();
            if settings.connection_notifications && state.connected.is_some_and(|old| old != device.connected)
                && elapsed(state.last_connection, now, 30) {
                alerts.push(Alert { title: "OpenPods connection", body: if device.connected { "A paired device connected".into() } else { "A paired device disconnected".into() } });
                state.last_connection = Some(now);
            }
            state.connected = Some(device.connected);
            if !device.connected { continue; }
            let Some(reading) = device.battery.aggregate.as_ref().filter(|r| r.fresh(now)) else { continue };
            if reading.percent() > settings.low_battery_threshold.saturating_add(5) { state.low = false; }
            if settings.low_battery_notifications && !state.low && reading.percent() <= settings.low_battery_threshold
                && elapsed(state.last_battery, now, COOLDOWN) {
                alerts.push(Alert { title: "OpenPods low battery", body: format!("A paired device reports {}% (BlueZ aggregate)", reading.percent()) });
                state.last_battery = Some(now);
                state.low = true;
            }
        }
        // Limit retained state to currently paired devices. The daemon owns rules,
        // so opening a second GUI cannot duplicate alerts.
        self.devices.retain(|id, _| snapshot.devices.iter().any(|d| &d.id == id));
        alerts
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{battery::Battery, model::Device};
    fn sample(percent: u8, time: u64) -> Snapshot {
        let mut battery = Battery::default();
        battery.observe(Some(percent), time, true);
        Snapshot { availability: Availability::Ready, adapters: vec![], devices: vec![Device { id: "fixture".into(), name: "Test".into(), connected: true, airpods_name_hint: true, battery }] }
    }
    #[test]
    fn disabled_stale_and_duplicate_alerts() {
        let mut rules = Rules::default();
        let settings = Settings { low_battery_notifications: true, ..Settings::default() };
        assert!(rules.evaluate(&sample(10, 0), &Settings::default(), 0).is_empty());
        assert!(rules.evaluate(&sample(10, 0), &settings, 301).is_empty());
        assert_eq!(rules.evaluate(&sample(10, 301), &settings, 301).len(), 1);
        assert!(rules.evaluate(&sample(10, 302), &settings, 302).is_empty());
        assert!(rules.evaluate(&sample(80, 303), &settings, 303).is_empty());
        assert!(rules.evaluate(&sample(10, 304), &settings, 304).is_empty());
        assert_eq!(rules.evaluate(&sample(10, 2200), &settings, 2200).len(), 1);
    }
    #[test]
    fn connection_transition_not_initial_snapshot() {
        let settings = Settings { connection_notifications: true, ..Settings::default() };
        let mut rules = Rules::default();
        let mut snapshot = sample(90, 0);
        assert!(rules.evaluate(&snapshot, &settings, 0).is_empty());
        snapshot.devices[0].connected = false;
        assert_eq!(rules.evaluate(&snapshot, &settings, 1).len(), 1);
        snapshot.devices[0].connected = true;
        assert!(rules.evaluate(&snapshot, &settings, 2).is_empty());
    }
}
