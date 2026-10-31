use adw::{gtk::{self, glib}, prelude::*};
use crate::{model::{Availability, Snapshot}, settings::{Settings, Theme}};
use std::{cell::RefCell, rc::Rc, time::Duration};
use tokio::sync::watch;

fn label(text: &str) -> gtk::Label {
    gtk::Label::builder().label(text).wrap(true).xalign(0.0).selectable(true).build()
}

fn theme(value: Theme) {
    adw::StyleManager::default().set_color_scheme(match value {
        Theme::System => adw::ColorScheme::Default,
        Theme::Light => adw::ColorScheme::ForceLight,
        Theme::Dark => adw::ColorScheme::ForceDark,
    });
}

pub fn run() -> std::process::ExitCode {
    let runtime = match tokio::runtime::Runtime::new() {
        Ok(runtime) => runtime,
        Err(_) => { eprintln!("Cannot start background runtime"); return std::process::ExitCode::FAILURE; }
    };
    let handle = runtime.handle().clone();
    let app = adw::Application::builder().application_id(crate::APP_ID).build();
    app.connect_activate(move |app| {
        if let Some(window) = app.active_window() { window.present(); return; }
        build_window(app, &handle);
    });
    // Do not forward CLI subcommands into GApplication's option parser.
    let code = app.run_with_args(&["openpods"]);
    runtime.shutdown_timeout(Duration::from_secs(2));
    if code == glib::ExitCode::SUCCESS { std::process::ExitCode::SUCCESS } else { std::process::ExitCode::FAILURE }
}

