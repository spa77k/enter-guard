//! Process identification is refreshed by the UI timer, never inside the key hook.
//! The retained process handle prevents a recycled PID from reusing an old match.
use enter_guard::is_discord_path;
use std::ptr::null_mut;
use windows_sys::Win32::{Foundation::*, System::Threading::*, UI::WindowsAndMessaging::*};
#[derive(Default)]
pub struct Target {
    window: HWND,
    process: HANDLE,
    pid: u32,
    discord: bool,
}
impl Drop for Target {
    fn drop(&mut self) { if !self.process.is_null() { unsafe { CloseHandle(self.process); } } }
}
impl Target {
    pub fn refresh(&mut self) {
        let window = unsafe { GetForegroundWindow() };
        let mut pid = 0;
        if !window.is_null() { unsafe { GetWindowThreadProcessId(window, &mut pid); } }
        if window == self.window && pid == self.pid && !self.process.is_null() { return; }
        *self = Self::default();
        if window.is_null() || pid == 0 { return; }
        let process = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) };
        if process.is_null() { return; }
        let mut buffer = [0u16; 32768];
        let mut len = buffer.len() as u32;
        let ok = unsafe { QueryFullProcessImageNameW(process, 0, buffer.as_mut_ptr(), &mut len) };
        self.window = window;
        self.process = process;
        self.pid = pid;
        self.discord = ok != 0 && is_discord_path(&String::from_utf16_lossy(&buffer[..len as usize]));
    }
    pub fn matches(&self, window: HWND) -> bool {
        if !self.discord || window.is_null() || self.window != window || self.process.is_null() { return false; }
        let mut pid = 0;
        unsafe { GetWindowThreadProcessId(window, &mut pid); }
        // A handle is kept until refresh. PID equality plus its live process is
        // required; a foreground snapshot alone is never used for injection.
        let mut code = 0;
        pid == self.pid && unsafe { GetExitCodeProcess(self.process, &mut code) } != 0 && code == STILL_ACTIVE as u32
    }
    pub fn clear(&mut self) { *self = Self { window: null_mut(), process: null_mut(), pid: 0, discord: false }; }
}
