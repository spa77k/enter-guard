//! Only application preferences are persisted. Never key events or input contents.
use std::{fs, io::{self, Read}, path::Path};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Settings {
    pub stop_key: u8,
}
impl Default for Settings {
    fn default() -> Self { Self { stop_key: 10 } }
}
impl Settings {
    pub fn parse(s: &str) -> Option<Self> {
        let mut lines = s.lines();
        if lines.next()? != "version=1" { return None; }
        let key = lines.next()?.strip_prefix("stop_key=F")?.parse::<u8>().ok()?;
        if !(8..=11).contains(&key) || lines.next().is_some() { return None; }
        Some(Self { stop_key: key })
    }
    pub fn serialize(self) -> String { format!("version=1\nstop_key=F{}\n", self.stop_key) }
    pub fn load(path: &Path) -> io::Result<Self> {
        let file = match fs::File::open(path) {
            Ok(file) => file,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(e) => return Err(e),
        };
        let mut contents = String::new();
        file.take(129).read_to_string(&mut contents)?;
        if contents.len() > 128 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Settings too large"));
        }
        Self::parse(&contents).ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Invalid settings"))
    }
    pub fn save(self, path: &Path) -> io::Result<()> {
        if let Some(parent) = path.parent() { fs::create_dir_all(parent)?; }
        let temporary = path.with_extension("tmp");
        fs::write(&temporary, self.serialize())?;
        if let Err(error) = fs::rename(&temporary, path) {
            let _ = fs::remove_file(temporary);
            return Err(error);
        }
        Ok(())
    }

}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn settings_roundtrip() {
        for key in 8..=11 { let s = Settings { stop_key: key }; assert_eq!(Settings::parse(&s.serialize()), Some(s)); }
    }
    #[test] fn reject_malformed_and_unknown_settings() {
        for s in ["", "version=2\nstop_key=F10", "version=1\nstop_key=F7", "version=1\nstop_key=F12", "version=1\nstop_key=F13", "version=1\nstop_key=F10\nenabled=true", "version=1\nstop_key=F10\n\n"] { assert_eq!(Settings::parse(s), None); }
    }
    #[test] fn file_roundtrip_and_corruption() {
        let dir = std::env::temp_dir().join(format!("enterguard-settings-test-{}", std::process::id()));
        let path = dir.join("settings.txt");
        assert_eq!(Settings::load(&path).unwrap(), Settings::default());
        Settings { stop_key: 8 }.save(&path).unwrap();
        assert_eq!(Settings::load(&path).unwrap().stop_key, 8);
        Settings { stop_key: 11 }.save(&path).unwrap();
        assert_eq!(Settings::load(&path).unwrap().stop_key, 11);
        fs::write(&path, "x".repeat(1024)).unwrap();
        assert!(Settings::load(&path).is_err());
        fs::remove_dir_all(dir).unwrap();
    }
    #[test] fn defaults_never_contain_enabled_or_autostart() {
        assert_eq!(Settings::default().serialize(), "version=1\nstop_key=F10\n");
    }
}
