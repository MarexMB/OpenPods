//! System BlueZ only. No scanning, pairing, profile changes or accessory sockets.
use crate::{battery::Battery, model::{adapter_state, name_hint, Adapter, Availability, Device, Snapshot}};
use futures_util::StreamExt;
use std::{collections::HashMap, hash::{Hash, Hasher}, time::Duration};
use tokio::sync::watch;
use zbus::{zvariant::{OwnedObjectPath, OwnedValue}, Connection, MatchRule, MessageStream, Proxy};

type Properties = HashMap<String, OwnedValue>;
type Objects = HashMap<OwnedObjectPath, HashMap<String, Properties>>;
const TIMEOUT: Duration = Duration::from_secs(15);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackendError { Bus, Service, Permission, Timeout, DeviceGone, Operation }

impl BackendError {
    pub fn message(self) -> &'static str {
        match self {
            Self::Bus => "System D-Bus unavailable",
            Self::Service => "BlueZ service unavailable",
            Self::Permission => "Bluetooth operation denied by system policy",
            Self::Timeout => "Bluetooth operation timed out; check current state before retrying",
            Self::DeviceGone => "Device no longer paired or selector is unknown; refresh devices",
            Self::Operation => "BlueZ operation failed; check adapter, rfkill and device readiness",
        }
    }
    fn availability(self) -> Availability {
        match self {
            Self::Bus => Availability::BusUnavailable,
            Self::Permission => Availability::PermissionDenied,
            _ => Availability::ServiceUnavailable,
        }
    }
}

fn classify(error: &zbus::Error) -> BackendError {
    // Inspect the error name, never print error bodies (which can contain MACs).
    if let zbus::Error::MethodError(name, _, _) = error {
        match name.as_str() {
            "org.freedesktop.DBus.Error.AccessDenied" | "org.bluez.Error.NotAuthorized" => return BackendError::Permission,
            "org.freedesktop.DBus.Error.ServiceUnknown" | "org.freedesktop.DBus.Error.NameHasNoOwner" => return BackendError::Service,
            _ => {}
        }
    }
    BackendError::Operation
}

async fn bounded<T>(future: impl std::future::Future<Output = zbus::Result<T>>) -> Result<T, BackendError> {
    tokio::time::timeout(TIMEOUT, future).await.map_err(|_| BackendError::Timeout)?.map_err(|e| classify(&e))
}

async fn connection() -> Result<Connection, BackendError> {
    tokio::time::timeout(TIMEOUT, Connection::system()).await.map_err(|_| BackendError::Timeout)?
        .map_err(|_| BackendError::Bus)
}

async fn objects(connection: &Connection) -> Result<Objects, BackendError> {
    let proxy = bounded(Proxy::new(connection, "org.bluez", "/", "org.freedesktop.DBus.ObjectManager")).await?;
    bounded(proxy.call("GetManagedObjects", &())).await
}

fn boolean(properties: &Properties, key: &str) -> bool {
    properties.get(key).and_then(|v| bool::try_from(v).ok()).unwrap_or(false)
}

fn text(properties: &Properties, key: &str) -> Option<String> {
    properties.get(key).and_then(|v| <&str>::try_from(v).ok()).map(str::to_owned)
}

fn selector(path: &str) -> String {
    let mut hash = std::collections::hash_map::DefaultHasher::new();
    path.hash(&mut hash);
    format!("device-{:016x}", hash.finish())
}

#[derive(Default)]
struct State {
    batteries: HashMap<String, Battery>,
}

impl State {
    fn snapshot(&mut self, objects: Objects, battery_event: Option<&str>, now: u64) -> Snapshot {
        let mut adapters = Vec::new();
        let mut devices = Vec::new();
        for (path, interfaces) in objects {
            if let Some(p) = interfaces.get("org.bluez.Adapter1") {
                adapters.push(Adapter { powered: boolean(p, "Powered") });
            }
            let Some(p) = interfaces.get("org.bluez.Device1") else { continue };
            if !boolean(p, "Paired") { continue; }
            let id = selector(path.as_str());
            let name = text(p, "Alias").or_else(|| text(p, "Name")).unwrap_or_else(|| "Paired Bluetooth device".into());
            let value = interfaces.get("org.bluez.Battery1").and_then(|p| p.get("Percentage"))
                .and_then(|v| u8::try_from(v).ok());
            let battery = self.batteries.entry(id.clone()).or_default();
            battery.observe(value, now, battery_event == Some(path.as_str()));
            devices.push(Device { id, airpods_name_hint: name_hint(&name), name, connected: boolean(p, "Connected"), battery: battery.clone() });
        }
        devices.sort_by(|a, b| a.id.cmp(&b.id));
        self.batteries.retain(|id, _| devices.iter().any(|d| &d.id == id));
        let availability = adapter_state(&adapters.iter().map(|a| a.powered).collect::<Vec<_>>());
        Snapshot { availability, adapters, devices }
    }
}

pub async fn snapshot() -> Result<Snapshot, BackendError> {
    let connection = connection().await?;
    Ok(State::default().snapshot(objects(&connection).await?, None, crate::now()))
}

