# Compatibility and evidence

**No physical AirPods model or CachyOS desktop has been verified for this release.**
Implemented means code exists, not that hardware testing passed. Automated tests have
been authored but require the toolchain described in [VALIDATION.md](VALIDATION.md).

| Device family | Verified identifier | Paired state / Battery1 | Left/right/case | ANC / transparency / adaptive | In-ear / MPRIS | Hardware verified |
| --- | --- | --- | --- | --- | --- | --- |
| AirPods 1 / 2 / 3 | None | Generic BlueZ path implemented, availability depends on BlueZ/device | Unavailable | Unavailable | Unavailable | No |
| AirPods 4 | None | Same | Unavailable | Unavailable | Unavailable | No |
| AirPods 4 with ANC | None | Same | Unavailable | Unavailable | Unavailable | No |
| AirPods Pro / Pro 2 / Pro 3 | None | Same | Unavailable | Unavailable | Unavailable | No |
| AirPods Max | None | Same | Unavailable | Unavailable | Unavailable | No |
| Unknown / renamed / future models | None | All paired devices listed | Unavailable | Unavailable | Unavailable | No |

No statement above guarantees that any AirPods firmware exposes Battery1. An absent
interface is shown as unavailable. Noise modes are not offered merely because a name
contains “Pro” or “Max”. There are no experimental packets enabled behind the UI.

| Feature | Implementation | Automated coverage authored | Native/hardware validation |
| --- | --- | --- | --- |
| Paired enumeration and connection state | Implemented | Object fixtures; private D-Bus integration | Pending |
| Connect/disconnect | Implemented | Private D-Bus integration | Pending |
| Aggregate Battery1 | Implemented | Bounds, unknown/removal, duplicates, freshness | Pending |
| Settings / notification policy | Implemented | Persistence, invalid input, cooldown/transition rules | Pending |
| GTK4/libadwaita UI | Implemented | State formatting only; no UI automation yet | Pending |
| User daemon / notifications | Implemented | Cancellation; policy logic | Pending |
| Plasma tray / StatusNotifierItem | Planned; window fallback exists | No | Pending |
| Apple accessory protocol, BLE battery | Research only; no commands or parser | No protocol fixtures | Pending |
| Noise, in-ear, MPRIS, PipeWire output | Planned / unavailable | No | Pending |
| PKGBUILD, launcher, user service | Authored | CI desktop-file check configured | Pending |