fn build_window(app: &adw::Application, runtime: &tokio::runtime::Handle) {
    let window = adw::ApplicationWindow::builder().application(app).title("OpenPods").default_width(620).default_height(650).build();
    let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let header = adw::HeaderBar::new();
    let settings_button = gtk::Button::builder().icon_name("preferences-system-symbolic").tooltip_text("Settings and background service").build();
    header.pack_end(&settings_button);
    outer.append(&header);
    let body = gtk::Box::new(gtk::Orientation::Vertical, 16);
    for side in [&body] { side.set_margin_top(24); side.set_margin_bottom(24); side.set_margin_start(24); side.set_margin_end(24); }
    let title = label("OpenPods");
    title.add_css_class("title-1");
    body.append(&title);
    body.append(&label("Local Bluetooth monitoring · CachyOS / KDE first"));
    let status = label("Connecting to system BlueZ…");
    body.append(&status);
    let model = gtk::StringList::new(&[]);
    let selector = gtk::DropDown::new(Some(model.clone()), None::<gtk::Expression>);
    selector.set_tooltip_text(Some("Select a paired Bluetooth device"));
    body.append(&label("Paired device (names do not verify model identity)"));
    body.append(&selector);
    let detail = label("Pair your AirPods in KDE System Settings → Bluetooth. Renamed devices are included.");
    body.append(&detail);
    let actions = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    let connect = gtk::Button::with_label("Connect");
    connect.set_sensitive(false);
    let monitoring = gtk::Button::with_label("Stop monitoring");
    actions.append(&connect);
    actions.append(&monitoring);
    body.append(&actions);
    let result = label("");
    body.append(&result);
    body.append(&label("Battery1 is an aggregate reading observed from BlueZ, not a guaranteed new hardware measurement. Per-ear and case values remain unavailable."));
    body.append(&label("Tray unavailable in this version. Keep this window open, or enable the optional notification service in Settings. Closing the window stops its monitor."));
    body.append(&label("OpenPods 0.1.0 · MIT · Copyright © 2026 Marko Borocki\nAdvanced noise, ear detection and audio routing controls are unavailable.\nDiagnostics: run openpods diagnose in a terminal."));
    let scroll = gtk::ScrolledWindow::builder().vexpand(true).child(&body).build();
    outer.append(&scroll);
    window.set_content(Some(&outer));

    let loaded = Settings::path().and_then(|p| Settings::load(&p));
    if loaded.is_err() { result.set_text("Settings could not be read. Defaults are active; use Settings to review and save."); }
    theme(loaded.unwrap_or_default().theme);
    let settings_window = window.downgrade();
    let settings_runtime = runtime.clone();
    settings_button.connect_clicked(move |_| {
        if let Some(parent) = settings_window.upgrade() { settings(&parent, &settings_runtime); }
    });

    let state = Rc::new(RefCell::new(Snapshot::unavailable(Availability::ServiceUnavailable)));
    let (output, mut receiver) = watch::channel(state.borrow().clone());
    let (stop, cancelled) = watch::channel(false);
    runtime.spawn(crate::bluetooth::monitor(output.clone(), cancelled));
    let stop = Rc::new(RefCell::new(stop));
    let (updates, ui_updates) = async_channel::bounded(1);
    runtime.spawn(async move {
        while receiver.changed().await.is_ok() {
            let snapshot = receiver.borrow_and_update().clone();
            if updates.send(snapshot).await.is_err() { break; }
        }
    });
    let stopped = Rc::new(std::cell::Cell::new(false));
    let stop_button = stop.clone();
    let monitor_runtime = runtime.clone();
    let monitor_stopped = stopped.clone();
    let monitor_status = status.clone();
    let monitor_connect = connect.clone();
    monitoring.connect_clicked(move |button| {
        if monitor_stopped.get() {
            let (sender, receiver) = watch::channel(false);
            *stop_button.borrow_mut() = sender;
            monitor_runtime.spawn(crate::bluetooth::monitor(output.clone(), receiver));
            monitor_stopped.set(false);
            button.set_label("Stop monitoring");
            monitor_status.set_text("Reconnecting to BlueZ…");
        } else {
            let _ = stop_button.borrow().send(true);
            monitor_stopped.set(true);
            button.set_label("Start monitoring");
            monitor_status.set_text("Monitoring stopped. Displayed values are not live.");
            monitor_connect.set_sensitive(false);
        }
    });
    let close_stop = stop.clone();
    window.connect_close_request(move |_| { let _ = close_stop.borrow().send(true); glib::Propagation::Proceed });

    let selection_state = state.clone();
    let selection_detail = detail.clone();
    let selection_connect = connect.clone();
    let selection_stopped = stopped.clone();
    selector.connect_selected_notify(move |selector| {
        selected(&selection_state.borrow(), selector.selected(), &selection_detail, &selection_connect, selection_stopped.get());
    });

    let action_state = state.clone();
    let action_selector = selector.clone();
    let action_runtime = runtime.clone();
    connect.connect_clicked(move |button| {
        let Some(device) = action_state.borrow().devices.get(action_selector.selected() as usize).cloned() else { return; };
        button.set_sensitive(false);
        result.set_text("Requesting BlueZ connection change…");
        let (send, receive) = async_channel::bounded(1);
        action_runtime.spawn(async move { let _ = send.send(crate::bluetooth::set_connected(&device.id, !device.connected).await).await; });
        let result = result.clone();
        // State changes, not method return values, update the connection display.
        glib::spawn_future_local(async move {
            match receive.recv().await {
                Ok(Ok(())) => result.set_text("Request completed; connection state comes from BlueZ events."),
                Ok(Err(error)) => result.set_text(error.message()),
                Err(_) => result.set_text("Connection worker stopped"),
            }
        });
    });

    let weak = window.downgrade();
    let update_state = state.clone();
    let update_selector = selector.clone();
    let update_detail = detail.clone();
    let update_connect = connect.clone();
    let update_stopped = stopped.clone();
    glib::spawn_future_local(async move {
        while let Ok(snapshot) = ui_updates.recv().await {
            if weak.upgrade().is_none() { break; }
            if update_stopped.get() { continue; }
            status.set_text(snapshot.availability.description());
            let old_id = update_state.borrow().devices.get(update_selector.selected() as usize).map(|d| d.id.clone());
            let names: Vec<String> = snapshot.devices.iter().map(|d| crate::privacy::redact(&d.name)).collect();
            let names: Vec<&str> = names.iter().map(String::as_str).collect();
            let position = snapshot.devices.iter().position(|d| Some(&d.id) == old_id.as_ref()).unwrap_or(0);
            *update_state.borrow_mut() = snapshot;
            model.splice(0, model.n_items(), &names);
            update_selector.set_selected(position as u32);
            selected(&update_state.borrow(), update_selector.selected(), &update_detail, &update_connect, false);
        }
    });
    let weak = window.downgrade();
    glib::timeout_add_local(Duration::from_secs(30), move || {
        if weak.upgrade().is_none() { return glib::ControlFlow::Break; }
        selected(&state.borrow(), selector.selected(), &detail, &connect, stopped.get());
        glib::ControlFlow::Continue
    });
    window.present();
}

