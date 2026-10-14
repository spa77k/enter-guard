#![cfg_attr(windows, windows_subsystem = "windows")]
#[cfg(windows)]
mod preferences;
#[cfg(windows)]
mod windows;
fn main() {
    #[cfg(windows)]
    windows::run();
    #[cfg(not(windows))]
    eprintln!("EnterGuard は Windows 11 x64 専用です。");
}
