use std::time::Duration;

pub async fn run() {
    println!("OpenPods {} — local diagnostics (Bluetooth IDs redacted)", crate::VERSION);
    println!("Review device names and host-specific details before sharing output.");
    if let Ok(os) = std::fs::read_to_string("/etc/os-release") {
        for line in os.lines().filter(|l| l.starts_with("PRETTY_NAME=") || l.starts_with("ID=")) {
            println!("{}", crate::privacy::redact(line));
        }
    }
    for key in ["XDG_CURRENT_DESKTOP", "XDG_SESSION_TYPE", "WAYLAND_DISPLAY"] {
        println!("{key}: {}", crate::privacy::redact(&std::env::var(key).unwrap_or_else(|_| "unset".into())));
    }
    for (program, args) in [
        ("pacman", vec!["-Q", "bluez", "bluez-utils"]),
        ("systemctl", vec!["is-active", "bluetooth.service"]),
        ("rfkill", vec!["list", "bluetooth"]),
        ("bluetoothctl", vec!["show"]),
        ("bluetoothctl", vec!["devices", "Paired"]),
    ] {
        println!("\n$ {program} {}", args.join(" "));
        let mut command = tokio::process::Command::new(program);
        command.args(args).kill_on_drop(true).env("LC_ALL", "C");
        match tokio::time::timeout(Duration::from_secs(5), command.output()).await {
            Ok(Ok(output)) => {
                println!("exit: {}", output.status);
                print!("{}", crate::privacy::redact(&String::from_utf8_lossy(&output.stdout)));
                print!("{}", crate::privacy::redact(&String::from_utf8_lossy(&output.stderr)));
            },
            Ok(Err(_)) => println!("Command unavailable or could not start"),
            Err(_) => println!("Timed out; diagnostic subprocess stopped"),
        }
    }
    println!("\nNo settings were changed. Tray: not implemented; use the application window.\nAdvanced protocol controls: unavailable. Physical CachyOS/AirPods verification: pending.");
}
