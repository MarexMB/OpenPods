# OpenPods contribution guide

Preserve LICENSE and its attribution. Original code is MIT; do not copy GPL/AGPL implementations.
Do not commit or push unless the user authorizes it. Do not install packages, enable services,
or modify Bluetooth/system settings without permission.

Rust core modules must compile without GTK or Linux. Linux-only integrations use cfg gates.
BlueZ owns pairing, connections and battery data; no parallel Bluetooth stack. UI and CLI use
the same backend. Never manufacture live values or infer model capabilities from device names.
No protocol constants without linked evidence and fixtures. Aggregate battery is not per-ear battery.
Keep automatic media actions and notifications disabled by default. No raw device paths in logs.

Checks: `cargo fmt --check`, `cargo test --no-default-features`,
`cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --release`.
Linux native checks need GTK4/libadwaita development files. Windows checks cover the portable core only.
Record commands and outcomes in docs/VALIDATION.md. Hardware validation must identify model and firmware.
Keep docs/COMPATIBILITY.md honest: implemented, automated coverage and physical verification are separate.
