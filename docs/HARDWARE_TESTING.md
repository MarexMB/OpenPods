# Physical-device verification record

**No rows have been verified. Do not mark a test as passed without performing it.**
Synthetic D-Bus fixtures do not count as physical AirPods testing.

Record locally (omit Bluetooth addresses from shared reports):

```text
Date / tester:
CachyOS /etc/os-release:
Desktop version / Wayland or X11:
Kernel / BlueZ / PipeWire / WirePlumber versions:
OpenPods git revision / Rust version:
AirPods exact model / firmware / evidence for identity:
Adapter chipset (no MAC):
Package or source build:
```

| Check | Procedure and evidence to record | Result |
| --- | --- | --- |
| Native build | Run every command in CACHYOS.md, save errors without device identifiers | Pending |
| GUI | Launch from KDE menu as non-root, light/dark, keyboard navigation, resize | Pending |
| Single instance | Launch twice; one window should be presented | Pending |
| Paired enumeration | Pair in KDE first; verify renamed and multiple devices appear | Pending |
| Connection state | Connect/disconnect in both KDE and OpenPods; compare BlueZ state | Pending |
| Genuine battery | Confirm Battery1 exists and compare percentage; note its source/cache limits | Pending |
| Missing components | Verify left/right/case and charging stay unavailable, never aggregate copies | Pending |
| Freshness | Wait beyond 300s without battery events; disconnect; inspect stale indication | Pending |
| Adapter states | Test powered off, absent adapter, stopped BlueZ, permission denial separately | Pending |
| rfkill | Inspect real soft/hard block diagnostics; do not automatically change system state | Pending |
| Recovery | Suspend/resume, BlueZ restart, adapter reconnection and unpairing | Pending |
| Notifications | Default off; opt in, verify host delivery, transition and cooldown behavior | Pending |
| Notification owner | Start daemon twice; second must fail, GUI must not duplicate notifications | Pending |
| Service lifecycle | Enable/disable as user; SIGTERM/GUI close; inspect processes and bus names | Pending |
| Tray fallback | No tray icon expected; menu window remains usable on Plasma and GNOME | Pending |
| Audio | Ensure connecting does not add OpenPods-driven profile/codec/routing changes | Pending |
| Packaging | makepkg -si, installed launcher/icon/service; remove with pacman -R | Pending |
| Idle load | Measure CPU, memory and D-Bus traffic at rest and under device events | Pending |
| Privacy | Review diagnostics/logs for addresses, secrets and unexpected network activity | Pending |

Compare only sources actually available. If BlueZ does not provide Battery1, record
“unavailable” rather than a failure of invented component/protocol support. Use actual
battery discharge for hardware notification tests; fabricated reports belong only in
the explicitly isolated D-Bus test process.

Advanced protocol tests cannot yet be performed because that implementation is absent.
Future acceptance must include captures with provenance, exact model/firmware, malformed
frames, timeouts, confirmed state, competing clients and clean socket teardown. Never
mark ANC, transparency, adaptive audio or in-ear playback verified from a socket opening.
