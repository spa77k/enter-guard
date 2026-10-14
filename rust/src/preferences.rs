use enter_guard::settings::Settings;
use std::{io, path::PathBuf, ptr::null};
use windows_sys::Win32::{Foundation::*, System::Registry::*};
fn wide(s: &str) -> Vec<u16> { s.encode_utf16().chain(Some(0)).collect() }
fn path() -> io::Result<PathBuf> {
    std::env::var_os("LOCALAPPDATA").map(|p| PathBuf::from(p).join("EnterGuard/settings.txt"))
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "LOCALAPPDATA unavailable"))
}
pub fn load() -> io::Result<Settings> { Settings::load(&path()?) }
pub fn save(settings: Settings) -> io::Result<()> { settings.save(&path()?) }
const RUN: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
const VALUE: &str = "EnterGuard";
fn command() -> io::Result<String> {
    let exe = std::env::current_exe()?;
    let exe = exe.to_str().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Invalid executable path"))?;
    if exe.contains('"') { return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid executable path")); }
    Ok(format!("\"{exe}\" --autostart"))
}
pub fn autostart_enabled() -> io::Result<bool> {
    let mut bytes = 0;
    let result = unsafe { RegGetValueW(HKEY_CURRENT_USER, wide(RUN).as_ptr(), wide(VALUE).as_ptr(), RRF_RT_REG_SZ, std::ptr::null_mut(), std::ptr::null_mut(), &mut bytes) };
    if result == ERROR_FILE_NOT_FOUND { return Ok(false); }
    if result != ERROR_SUCCESS { return Err(io::Error::from_raw_os_error(result as i32)); }
    if bytes > 65536 || bytes % 2 != 0 { return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid startup value")); }
    let mut buffer = vec![0u16; bytes as usize / 2];
    let result = unsafe { RegGetValueW(HKEY_CURRENT_USER, wide(RUN).as_ptr(), wide(VALUE).as_ptr(), RRF_RT_REG_SZ, std::ptr::null_mut(), buffer.as_mut_ptr().cast(), &mut bytes) };
    if result != ERROR_SUCCESS { return Err(io::Error::from_raw_os_error(result as i32)); }
    Ok(buffer == wide(&command()?))
}
pub fn set_autostart(enabled: bool) -> io::Result<()> {
    let mut key = std::ptr::null_mut();
    let result = unsafe { RegCreateKeyExW(HKEY_CURRENT_USER, wide(RUN).as_ptr(), 0, null(), REG_OPTION_NON_VOLATILE, KEY_SET_VALUE, null(), &mut key, std::ptr::null_mut()) };
    if result != ERROR_SUCCESS { return Err(io::Error::from_raw_os_error(result as i32)); }
    let result = if enabled {
        let command = match command() { Ok(c) => wide(&c), Err(e) => { unsafe { RegCloseKey(key); } return Err(e); } };
        unsafe { RegSetValueExW(key, wide(VALUE).as_ptr(), 0, REG_SZ, command.as_ptr().cast(), (command.len() * 2) as u32) }
    } else { unsafe { RegDeleteValueW(key, wide(VALUE).as_ptr()) } };
    unsafe { RegCloseKey(key); }
    if result == ERROR_SUCCESS || (!enabled && result == ERROR_FILE_NOT_FOUND) { Ok(()) }
    else { Err(io::Error::from_raw_os_error(result as i32)) }
}
