# Architecture

One Rust package, one executable, shared backend. `openpods` launches GTK4/libadwaita;
CLI subcommands use the same BlueZ module. `openpods daemon` is an optional independent
notification monitor. BlueZ's system D-Bus objects are the authoritative shared state;
there is no OpenPods IPC server or second Bluetooth connection stack. Multiple read-only
monitors are safe. Only the daemon sends notifications, guarded by a session-bus name.
GTK's GApplication ID provides GUI single-instance activation.

| Module | Responsibility |
| --- | --- |
| `model` | Adapter availability, paired-device state, conservative capabilities |
| `battery` | Range validation, aggregate vs components, observation time, freshness |
| `bluetooth` | ObjectManager enumeration, event subscriptions, connect/disconnect |
| `ui` | Native widgets, selection, settings, background task bridge |
| `settings` | Validated XDG JSON, temporary-file replacement |
| `notifications` | Pure opt-in transition, hysteresis and cooldown rules |
| `daemon` | Single notification owner, freedesktop notifications, signal shutdown |
| `diagnostics` / `privacy` | Read-only prerequisite checks, address redaction |
| `tests/bluez_bus.rs` | Explicitly isolated D-Bus service fixture |

## State and lifecycle

The backend subscribes before enumeration. Device/adapter/interface changes trigger
GetManagedObjects; no discovery scans or periodic Bluetooth polling are performed.
Watch channels keep the latest state instead of an unbounded queue. Bus/service failure
clears live devices and retries at 1, 2, 4, 8, 16, then 30-second intervals. A long stable
session resets backoff. Cancellation drops futures, streams and connection ownership.
D-Bus calls have 15-second timeouts; a timed-out connect may still complete in BlueZ,
so inspect status before retrying. The GUI keeps blocking work off the GTK thread.

The UI refreshes age labels every 30 seconds, without reading Bluetooth. The daemon
checks settings every 30 seconds, also without polling Bluetooth. On exit the GUI
runtime shuts down; daemon SIGINT/SIGTERM cancels and joins its monitor. No root GUI.

## Truthful readings

Battery1 exposes a percentage but does not specify left/right/case or charging status.
Only `aggregate` is populated; charging is unknown. Snapshot duplicates preserve the
original observation time. Percentage property signals or changed values update it.
After 300 seconds, or while disconnected, UI values are stale. BlueZ provides no
measurement timestamp: initial observation is explicitly NOT proof of a new hardware
measurement. No durable battery cache or synthetic fallback exists.

All paired devices are listed because renamed AirPods cannot reliably be identified
by alias. Names are hints only. No verified model registry is bundled, and all advanced
capabilities are false. No proprietary protocol/socket modules are added as empty
placeholders. They should be introduced only with verified packets and legal provenance.

## Privacy and platform boundaries

No network services, accounts, telemetry or uploads. Logs use fixed messages; CLI
selectors hash internal object paths. Selectors are pseudonyms, NOT cryptographic
anonymization or stable identifiers across Rust versions. Names and diagnostic host
details may still identify users; review reports before sharing. Address/path fragments
are redacted in rendered names and diagnostics. Environment desktop/session variables
are diagnostics only, never authorization inputs.

The portable core can be tested on Windows. Real BlueZ, GUI and daemon are Linux-gated.
There is no audio routing integration, tray host assumption, global shortcut, window
positioning request or always-on-top policy. The normal application window is the
explicit tray fallback on Plasma, GNOME and desktops without a tray host.