fn selected(state: &Snapshot, index: u32, detail: &gtk::Label, button: &gtk::Button, stopped: bool) {
    if let Some(device) = state.devices.get(index as usize) {
        detail.set_text(&device.description(crate::now()));
        button.set_label(if device.connected { "Disconnect" } else { "Connect" });
        button.set_sensitive(!stopped && state.availability == Availability::Ready);
    } else {
        detail.set_text("No paired device available. Pair in KDE System Settings → Bluetooth, or use bluetoothctl.");
        button.set_sensitive(false);
    }
}

fn settings(parent: &adw::ApplicationWindow, runtime: &tokio::runtime::Handle) {
    let window = gtk::Window::builder().title("OpenPods settings").transient_for(parent).modal(true).default_width(480).build();
    let body = gtk::Box::new(gtk::Orientation::Vertical, 14);
    body.set_margin_top(24); body.set_margin_bottom(24); body.set_margin_start(24); body.set_margin_end(24);
    let current = Settings::path().and_then(|p| Settings::load(&p)).unwrap_or_default();
    let low = gtk::CheckButton::with_label("Notify on low aggregate battery");
    low.set_active(current.low_battery_notifications);
    let connection = gtk::CheckButton::with_label("Notify on connection changes");
    connection.set_active(current.connection_notifications);
    let threshold = gtk::SpinButton::with_range(1.0, 100.0, 1.0);
    threshold.set_value(f64::from(current.low_battery_threshold));
    let appearance = gtk::DropDown::from_strings(&["System appearance", "Light", "Dark"]);
    appearance.set_selected(match current.theme { Theme::System => 0, Theme::Light => 1, Theme::Dark => 2 });
    let feedback = label("");
    body.append(&low); body.append(&connection); body.append(&label("Low battery threshold (%)")); body.append(&threshold); body.append(&appearance);
    body.append(&label("Notification rules apply to all paired devices. The background service must be running. Playback automation is unavailable and never enabled."));
    let save = gtk::Button::with_label("Save preferences");
    body.append(&save);
    let save_feedback = feedback.clone();
    save.connect_clicked(move |_| {
        let settings = Settings { low_battery_notifications: low.is_active(), connection_notifications: connection.is_active(), low_battery_threshold: threshold.value_as_int() as u8,
            theme: match appearance.selected() { 1 => Theme::Light, 2 => Theme::Dark, _ => Theme::System } };
        match Settings::path().and_then(|path| settings.save(&path)) {
            Ok(()) => { theme(settings.theme); save_feedback.set_text("Saved. Background service reloads within 30 seconds."); },
            Err(_) => save_feedback.set_text("Could not save settings. Check your XDG configuration directory."),
        }
    });
    body.append(&label("Start at login (installed user service required):"));
    for (title, verb) in [("Enable and start notification service", "enable"), ("Disable and stop notification service", "disable")] {
        let button = gtk::Button::with_label(title);
        let runtime = runtime.clone();
        let feedback = feedback.clone();
        button.connect_clicked(move |button| {
            button.set_sensitive(false);
            let (send, receive) = async_channel::bounded(1);
            runtime.spawn(async move {
                let mut command = tokio::process::Command::new("systemctl");
                command.args(["--user", verb, "--now", "openpods.service"]).kill_on_drop(true);
                let success = matches!(tokio::time::timeout(Duration::from_secs(10), command.output()).await, Ok(Ok(output)) if output.status.success());
                let _ = send.send(success).await;
            });
            let feedback = feedback.clone();
            let button = button.clone();
            glib::spawn_future_local(async move {
                feedback.set_text(if receive.recv().await.unwrap_or(false) { "User service command completed." } else { "Service command failed or timed out. Check systemctl --user status openpods.service." });
                button.set_sensitive(true);
            });
        });
        body.append(&button);
    }
    body.append(&feedback);
    window.set_child(Some(&body));
    window.present();
}
