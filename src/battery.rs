use serde::{Deserialize, Serialize};

pub const STALE_AFTER_SECS: u64 = 300;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Charging {
    Unknown,
    Charging,
    NotCharging,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Reading {
    // Private: all inputs must go through validation.
    percent: u8,
    pub observed_at: u64,
    pub charging: Charging,
}

impl Reading {
    pub fn bluez(percent: u8, observed_at: u64) -> Option<Self> {
        (percent <= 100).then_some(Self {
            percent,
            observed_at,
            charging: Charging::Unknown,
        })
    }

    pub fn percent(&self) -> u8 {
        self.percent
    }

    pub fn fresh(&self, now: u64) -> bool {
        self.percent <= 100
            && now >= self.observed_at
            && now - self.observed_at <= STALE_AFTER_SECS
    }

    pub fn display(&self, now: u64, connected: bool) -> String {
        let state = if connected && self.fresh(now) { "" } else { " · stale" };
        format!("{}%{} · BlueZ Battery1 · observed {} · charging unknown", self.percent, state, self.observed_at)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Battery {
    pub aggregate: Option<Reading>,
    pub left: Option<Reading>,
    pub right: Option<Reading>,
    pub case: Option<Reading>,
}

impl Battery {
    /// A repeated snapshot is not a new physical measurement. Only an actual
    /// Battery1 property signal or changed value advances observation time.
    pub fn observe(&mut self, value: Option<u8>, now: u64, property_event: bool) {
        match value.and_then(|v| Reading::bluez(v, now)) {
            Some(reading) => {
                if property_event || self.aggregate.as_ref().map(Reading::percent) != Some(reading.percent()) {
                    self.aggregate = Some(reading);
                }
            }
            None => self.aggregate = None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounds_unknown_and_components() {
        assert!(Reading::bluez(101, 0).is_none());
        assert!(Reading::bluez(255, 0).is_none());
        for n in [0, 50, 100] { assert_eq!(Reading::bluez(n, 0).unwrap().percent(), n); }
        let mut battery = Battery::default();
        battery.observe(Some(42), 10, false);
        assert!(battery.left.is_none() && battery.right.is_none() && battery.case.is_none());
        assert_eq!(battery.aggregate.unwrap().charging, Charging::Unknown);
    }

    #[test]
    fn snapshots_do_not_refresh_and_invalidated_is_unknown() {
        let mut battery = Battery::default();
        battery.observe(Some(42), 10, false);
        battery.observe(Some(42), 100, false);
        assert_eq!(battery.aggregate.as_ref().unwrap().observed_at, 10);
        battery.observe(Some(42), 110, true);
        assert_eq!(battery.aggregate.as_ref().unwrap().observed_at, 110);
        battery.observe(None, 120, false);
        assert!(battery.aggregate.is_none());
    }

    #[test]
    fn freshness_and_formatting() {
        let r = Reading::bluez(0, 100).unwrap();
        assert!(!r.fresh(99));
        assert!(r.fresh(400));
        assert!(!r.fresh(401));
        assert!(r.display(100, false).contains("stale"));
        assert!(r.display(100, true).starts_with("0%"));
    }
}
