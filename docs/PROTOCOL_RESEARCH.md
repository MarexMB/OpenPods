# Protocol research and licensing boundary

Research date: 2026-10-31. This release deliberately implements only public BlueZ D-Bus
interfaces. No Apple command constant, frame, model identifier or PSM is used in code.

Primary interface references:

- [BlueZ Device1](https://bluez.readthedocs.io/en/latest/device-api/): paired and connected properties; Connect and Disconnect methods.
- [BlueZ Adapter1](https://bluez.readthedocs.io/en/latest/adapter-api/): adapter availability and Powered state.
- [BlueZ Battery1](https://bluez.readthedocs.io/en/latest/battery-api/): unsigned byte percentage; no component mapping or charging field.
- [zbus](https://docs.rs/zbus/): maintained Rust D-Bus transport and object server used in fixtures.
- [libadwaita Rust bindings](https://docs.rs/libadwaita/): native GTK integration.

Public projects investigated:

- [LibrePods](https://github.com/librepods-org/librepods): GPL-3.0-or-later; describes accessory protocol support and model/firmware-dependent behavior. Its code was not copied, translated or linked.
- [LinuxPods](https://github.com/Explor3Universe/LinuxPods): documents derivation from LibrePods; not an MIT code source.
- [librepods-rs](https://github.com/brianpht/librepods-rs): further protocol research lead, not a bundled dependency or a validated compatibility reference.

PSM 0x1001 is a research lead supplied in the project brief, not a verified transport
contract for this application. Socket availability alone would not establish a usable
session. No packets are transmitted, no DeviceID spoofing is performed, and no BlueZ
configuration is changed. BLE manufacturer data is not parsed without independently
validated layout, model applicability, randomized-address handling and component mapping.

Before advanced controls can ship: obtain legally usable protocol descriptions and
fixtures with provenance; identify exact models/firmware; validate framing, bounds,
charging-state and component mapping; obtain hardware captures with explicit consent;
test acknowledgements, timeout/cancellation, competing-client exclusion and reconnection.
Keep requested state distinct from acknowledged state. Never infer success from send().

The original MIT LICENSE remains unchanged. The project icon is original geometric
artwork. System BlueZ is used via IPC, not bundled or linked into OpenPods. GTK and
libadwaita are dynamically linked system libraries under their own licenses. Cargo
dependencies retain their licenses; review the resolved dependency graph and distribute
required notices before a binary release. There is not yet a validated release lockfile.
