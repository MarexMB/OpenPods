# CachyOS installation and validation

CachyOS Desktop is the primary target, KDE Plasma 6 / Wayland first, then KDE X11,
GNOME and other CachyOS desktops. **This guide has not been executed on CachyOS.**
Development host: Windows, no installed WSL distribution, no Cargo. Source implementation
and authored tests must not be confused with a validated binary. No hardware verification.

## Inspect the actual machine first

```sh
cat /etc/os-release
printf '%s\n' "$XDG_CURRENT_DESKTOP" "$XDG_SESSION_TYPE" "$WAYLAND_DISPLAY"
pacman -Q bluez bluez-utils
systemctl status bluetooth.service
rfkill list
bluetoothctl show
bluetoothctl devices Paired
```

These manual commands can display Bluetooth identifiers. Do not publish their raw output.
Once built, `openpods diagnose` runs bounded, read-only checks with MAC/path redaction;
review remaining device names and host details before sharing. Environment variables
describe a session and never authorize operations.

## Prerequisites — user-directed actions only

Use CachyOS's configured repositories and **pacman**. Package names were checked in the
real [CachyOS package database](https://packages.cachyos.org/), including:

- [bluez](https://packages.cachyos.org/package/extra/x86_64/bluez) and [bluez-utils](https://packages.cachyos.org/package/extra/x86_64/bluez-utils)
- [gtk4](https://packages.cachyos.org/package/extra/x86_64/gtk4) and [libadwaita](https://packages.cachyos.org/package/cachyos-extra-v3/x86_64_v3/libadwaita)
- [rust](https://packages.cachyos.org/package/extra/x86_64/rust) and [pkgconf](https://packages.cachyos.org/package/cachyos-core-v3/x86_64_v3/pkgconf)
- [dbus](https://packages.cachyos.org/package/core/x86_64/dbus), [gcc-libs](https://packages.cachyos.org/package/core/x86_64/gcc-libs), [util-linux / rfkill](https://packages.cachyos.org/package/core/x86_64/util-linux)

The optimized v3/v4 listings demonstrate repository variants, not CPU requirements.
Let pacman select packages appropriate for your existing CachyOS configuration. Do not
switch repositories or force architecture-specific binaries for this application.

```sh
# Inspect current configured repository metadata before installing:
pacman -Si rust pkgconf gtk4 libadwaita bluez bluez-utils
# Native compiler and GUI build dependencies:
sudo pacman -S --needed rust pkgconf gtk4 libadwaita bluez bluez-utils
# Only if Bluetooth is missing/stopped and you choose to enable it:
sudo pacman -S --needed bluez bluez-utils
sudo systemctl enable --now bluetooth.service
```

CachyOS Hello may also offer Bluetooth setup. Pair devices through KDE System Settings
or bluetoothctl. OpenPods does not silently execute these commands, change system units,
disable power management, install firmware, spoof device IDs or alter kernel parameters.
No specific CachyOS kernel flavor, bootloader or out-of-tree module is required by this code.
Access to system BlueZ follows existing D-Bus/polkit policy; the GUI must never run as root.

Runtime: system BlueZ, a system and session D-Bus, GTK4 and libadwaita (API >= 1.4).
The selected gtk-rs release may impose a newer GLib minimum; Cargo/pkg-config will check.
Build: stable Rust/Cargo, native C toolchain (pulled by CachyOS's rust package), pkgconf.
The packaging path also needs the normal makepkg toolchain on the machine. Verify it with
`command -v makepkg` and inspect `pacman -Si base-devel` before installing missing tooling.
The base-devel package page could not be verified remotely in this session.

## Native build and tests

From the repository root, as your ordinary desktop user:

```sh
cargo fmt
cargo fmt --check
cargo test --no-default-features
cargo test --all-features
dbus-run-session -- cargo test --test bluez_bus -- --ignored
cargo clippy --all-targets --all-features -- -D warnings
cargo build --release
./target/release/openpods version
./target/release/openpods status
./target/release/openpods
```

Expected executable path: `target/release/openpods`. Commands are **pending execution**,
not reported as passing. The isolated D-Bus test claims its own org.bluez on a private
session bus and uses synthetic fixtures; it is not a hardware test.

No Cargo.lock could be generated without Cargo. Resolve, review and commit a lockfile on
the first supported builder, then use `cargo build --release --locked` and add `--locked`
to PKGBUILD's Cargo commands before a reproducible release. Current packaging is a local
development package, not a reproducibly validated release. No CPU-specific RUSTFLAGS.

## Local package and application menu

Start from a clean committed checkout, so the archive includes exactly reviewed HEAD:

```sh
bash scripts/package.sh
cd .package
# Read PKGBUILD and review the source archive before proceeding.
makepkg -si
openpods
```

`scripts/package.sh` only creates a source archive and checksummed PKGBUILD. It never
installs anything. `makepkg -si` builds/tests and asks pacman to install as needed.
**Never use sudo makepkg.** Neither AUR publication nor repository availability is claimed.

Installed files: `/usr/bin/openpods`, application desktop entry, scalable hicolor icon,
MIT license/docs, and `/usr/lib/systemd/user/openpods.service`. Launch **OpenPods** from
the KDE application menu. Second activation should present the existing window via
GApplication's session-bus ID `io.github.MarexMB.OpenPods`; native confirmation is pending.

## Optional user service

After package installation:

```sh
systemctl --user daemon-reload
systemctl --user enable --now openpods.service
systemctl --user status openpods.service
journalctl --user -u openpods.service
# Stop and remove login activation:
systemctl --user disable --now openpods.service
```

Settings has explicit enable/start and disable/stop buttons invoking the same user-only
commands. Notification rules are off until enabled in Settings. Threshold defaults to
20%; rules currently cover all paired devices. The daemon reloads preferences within
30 seconds. A missing notification host produces a generic error and does not spam retries.
The service is tied to graphical-session.target, not system-wide boot or lingering.
Do not enable it with sudo or modify system-wide service units. SIGINT/SIGTERM shut it down.

## Plasma Wayland and tray behavior

The UI uses ordinary native windows and system appearance. No global positioning,
always-on-top or unrestricted global shortcut is assumed. It does not modify KDE's
Bluetooth panel, PipeWire/WirePlumber routing, codecs or profiles. GTK's standard desktop
integration applies; no custom portal bypass is used.

StatusNotifierItem is **not implemented**. No libappindicator/GTK tray compatibility is
claimed. The explicit fallback is the normal application-menu window plus optional
background notifications. Close the window to stop its monitor; reopen it from the menu.
The notification daemon is separately managed. Verify the fallback on real Plasma and
GNOME before marking desktop support as tested.

## Bluetooth troubleshooting

| Symptom | Distinguish / next check |
| --- | --- |
| System bus unavailable | Verify the desktop session and system D-Bus; do not run the GUI under sudo |
| BlueZ service unavailable | `pacman -Q bluez bluez-utils`; `systemctl status bluetooth.service` |
| No adapter | BlueZ responds but no Adapter1 exists; inspect hardware and OS Bluetooth status |
| Powered off | Adapter exists and Powered is false; inspect KDE toggle and `rfkill list` |
| Soft/hard block | rfkill diagnostics distinguish these; the app does not automatically unblock hardware |
| No paired device | Pair through KDE or bluetoothctl; BLE visibility alone is not a connection |
| Permission denied | Inspect local D-Bus/polkit policy; avoid broad policy changes or running as root |
| Battery unavailable | Device may not expose Battery1; do not infer a protocol failure from this alone |
| Battery stale | No fresh observation for 300s, or device disconnected; BlueZ may retain cached values |
| Operation timeout | Check status before retrying: BlueZ may finish the original operation later |
| No notifications | Enable preferences, check user service and Plasma notification settings |

Connection status is Device1.Connected, not audio output selection. Exact model identity,
noise mode, in-ear state and component battery remain unavailable. No accessory sessions
are opened, so OpenPods does not compete for that proprietary transport.

## Clean removal

```sh
systemctl --user disable --now openpods.service
sudo pacman -R openpods
systemctl --user daemon-reload
```

Close the GUI before removal. Package removal does not delete user preferences. If you
want them removed too, manually delete only `$XDG_CONFIG_HOME/openpods/settings.json`
(normally `~/.config/openpods/settings.json`) after reviewing it. Do not disable system
Bluetooth or remove BlueZ merely because OpenPods is removed; other applications use it.

## CachyOS smoke test — all pending

- [ ] Record /etc/os-release, Plasma/session type, BlueZ/kernel versions, model and firmware.
- [ ] Build, unit tests, isolated D-Bus test, fmt and clippy pass on CachyOS.
- [ ] `makepkg -si` installs and desktop entry/icon launch without root.
- [ ] Paired devices appear; renamed and multiple devices retain truthful connection state.
- [ ] Aggregate Battery1 matches its actual source; unavailable components remain unavailable.
- [ ] Service stopped, adapter absent, powered off and rfkill blocked are diagnosed distinctly.
- [ ] Disconnect/reconnect, BlueZ restart, suspend/resume and removal recover without busy loops.
- [ ] Opt-in notifications work on Plasma; stale/duplicate reports do not repeatedly alert.
- [ ] No-tray window fallback, keyboard access, dark mode and second launch behave correctly.
- [ ] GUI close and service stop leave no hanging OpenPods process or D-Bus owner.
- [ ] Pacman removal cleans packaged files and preserves unrelated Bluetooth configuration.

Use [HARDWARE_TESTING.md](HARDWARE_TESTING.md) for the detailed result template.
