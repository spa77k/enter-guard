//! Only application preferences are persisted. Never key events or input contents.
use std::{fs, io, path::Path};
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
        if !(8..=12).contains(&key) || lines.next().is_some() { return None; }
        Some(Self { stop_key: key })
    }
    pub fn serialize(self) -> String { format!("version=1\nstop_key=F{}\n", self.stop_key) }
    pub fn load(path: &Path) -> io::Result<Self> {
        match fs::read_to_string(path) {
            Ok(s) => Self::parse(&s).ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Invalid settings")),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(e),
        }
    }
    pub fn save(self, path: &Path) -> io::Result<()> {
        if let Some(parent) = path.parent() { fs::create_dir_all(parent)?; }
        // Tiny preferences only. A truncated file is rejected on the next start.
        fs::write(path, self.serialize())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn settings_roundtrip() {
        for key in 8..=12 { let s = Settings { stop_key: key }; assert_eq!(Settings::parse(&s.serialize()), Some(s)); }
    }
    #[test] fn reject_malformed_and_unknown_settings() {
        for s in ["", "version=2\nstop_key=F10", "version=1\nstop_key=F7", "version=1\nstop_key=F13", "version=1\nstop_key=F10\nenabled=true", "version=1\nstop_key=F10\n\n"] { assert_eq!(Settings::parse(s), None); }
    }
    #[test] fn defaults_never_contain_enabled_or_autostart() {
        assert_eq!(Settings::default().serialize(), "version=1\nstop_key=F10\n");
    }
}
