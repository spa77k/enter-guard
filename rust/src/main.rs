#![cfg_attr(windows, windows_subsystem = "windows")]
#[cfg(windows)]
mod windows;
#[cfg(windows)]
mod preferences;
fn main() {
    #[cfg(windows)]
    windows::run();
    #[cfg(not(windows))]
    eprintln!("EnterGuard は Windows 11 x64 専用です。");
}
