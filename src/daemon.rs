use crate::{bluetooth, model::{Availability, Snapshot}, notifications::Rules, settings::Settings};
use std::{collections::HashMap, time::Duration};
use tokio::sync::watch;
use zbus::{zvariant::Value, Connection, Proxy};

async fn notify(connection: &Connection, title: &str, body: &str) -> Result<(), ()> {
    let operation = async {
        let proxy = Proxy::new(connection, "org.freedesktop.Notifications", "/org/freedesktop/Notifications", "org.freedesktop.Notifications").await?;
        proxy.call::<_, _, u32>("Notify", &("OpenPods", 0u32, crate::APP_ID, title, body, Vec::<String>::new(), HashMap::<String, Value<'_>>::new(), -1i32)).await
    };
    tokio::time::timeout(Duration::from_secs(5), operation).await.map_err(|_| ())?.map(|_| ()).map_err(|_| ())
}

pub async fn run() -> Result<(), &'static str> {
    let connection = zbus::connection::Builder::session().map_err(|_| "Session D-Bus unavailable")?
        .name("io.github.MarexMB.OpenPods.Monitor").map_err(|_| "Invalid application ID")?
        .build().await.map_err(|_| "Cannot own monitor name; daemon already running or session bus unavailable")?;
    let (output, mut state) = watch::channel(Snapshot::unavailable(Availability::ServiceUnavailable));
    let (stop, cancelled) = watch::channel(false);
    let monitor = tokio::spawn(bluetooth::monitor(output, cancelled));
    let mut rules = Rules::default();
    let mut reload = tokio::time::interval(Duration::from_secs(30));
    let mut term = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()).map_err(|_| "Cannot register shutdown signal")?;
    let mut settings = Settings::default();
    let mut settings_failed = false;
    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => break,
            _ = term.recv() => break,
            changed = state.changed() => { if changed.is_err() { break; } },
            _ = reload.tick() => {},
        }
        match Settings::path().and_then(|path| Settings::load(&path)) {
            Ok(next) => { settings = next; settings_failed = false; },
            Err(_) => {
                settings.low_battery_notifications = false;
                settings.connection_notifications = false;
                if !settings_failed { eprintln!("Settings unavailable or invalid; notifications disabled"); }
                settings_failed = true;
            }
        }
        let snapshot = state.borrow_and_update().clone();
        for alert in rules.evaluate(&snapshot, &settings, crate::now()) {
            if notify(&connection, alert.title, &alert.body).await.is_err() {
                eprintln!("Desktop notification delivery failed; rule remains throttled");
            }
        }
    }
    let _ = stop.send(true);
    let _ = monitor.await;
    Ok(())
}
