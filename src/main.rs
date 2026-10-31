use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args == ["version"] || args == ["--version"] {
        println!("OpenPods {}", openpods::VERSION);
        return ExitCode::SUCCESS;
    }
    if args == ["help"] || args == ["--help"] {
        println!("OpenPods\n  openpods [gui]\n  openpods status\n  openpods devices\n  openpods diagnose\n  openpods connect <device-id>\n  openpods disconnect <device-id>\n  openpods daemon\n  openpods version\n\nIDs come from `devices`. All paired devices are shown; names cannot verify AirPods models.\nNotifications are opt-in and require the optional daemon. No advanced protocol controls.");
        return ExitCode::SUCCESS;
    }
    #[cfg(target_os = "linux")]
    { linux(&args) }
    #[cfg(not(target_os = "linux"))]
    {
        eprintln!("{}", openpods::model::Availability::UnsupportedPlatform.description());
        ExitCode::FAILURE
    }
}

#[cfg(target_os = "linux")]
fn linux(args: &[String]) -> ExitCode {
    if args.is_empty() || args == ["gui"] {
        #[cfg(feature = "gui")]
        { return openpods::ui::run(); }
        #[cfg(not(feature = "gui"))]
        { eprintln!("GUI disabled in this build; rebuild with default features"); return ExitCode::FAILURE; }
    }
    let runtime = match tokio::runtime::Runtime::new() { Ok(rt) => rt, Err(_) => { eprintln!("Cannot start async runtime"); return ExitCode::FAILURE; } };
    let result: Result<(), &str> = runtime.block_on(async {
        match args.first().map(String::as_str) {
            Some("status" | "devices") if args.len() == 1 => {
                let state = openpods::bluetooth::snapshot().await.map_err(|e| e.message())?;
                println!("{}", state.availability.description());
                if state.devices.is_empty() { println!("No paired devices. Pair using KDE System Settings or bluetoothctl."); }
                for device in state.devices {
                    println!("\n{} · {}\n{}", device.id, openpods::privacy::redact(&device.name), device.description(openpods::now()));
                }
                Ok(())
            },
            Some("connect" | "disconnect") if args.len() == 2 => {
                openpods::bluetooth::set_connected(&args[1], args[0] == "connect").await.map_err(|e| e.message())?;
                println!("BlueZ request completed. Use status to inspect the current connection state.");
                Ok(())
            },
            Some("diagnose") if args.len() == 1 => { openpods::diagnostics::run().await; Ok(()) },
            Some("daemon") if args.len() == 1 => openpods::daemon::run().await,
            _ => Err("Unknown command or invalid arguments. Run openpods --help."),
        }
    });
    match result { Ok(()) => ExitCode::SUCCESS, Err(message) => { eprintln!("{message}"); ExitCode::FAILURE } }
}
