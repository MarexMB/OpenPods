//! Hardware-independent state and Linux integration for OpenPods.
pub const APP_ID: &str = "io.github.MarexMB.OpenPods";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub mod battery;
pub mod model;
pub mod settings;
pub mod notifications;
pub mod privacy;
pub mod retry;
#[cfg(target_os = "linux")]
pub mod daemon;
#[cfg(target_os = "linux")]
pub mod diagnostics;
#[cfg(all(target_os = "linux", feature = "gui"))]
pub mod ui;
#[cfg(target_os = "linux")]
pub mod bluetooth;

pub fn now() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs()
}
