//! Run with: dbus-run-session -- cargo test --test bluez_bus -- --ignored
//! A private D-Bus fixture, never a connection to real Bluetooth hardware.
#![cfg(target_os = "linux")]
use openpods::{bluetooth, model::{Availability, Snapshot}};
use std::time::Duration;
use tokio::sync::watch;
use zbus::object_server::SignalEmitter;

struct Adapter;
#[zbus::interface(name = "org.bluez.Adapter1")]
impl Adapter {
    #[zbus(property)]
    fn powered(&self) -> bool { true }
}

#[derive(Default)]
struct Device { connected: bool }
#[zbus::interface(name = "org.bluez.Device1")]
impl Device {
    #[zbus(property)]
    fn paired(&self) -> bool { true }
    #[zbus(property)]
    fn alias(&self) -> &str { "Fixture AirPods — NOT hardware" }
    #[zbus(property)]
    fn connected(&self) -> bool { self.connected }
    async fn connect(&mut self, #[zbus(signal_emitter)] emitter: SignalEmitter<'_>) -> zbus::fdo::Result<()> {
        self.connected = true;
        self.connected_changed(&emitter).await?;
        Ok(())
    }
    async fn disconnect(&mut self, #[zbus(signal_emitter)] emitter: SignalEmitter<'_>) -> zbus::fdo::Result<()> {
        self.connected = false;
        self.connected_changed(&emitter).await?;
        Ok(())
    }
}

struct Battery;
#[zbus::interface(name = "org.bluez.Battery1")]
impl Battery {
    #[zbus(property)]
    fn percentage(&self) -> u8 { 64 }
}

#[tokio::test]
#[ignore = "requires isolated dbus-run-session; never run against the system bus"]
async fn object_manager_connection_events_and_clean_shutdown() {
    let address = std::env::var("DBUS_SESSION_BUS_ADDRESS").expect("Use dbus-run-session");
    // Only this integration-test executable modifies this variable. No production override.
    std::env::set_var("DBUS_SYSTEM_BUS_ADDRESS", address);
    let path = "/org/bluez/hci0/dev_00_11_22_33_44_55";
    let service = zbus::connection::Builder::system().unwrap()
        .name("org.bluez").unwrap()
        .serve_at("/", zbus::fdo::ObjectManager).unwrap()
        .serve_at("/org/bluez/hci0", Adapter).unwrap()
        .serve_at(path, Device::default()).unwrap()
        .serve_at(path, Battery).unwrap()
        .build().await.unwrap();
    let first = bluetooth::snapshot().await.unwrap();
    assert_eq!(first.availability, Availability::Ready);
    assert_eq!(first.devices.len(), 1);
    assert_eq!(first.devices[0].battery.aggregate.as_ref().unwrap().percent(), 64);
    assert!(first.devices[0].battery.left.is_none());
    assert!(!first.devices[0].connected);
    let id = first.devices[0].id.clone();
    let (output, mut receiver) = watch::channel(Snapshot::unavailable(Availability::BusUnavailable));
    let (stop, cancelled) = watch::channel(false);
    let task = tokio::spawn(bluetooth::monitor(output, cancelled));
    tokio::time::timeout(Duration::from_secs(3), receiver.changed()).await.unwrap().unwrap();
    bluetooth::set_connected(&id, true).await.unwrap();
    tokio::time::timeout(Duration::from_secs(3), async {
        loop {
            if receiver.borrow_and_update().devices.first().is_some_and(|d| d.connected) { break; }
            receiver.changed().await.unwrap();
        }
    }).await.unwrap();
    bluetooth::set_connected(&id, false).await.unwrap();
    assert!(!bluetooth::snapshot().await.unwrap().devices[0].connected);
    service.object_server().remove::<Battery, _>(path).await.unwrap();
    assert!(bluetooth::snapshot().await.unwrap().devices[0].battery.aggregate.is_none());
    service.release_name("org.bluez").await.unwrap();
    tokio::time::timeout(Duration::from_secs(3), async {
        loop {
            if receiver.borrow_and_update().availability == Availability::ServiceUnavailable { break; }
            receiver.changed().await.unwrap();
        }
    }).await.unwrap();
    service.request_name("org.bluez").await.unwrap();
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            if receiver.borrow_and_update().availability == Availability::Ready { break; }
            receiver.changed().await.unwrap();
        }
    }).await.unwrap();
    stop.send(true).unwrap();
    tokio::time::timeout(Duration::from_secs(1), task).await.unwrap().unwrap();
}
