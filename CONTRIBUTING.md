# Contributing

Read [AGENTS.md](AGENTS.md), [architecture](docs/ARCHITECTURE.md) and
[protocol provenance](docs/PROTOCOL_RESEARCH.md). Original contributions are MIT.
Do not copy incompatible copyleft code into this repository.

Use stable Rust. Keep UI out of core state code and hardware out of unit tests. Run:

```sh
cargo fmt
cargo fmt --check
cargo test --no-default-features
cargo test --all-features
dbus-run-session -- cargo test --test bluez_bus -- --ignored
cargo clippy --all-targets --all-features -- -D warnings
cargo build --release
```

The private-bus test uses synthetic fixtures labelled as such. Never attach raw addresses,
pairing keys or unreviewed protocol dumps to bug reports. Use `openpods diagnose` and
review names and host details before sharing. Identify exact tested model/firmware when
reporting hardware results. Passing mock tests is not physical-device verification.

Current build validation is blocked on the development machine; see docs/VALIDATION.md.
Before release, generate and commit Cargo.lock on a supported builder and use `--locked`
for all subsequent builds. CI is provided but has not been pushed or executed by this task.