pub async fn set_connected(id: &str, connect: bool) -> Result<(), BackendError> {
    let connection = connection().await?;
    let objects = objects(&connection).await?;
    let path = objects.iter().find(|(path, interfaces)| selector(path.as_str()) == id && interfaces.get("org.bluez.Device1").is_some_and(|p| boolean(p, "Paired")))
        .map(|(path, _)| path).ok_or(BackendError::DeviceGone)?;
    let proxy = bounded(Proxy::new(&connection, "org.bluez", path.as_str(), "org.bluez.Device1")).await?;
    bounded(proxy.call::<_, _, ()>(if connect { "Connect" } else { "Disconnect" }, &())).await
}

async fn cycle(output: &watch::Sender<Snapshot>) -> Result<(), BackendError> {
    let connection = connection().await?;
    // Subscribe before enumeration so changes during startup are not lost.
    let rule = MatchRule::builder().msg_type(zbus::message::Type::Signal)
        .sender("org.bluez").map_err(|_| BackendError::Operation)?.build();
    let mut signals = bounded(MessageStream::for_match_rule(rule, &connection, Some(128))).await?;
    let owner_rule = MatchRule::builder().msg_type(zbus::message::Type::Signal)
        .sender("org.freedesktop.DBus").map_err(|_| BackendError::Operation)?
        .interface("org.freedesktop.DBus").map_err(|_| BackendError::Operation)?
        .member("NameOwnerChanged").map_err(|_| BackendError::Operation)?
        .add_arg("org.bluez").map_err(|_| BackendError::Operation)?.build();
    let mut owners = bounded(MessageStream::for_match_rule(owner_rule, &connection, Some(8))).await?;
    let mut state = State::default();
    output.send_replace(state.snapshot(objects(&connection).await?, None, crate::now()));
    loop {
        let message = tokio::select! {
            _ = owners.next() => return Err(BackendError::Service),
            message = signals.next() => message.ok_or(BackendError::Bus)?.map_err(|e| classify(&e))?,
        };
        let header = message.header();
        let member = header.member().map(|m| m.as_str()).unwrap_or("");
        if !matches!(member, "PropertiesChanged" | "InterfacesAdded" | "InterfacesRemoved") { continue; }
        let battery_event = if member == "PropertiesChanged" {
            message.body().deserialize::<(String, Properties, Vec<String>)>().ok()
                .filter(|(interface, changed, _)| interface == "org.bluez.Battery1" && changed.contains_key("Percentage"))
                .and_then(|_| header.path().map(|p| p.as_str().to_owned()))
        } else { None };
        let next = state.snapshot(objects(&connection).await?, battery_event.as_deref(), crate::now());
        output.send_if_modified(|current| { if *current == next { false } else { *current = next; true } });
    }
}

/// Cancellation drops all streams and the system bus connection. No worker survives it.
pub async fn monitor(output: watch::Sender<Snapshot>, mut stop: watch::Receiver<bool>) {
    let work = async {
        let mut delay = 1;
        loop {
            let started = std::time::Instant::now();
            if let Err(error) = cycle(&output).await {
                output.send_replace(Snapshot::unavailable(error.availability()));
            }
            if started.elapsed() > Duration::from_secs(60) { delay = 1; }
            tokio::time::sleep(Duration::from_secs(delay)).await;
            delay = (delay * 2).min(30);
        }
    };
    tokio::select! { _ = work => {}, _ = async { while !*stop.borrow_and_update() { if stop.changed().await.is_err() { break; } } } => {} }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture(connected: bool, percent: Option<u8>) -> Objects {
        let mut properties = Properties::new();
        properties.insert("Paired".into(), true.into());
        properties.insert("Connected".into(), connected.into());
        let mut interfaces = HashMap::from([("org.bluez.Device1".into(), properties)]);
        if let Some(percent) = percent { interfaces.insert("org.bluez.Battery1".into(), HashMap::from([("Percentage".into(), percent.into())])); }
        HashMap::from([(OwnedObjectPath::try_from("/org/bluez/hci0/dev_00_11_22_33_44_55").unwrap(), interfaces)])
    }
    #[test]
    fn real_object_shapes_transition_and_removal() {
        let mut state = State::default();
        let first = state.snapshot(fixture(true, Some(75)), None, 10);
        assert!(first.devices[0].connected);
        assert!(!first.devices[0].id.contains("00_11"));
        let second = state.snapshot(fixture(false, None), None, 20);
        assert!(!second.devices[0].connected);
        assert!(second.devices[0].battery.aggregate.is_none());
        assert!(state.snapshot(HashMap::new(), None, 30).devices.is_empty());
        assert!(state.batteries.is_empty());
    }
    #[tokio::test]
    async fn cancelled_monitor_exits_without_bluetooth() {
        let (output, _) = watch::channel(Snapshot::unavailable(Availability::BusUnavailable));
        let (_, stop) = watch::channel(true);
        tokio::time::timeout(Duration::from_millis(200), monitor(output, stop)).await.unwrap();
    }

    #[test]
    fn malformed_battery_never_becomes_a_reading() {
        let mut state = State::default();
        assert!(state.snapshot(fixture(true, Some(255)), None, 0).devices[0].battery.aggregate.is_none());
    }
}
