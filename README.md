# OpenPods

Native, local Bluetooth and AirPods monitoring. **CachyOS Desktop is the primary
support target**, with KDE Plasma 6 on Wayland first. GTK4/libadwaita provides the UI;
Rust and system BlueZ provide device state. No accounts, cloud services or telemetry.

**Development status: implemented source, not yet build- or hardware-validated.**
This repository began with only a README and MIT license. Development took place on
Windows without Cargo or WSL. No physical AirPods have been tested. The definition of
done is not yet met; compilation, formatting, native UI and package checks remain open.
See the precise [validation record](docs/VALIDATION.md), not CI configuration, for results.

## Implemented

- BlueZ paired-device enumeration, adapter availability, connection state and events.
- Connect/disconnect via Device1, without replacing pairing or audio management.
- Actual available Battery1 aggregate percentage, observation time and stale status.
  Missing left/right/case and charging data stays unavailable; no synthetic values.
- Native device selector, empty/error states, system/light/dark appearance, settings,
  monitoring start/stop, and explicit user-service enable/disable actions.
- CLI status, devices, diagnostics, connection actions and notification daemon.
- Opt-in low-battery and connection notifications with deduplication and throttling.
- Desktop entry, original icon, optional systemd user unit and reviewable local PKGBUILD.
- Portable unit tests, BlueZ object fixtures and an isolated D-Bus integration test.

All paired devices appear, including renamed AirPods. Names are only hints; no model
identifier is verified. Battery availability depends on BlueZ/device firmware.
ANC, transparency, adaptive audio, Apple accessory protocol, BLE component readings,
ear detection, MPRIS, audio-output status and tray integration are **not implemented**.
The application window is the explicit tray fallback. See [compatibility](docs/COMPATIBILITY.md).

## Build and run on CachyOS

Full prerequisites, package-name evidence, Bluetooth troubleshooting, KDE/Wayland notes,
user-service management and clean removal are in [docs/CACHYOS.md](docs/CACHYOS.md).
These commands are supplied for native validation; **they have not passed locally**.

```sh
sudo pacman -S --needed rust pkgconf gtk4 libadwaita bluez bluez-utils
cargo build --release
./target/release/openpods
./target/release/openpods devices
./target/release/openpods diagnose
```

Use `openpods connect <device-id>` / `disconnect <device-id>` with an ID from `devices`.
`openpods status` and `openpods version` are also available. A headless build uses
`cargo build --release --no-default-features`. Non-Linux builds only support portable
tests and version/help; live Linux functionality is not emulated.

For a local package, start from a clean committed checkout:

```sh
bash scripts/package.sh
cd .package
# Review PKGBUILD and the archived source first.
makepkg -si
openpods
```

The package installs an **OpenPods** application-menu entry. Never run makepkg as root.
No AUR or CachyOS repository publication is claimed. Cargo.lock is not generated yet;
dependency resolution and reproducible release validation remain blockers. The source
archive is checksummed, but this alone does not make the binary reproducible.

## Background notifications

After package installation, enable notification preferences in Settings, then explicitly
start the user service (Settings offers the same action):

```sh
systemctl --user enable --now openpods.service
# Disable when no longer wanted:
systemctl --user disable --now openpods.service
```

The service runs without a GUI or root, and is the sole OpenPods notification sender.
It does not enable Bluetooth, pair devices, change audio profiles or require a particular
CachyOS kernel. Closing the GUI stops that window's monitor; the optional service remains
until stopped. Settings live at `$XDG_CONFIG_HOME/openpods/settings.json`, defaulting to
`~/.config/openpods/settings.json`. Automatic playback changes are unavailable and off.

## Development and verification

```sh
cargo fmt --check
cargo test --no-default-features
cargo test --all-features
dbus-run-session -- cargo test --test bluez_bus -- --ignored
cargo clippy --all-targets --all-features -- -D warnings
```

The private-bus test contains labelled fixtures and must not target the real system bus.
No test fixture is exposed through the normal UI. See [hardware checklist](docs/HARDWARE_TESTING.md),
[architecture](docs/ARCHITECTURE.md), [contributing](CONTRIBUTING.md) and
[third-party/protocol provenance](docs/PROTOCOL_RESEARCH.md).

MIT © 2026 Marko Borocki. The original [LICENSE](LICENSE) is preserved.
