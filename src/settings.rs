use serde::{Deserialize, Serialize};
use std::{io, path::{Path, PathBuf}};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Theme { #[default] System, Light, Dark }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Settings {
    pub low_battery_notifications: bool,
    pub connection_notifications: bool,
    pub low_battery_threshold: u8,
    pub theme: Theme,
}

impl Default for Settings {
    fn default() -> Self {
        Self { low_battery_notifications: false, connection_notifications: false, low_battery_threshold: 20, theme: Theme::System }
    }
}

impl Settings {
    pub fn path() -> io::Result<PathBuf> {
        let base = std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from).filter(|p| p.is_absolute())
            .or_else(|| std::env::var_os("HOME").map(|p| PathBuf::from(p).join(".config")))
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "No XDG config or home directory"))?;
        Ok(base.join("openpods/settings.json"))
    }

    pub fn load(path: &Path) -> io::Result<Self> {
        use std::io::Read;
        let file = match std::fs::File::open(path) {
            Ok(file) => file,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(e) => return Err(e),
        };
        let mut bytes = Vec::new();
        file.take(16_385).read_to_end(&mut bytes)?;
        if bytes.len() > 16_384 { return Err(io::Error::new(io::ErrorKind::InvalidData, "Settings too large")); }
        let settings: Self = serde_json::from_slice(&bytes).map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid settings JSON"))?;
        settings.validate()?;
        Ok(settings)
    }

    pub fn validate(&self) -> io::Result<()> {
        if !(1..=100).contains(&self.low_battery_threshold) {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Threshold must be 1..100"));
        }
        Ok(())
    }

    pub fn save(&self, path: &Path) -> io::Result<()> {
        use std::io::Write;
        self.validate()?;
        let parent = path.parent().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Missing settings parent"))?;
        std::fs::create_dir_all(parent)?;
        let temporary = parent.join(format!("settings-{}.tmp", std::process::id()));
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)] {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options.open(&temporary)?;
        let result = (|| {
            file.write_all(&serde_json::to_vec_pretty(self)?)?;
            file.sync_all()?;
            drop(file);
            std::fs::rename(&temporary, path)
        })();
        if result.is_err() { let _ = std::fs::remove_file(&temporary); }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn defaults_validation_and_roundtrip() {
        let root = std::env::temp_dir().join(format!("openpods-settings-{}-{}", std::process::id(), crate::now()));
        let path = root.join("settings.json");
        assert_eq!(Settings::load(&path).unwrap(), Settings::default());
        let settings = Settings { low_battery_notifications: true, theme: Theme::Dark, ..Settings::default() };
        settings.save(&path).unwrap();
        assert_eq!(Settings::load(&path).unwrap(), settings);
        std::fs::write(&path, b"{\"low_battery_threshold\":255}").unwrap();
        assert!(Settings::load(&path).is_err());
        std::fs::write(&path, b"not json").unwrap();
        assert!(Settings::load(&path).is_err());
        std::fs::remove_dir_all(root).unwrap();
    }
}
