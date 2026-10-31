# Validation record

Development session: 2026-10-31 (local clock may roll into 2026-11-01).
Host: Windows PowerShell, repository `C:\Users\vikto\OneDrive\Desktop\vss\OpenPods`.
Initial HEAD: `6fec7e5` (README, Python-oriented .gitignore, MIT LICENSE only).
No initial application source. Original LICENSE remains byte-for-byte unchanged.

## Results actually obtained

| Exact command / check | Outcome |
| --- | --- |
| `git status --short` | Initial checkout clean; clean after implementation commits |
| `git log -5 --oneline` / `git remote -v` | Single initial commit; MarexMB/OpenPods origin confirmed |
| `Get-Command cargo,rustc,git,python,wsl -ErrorAction SilentlyContinue` | Git and wsl launcher found; Cargo/Rust/Python not on PATH |
| `if (Test-Path /etc/os-release) { Get-Content /etc/os-release }` | File absent on this Windows host; no CachyOS claim |
| `wsl --list --quiet` | WSL not installed; no Linux distribution available |
| `Test-Path "$env:USERPROFILE/.cargo/bin/cargo.exe"` | False |
| `vswhere.exe -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath` | No C++ toolchain installation returned |
| `cargo fmt --check` | Could not execute: cargo command not found |
| `cargo test --no-default-features` | Could not execute: cargo command not found |
| `cargo clippy --all-targets --all-features -- -D warnings` | Could not execute: cargo command not found |
| `cargo build --release` | Could not execute: cargo command not found |
| `& 'C:/Program Files/Git/bin/bash.exe' -n scripts/package.sh` | Passed shell syntax check |
| `& 'C:/Program Files/Git/bin/bash.exe' -n PKGBUILD` | Passed shell syntax check |
| `& 'C:/Program Files/Git/bin/bash.exe' scripts/package.sh` | Passed: created reviewed-HEAD source archive and checksummed PKGBUILD |
| `Get-FileHash -Algorithm SHA256 -LiteralPath .package/openpods-0.1.0.tar` compared with generated PKGBUILD | Matched |
| `& 'C:/Program Files/Git/bin/bash.exe' -n .package/PKGBUILD` | Passed generated package shell syntax check |
| `tar -tf .package/openpods-0.1.0.tar` | Archive readable, expected project files present |
| `[xml]$icon = Get-Content -Raw data/io.github.MarexMB.OpenPods.svg` | SVG XML parsed successfully |
| `git diff --check` | Passed whitespace-error check (not a Rust formatting check) |
| `git diff 6fec7e5 -- LICENSE` | Empty; original license preserved |

One attempted nested Bash/PowerShell checksum verification command failed due to quoting.
It was replaced with the successful PowerShell Get-FileHash comparison above; the package
generator itself had succeeded. Initial git staging was denied by the sandbox; the
authorized commit workflow subsequently used approved git access. No commits were pushed.

## Tests authored, not reported as passing

- Portable: percentage bounds, zero/unknown, missing components, charging unknown,
  duplicate observations, stale/future timestamps, state formatting, adapter transitions,
  name/capability gating, settings persistence/invalid input, notification defaults,
  transition deduplication, cooldown/hysteresis, identifier redaction and retry cap.
- Linux: real D-Bus value shapes, device disconnect/removal, malformed battery,
  monitor cancellation, operation timeout/failure mapping.
- Private D-Bus integration: ObjectManager discovery, Battery1, connect/disconnect
  signal propagation, interface removal, BlueZ owner loss/recovery, orderly shutdown.
- CI workflow authored for portable Windows and native Linux checks; **not run**.
  Ubuntu CI is a build check, not CachyOS acceptance.

No Rust tests passed in this session because none could execute. No GTK window was
rendered or visually verified. No physical AirPods model, notification host, Plasma
tray fallback, systemd service or CachyOS package install/removal was tested.

## Release blockers and next actions

1. Obtain an approved Rust toolchain/build environment. Permission to download a local
   toolchain was requested during implementation; no installation was performed.
2. Run `cargo fmt` then the required formatting, tests, lint and release build; fix all
   compile/lint defects. Source review is not proof of buildability.
3. Resolve dependencies and commit Cargo.lock; add --locked to builds and packaging.
4. On real CachyOS, run the native GUI/private-bus checks, makepkg install/removal and
   [hardware checklist](HARDWARE_TESTING.md). Record exact hardware and firmware.
5. Implement/test StatusNotifierItem if required, with the existing explicit fallback.
6. Only then expand protocol controls using verified, license-compatible evidence.

The requested definition of done is **not met yet**. The repository contains a substantive
native implementation and tests, but a successful native build and functional GUI run
remain unverified. No successful hardware or protocol results are implied by commits.
