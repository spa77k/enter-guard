#![cfg_attr(windows, windows_subsystem = "windows")]
#[cfg(target_os = "macos")]
mod macos;
#[cfg(windows)]
mod preferences;
#[cfg(windows)]
mod windows;
fn main() {
    #[cfg(windows)]
    windows::run();
    #[cfg(target_os = "macos")]
    macos::run();
    #[cfg(not(any(windows, target_os = "macos")))]
    eprintln!("EnterGuard は Windows 11 x64・macOS 向けです。");
}
