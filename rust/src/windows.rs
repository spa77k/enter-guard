use crate::preferences;
use enter_guard::settings::Settings;
use enter_guard::*;
use std::{
    cell::RefCell,
    mem::{size_of, zeroed},
    ptr::{null, null_mut},
};
use windows_sys::Win32::{
    Foundation::*,
    Graphics::Gdi::*,
    System::{LibraryLoader::*, SystemServices::SS_LEFT, Threading::*},
    UI::{HiDpi::GetDpiForWindow, Input::KeyboardAndMouse::*, Shell::*, WindowsAndMessaging::*},
};
const TRAY: u32 = WM_APP + 1;
const FAILURE: u32 = WM_APP + 2;
const TOGGLE: usize = 101;
const HIDE: usize = 102;
const EXIT: usize = 103;
const OPEN: usize = 104;
const SETTINGS: usize = 105;
const STARTUP: usize = 106;
const KEY_BASE: usize = 200;
struct State {
    hwnd: HWND,
    status: HWND,
    button: HWND,
    help: HWND,
    hide: HWND,
    exit: HWND,
    settings_button: HWND,
    settings: Settings,
    hook: HHOOK,
    icon: HICON,
    font: HFONT,
    enabled: bool,
    hotkey: bool,
    hotkey_id: i32,
    policies: [Policy; 2],
    failure: Option<String>,
    tray_added: bool,
    taskbar_message: u32,
    smoke: bool,
}
impl Default for State {
    fn default() -> Self {
        Self {
            hwnd: null_mut(),
            status: null_mut(),
            button: null_mut(),
            help: null_mut(),
            hide: null_mut(),
            exit: null_mut(),
            settings_button: null_mut(),
            settings: Settings::default(),
            hook: null_mut(),
            icon: null_mut(),
            font: null_mut(),
            enabled: false,
            hotkey: false,
            hotkey_id: 1,
            policies: Default::default(),
            failure: None,
            tray_added: false,
            taskbar_message: 0,
            smoke: false,
        }
    }
}
thread_local! { static STATE: RefCell<State> = RefCell::new(State::default()); }
fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}
fn state<T>(f: impl FnOnce(&mut State) -> T) -> T {
    STATE.with(|s| f(&mut s.borrow_mut()))
}
fn text(hwnd: HWND, s: &str) {
    unsafe {
        SetWindowTextW(hwnd, wide(s).as_ptr());
    }
}
fn tray_data(hwnd: HWND, icon: HICON, enabled: bool) -> NOTIFYICONDATAW {
    let mut n: NOTIFYICONDATAW = unsafe { zeroed() };
    n.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
    n.hWnd = hwnd;
    n.uID = 1;
    n.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
    n.uCallbackMessage = TRAY;
    n.hIcon = icon;
    let tip = wide(if enabled {
        "EnterGuard：変換を要求中（動作保証なし）"
    } else {
        "EnterGuard：停止中"
    });
    n.szTip[..tip.len()].copy_from_slice(&tip);
    n
}
fn refresh() {
    let (hwnd, status, button, icon, enabled, hook, hotkey, failure) = state(|s| {
        (
            s.hwnd,
            s.status,
            s.button,
            s.icon,
            s.enabled,
            s.hook,
            s.hotkey,
            s.failure.clone(),
        )
    });
    let hotkey_label = format!(
        "\r\nCtrl+Alt+F{} で停止（再開は画面から）",
        state(|s| s.settings.stop_key)
    );
    let label = failure.unwrap_or_else(|| {
        if hook.is_null() {
            "キーボード監視を開始できません。置き換えは無効です。".into()
        } else {
            format!(
                "{}{}",
                if enabled {
                    "変換を要求中（動作保証なし）：Enter改行／Ctrl+Enter送信"
                } else {
                    "停止中：Enterの動作を変更していません"
                },
                if hotkey {
                    &hotkey_label
                } else {
                    "\r\nホットキーを登録できません。画面・通知領域から操作してください。"
                }
            )
        }
    });
    text(status, &label);
    text(
        button,
        if enabled {
            "一時停止する"
        } else {
            "注意事項を確認して有効にする"
        },
    );
    unsafe {
        EnableWindow(
            button,
            (!hook.is_null() && state(|s| s.failure.is_none())) as i32,
        );
        Shell_NotifyIconW(NIM_MODIFY, &tray_data(hwnd, icon, enabled));
    }
}
fn stop() {
    state(|s| s.enabled = false);
    refresh();
}
fn toggle() {
    let (enabled, hook, faulted) = state(|s| (s.enabled, s.hook, s.failure.is_some()));
    if enabled {
        stop();
        return;
    }
    if hook.is_null() || faulted {
        return;
    }
    if pressed(VK_RETURN as i32) {
        return;
    }
    // Reinstall on explicit activation. This is not a health check: Windows can
    // still silently remove the new hook later. Never claim continuous protection.
    if unsafe { UnhookWindowsHookEx(hook) } == 0 {
        fail(
            "フック解除を確認できません。再開せず、EnterGuardを終了して起動し直してください。"
                .into(),
        );
        return;
    }
    let hook =
        unsafe { SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard), GetModuleHandleW(null()), 0) };
    state(|s| s.hook = hook);
    if hook.is_null() {
        fail("フックを再登録できません。EnterGuardを終了して起動し直してください。".into());
        return;
    }
    state(|s| s.enabled = true);
    refresh();
}
fn settings_menu(hwnd: HWND) {
    // Changing preferences always pauses transformations first.
    stop();
    unsafe {
        let menu = CreatePopupMenu();
        if menu.is_null() {
            return;
        }
        let selected = state(|s| s.settings.stop_key);
        for key in 8..=11 {
            let label = format!("停止キー：Ctrl+Alt+F{key}");
            AppendMenuW(
                menu,
                MF_STRING | if key == selected { MF_CHECKED } else { 0 },
                KEY_BASE + key as usize,
                wide(&label).as_ptr(),
            );
        }
        let startup = preferences::autostart_enabled();
        AppendMenuW(menu, MF_SEPARATOR, 0, null());
        AppendMenuW(
            menu,
            MF_STRING
                | if startup.as_ref().is_ok_and(|v| *v) {
                    MF_CHECKED
                } else {
                    0
                },
            STARTUP,
            wide("Windowsログイン時に起動（必ず停止状態）").as_ptr(),
        );
        let mut point = zeroed();
        GetCursorPos(&mut point);
        let command = TrackPopupMenu(
            menu,
            TPM_RETURNCMD | TPM_RIGHTBUTTON,
            point.x,
            point.y,
            0,
            hwnd,
            null(),
        ) as usize;
        DestroyMenu(menu);
        if (KEY_BASE + 8..=KEY_BASE + 11).contains(&command) {
            let next = Settings {
                stop_key: (command - KEY_BASE) as u8,
            };
            if next.stop_key != selected || !state(|s| s.hotkey) {
                // Register the replacement first; keep the previous key on conflict.
                let old_id = state(|s| s.hotkey_id);
                let new_id = if old_id == 1 { 2 } else { 1 };
                let registered = RegisterHotKey(
                    hwnd,
                    new_id,
                    MOD_CONTROL | MOD_ALT | MOD_NOREPEAT,
                    VK_F1 as u32 + next.stop_key as u32 - 1,
                ) != 0;
                if !registered {
                    notice(
                        hwnd,
                        "その停止キーは登録できません。以前の設定を維持します。",
                    );
                } else if let Err(e) = preferences::save(next) {
                    if UnregisterHotKey(hwnd, new_id) == 0 {
                        fail(
                            "停止キーの登録取消を確認できません。終了して起動し直してください。"
                                .into(),
                        );
                    }
                    notice(
                        hwnd,
                        &format!("設定を保存できません。以前の停止キーを維持します：{e}"),
                    );
                } else {
                    let old_removed = !state(|s| s.hotkey) || UnregisterHotKey(hwnd, old_id) != 0;
                    // Alternate IDs so the old binding remains until replacement is ready.
                    state(|s| {
                        s.settings = next;
                        s.hotkey = true;
                        s.hotkey_id = new_id;
                    });
                    if !old_removed {
                        fail(
                            "以前の停止キーの解除を確認できません。終了して起動し直してください。"
                                .into(),
                        );
                    }
                }
            }
        } else if command == STARTUP {
            match startup {
                Ok(current) => {
                    if let Err(e) = preferences::set_autostart(!current) {
                        notice(hwnd, &format!("自動起動設定を変更できません：{e}"));
                    }
                }
                Err(e) => notice(
                    hwnd,
                    &format!("現在の自動起動設定を確認できないため変更しません：{e}"),
                ),
            }
        }
    }
    refresh();
}
fn notice(hwnd: HWND, message: &str) {
    unsafe {
        MessageBoxW(
            hwnd,
            wide(message).as_ptr(),
            wide("EnterGuard").as_ptr(),
            MB_OK | MB_ICONWARNING,
        );
    }
}
fn fail(reason: String) {
    let hwnd = state(|s| {
        s.enabled = false;
        s.failure = Some(reason);
        s.hwnd
    });
    unsafe {
        PostMessageW(hwnd, FAILURE, 0, 0);
    }
}
fn pressed(key: i32) -> bool {
    unsafe { GetAsyncKeyState(key) < 0 }
}
fn modifiers() -> Modifiers {
    Modifiers {
        shift: pressed(VK_SHIFT as i32),
        left: pressed(VK_LCONTROL as i32),
        right: pressed(VK_RCONTROL as i32),
        alt: pressed(VK_MENU as i32),
        windows: pressed(VK_LWIN as i32) || pressed(VK_RWIN as i32),
    }
}
fn discord(hwnd: HWND) -> bool {
    if hwnd.is_null() {
        return false;
    }
    // No PID cache: a reused PID or a failed lookup must never keep an old match.
    unsafe {
        let mut pid = 0;
        GetWindowThreadProcessId(hwnd, &mut pid);
        if pid == 0 {
            return false;
        }
        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if process.is_null() {
            return false;
        }
        let mut buffer = [0u16; 32768];
        let mut len = buffer.len() as u32;
        let ok = QueryFullProcessImageNameW(process, 0, buffer.as_mut_ptr(), &mut len);
        CloseHandle(process);
        ok != 0 && is_discord_path(&String::from_utf16_lossy(&buffer[..len as usize]))
    }
}
unsafe extern "system" fn keyboard(code: i32, w: WPARAM, l: LPARAM) -> LRESULT {
    if code >= 0 {
        let key = unsafe { &*(l as *const KBDLLHOOKSTRUCT) };
        if key.vkCode == ENTER as u32
            && key.flags & LLKHF_INJECTED == 0
            && key.dwExtraInfo != INJECTION_TAG
        {
            let down = w as u32 == WM_KEYDOWN || w as u32 == WM_SYSKEYDOWN;
            let up = w as u32 == WM_KEYUP || w as u32 == WM_SYSKEYUP;
            if down || up {
                let m = if down {
                    modifiers()
                } else {
                    Modifiers::default()
                };
                let enabled = state(|s| s.enabled);
                let foreground = if down && enabled && m.can_remap() {
                    unsafe { GetForegroundWindow() }
                } else {
                    null_mut()
                };
                let is_discord = discord(foreground);
                let keypad = key.flags & LLKHF_EXTENDED != 0;
                let action = state(|s| {
                    s.policies[keypad as usize].handle(down, enabled, is_discord, m, false)
                });
                if matches!(action, Action::Newline | Action::Send) {
                    remap(action, m, keypad, foreground);
                }
                if action != Action::Pass {
                    return 1;
                }
            }
        }
    }
    unsafe { CallNextHookEx(null_mut(), code, w, l) }
}
fn remap(action: Action, m: Modifiers, keypad: bool, foreground: HWND) {
    let Some(keys) = sequence(action, m, keypad) else {
        return;
    };
    let inputs = encode(&keys);
    // Recheck both window and modifiers after preparation. SendInput cannot make
    // this check atomic with foreground changes; that residual race is documented.
    let now = modifiers();
    if foreground.is_null()
        || unsafe { GetForegroundWindow() } != foreground
        || !discord(foreground)
        || now.shift != m.shift
        || now.left != m.left
        || now.right != m.right
        || now.alt != m.alt
        || now.windows != m.windows
    {
        fail(
            "前面ウィンドウまたは修飾キーが変わったため停止しました。入力は再送していません。"
                .into(),
        );
        return;
    }
    let inserted = unsafe {
        SendInput(
            inputs.len() as u32,
            inputs.as_ptr(),
            size_of::<INPUT>() as i32,
        )
    };
    if inserted as usize == inputs.len() {
        return;
    }
    let error = unsafe { GetLastError() };
    let repair = encode(&cleanup(&keys, inserted as usize, m));
    let restored = repair.is_empty()
        || unsafe {
            SendInput(
                repair.len() as u32,
                repair.as_ptr(),
                size_of::<INPUT>() as i32,
            )
        } as usize
            == repair.len();
    fail(format!(
        "入力の置き換えに失敗して停止（{inserted}/{}、エラー{error}）。\r\n送信済みの可能性があります。再入力前にDiscordを確認してください。{}",
        inputs.len(),
        if restored {
            ""
        } else {
            "\r\nキー状態の復元も確認できません。入力を止め、必要ならWindowsを再起動してください。"
        }
    ));
}
fn show(hwnd: HWND) {
    unsafe {
        ShowWindow(hwnd, SW_SHOW);
        SetForegroundWindow(hwnd);
    }
}
fn menu(hwnd: HWND) {
    unsafe {
        let menu = CreatePopupMenu();
        if menu.is_null() {
            return;
        }
        AppendMenuW(menu, MF_STRING, OPEN, wide("画面を開く").as_ptr());
        AppendMenuW(menu, MF_STRING, TOGGLE, wide("有効／一時停止").as_ptr());
        AppendMenuW(
            menu,
            MF_STRING,
            SETTINGS,
            wide("設定（変更時は停止）").as_ptr(),
        );
        AppendMenuW(menu, MF_STRING, EXIT, wide("終了").as_ptr());
        let mut p = zeroed();
        GetCursorPos(&mut p);
        SetForegroundWindow(hwnd);
        let command = TrackPopupMenu(
            menu,
            TPM_RETURNCMD | TPM_RIGHTBUTTON,
            p.x,
            p.y,
            0,
            hwnd,
            null(),
        ) as usize;
        DestroyMenu(menu);
        match command {
            OPEN => show(hwnd),
            TOGGLE => toggle(),
            SETTINGS => settings_menu(hwnd),
            EXIT => {
                DestroyWindow(hwnd);
            }
            _ => (),
        }
        PostMessageW(hwnd, WM_NULL, 0, 0);
    }
}
const HELP: &str = "Windows版 Discord（通常版）専用です。\r\n\r\n有効中、Discordが最前面のとき、Enterを改行（Shift+Enter）、Ctrl+Enterを送信（Enter）に置き換えます。長押しは1回だけ処理します。\r\n\r\n入力欄を識別しないため、検索・ダイアログ・通話画面にも作用します。ログイン・パスワード入力前は停止してください。Shift / Alt / Windowsキー付きの組み合わせは変更しません。\r\n\r\n日本語IMEの変換確定にも作用します。Windows実機・IME・Discord画面で未検証の試用版です。誤送信を防ぐ保証はありません。変換確定が必要なときは一時停止してください。\r\n\r\n停止キーはCtrl+Alt+F10（設定で変更可能）。停止キーで再開はしません。設定から任意の自動起動も選べますが、起動直後は必ず停止中です。異常で停止した後は終了して起動し直してください。\r\n\r\n大切な会話の前に、自分だけのテスト用チャンネルで確認してください。";
fn layout(hwnd: HWND) {
    unsafe {
        let dpi = GetDpiForWindow(hwnd).max(96);
        let px = |n: i32| n * dpi as i32 / 96;
        let mut r = zeroed();
        GetClientRect(hwnd, &mut r);
        let width = (r.right - px(40)).max(px(200));
        let bottom = r.bottom;
        let help_height = (bottom - px(330)).max(px(160));
        let (status, help, button, hide, exit, old_font) =
            state(|s| (s.status, s.help, s.button, s.hide, s.exit, s.font));
        let font = CreateFontW(
            -px(16),
            0,
            0,
            0,
            FW_NORMAL as i32,
            0,
            0,
            0,
            DEFAULT_CHARSET as u32,
            0,
            0,
            0,
            0,
            wide("Yu Gothic UI").as_ptr(),
        );
        for control in [
            status,
            help,
            button,
            hide,
            exit,
            state(|s| s.settings_button),
        ] {
            SendMessageW(control, WM_SETFONT, font as usize, 1);
        }
        state(|s| s.font = font);
        if !old_font.is_null() {
            DeleteObject(old_font);
        }
        MoveWindow(status, px(20), px(16), width, px(100), 1);
        MoveWindow(help, px(20), px(120), width, help_height, 1);
        for (i, control) in [button, state(|s| s.settings_button), hide, exit]
            .iter()
            .enumerate()
        {
            MoveWindow(
                *control,
                px(20),
                bottom - px(190 - i as i32 * 45),
                width,
                px(36),
                1,
            );
        }
    }
}
unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    if msg != 0 && msg == state(|s| s.taskbar_message) {
        let (icon, enabled) = state(|s| (s.icon, s.enabled));
        let added = unsafe { Shell_NotifyIconW(NIM_ADD, &tray_data(hwnd, icon, enabled)) } != 0;
        state(|s| s.tray_added = added);
        if !added {
            stop();
            show(hwnd);
        }
        unsafe {
            EnableWindow(state(|s| s.hide), added as i32);
        }
        return 0;
    }
    match msg {
        WM_TIMER if w == 1 && state(|s| s.smoke) => {
            unsafe {
                KillTimer(hwnd, 1);
            }
            let ready = state(|s| {
                !s.enabled
                    && !s.hook.is_null()
                    && !s.icon.is_null()
                    && s.tray_added
                    && !s.font.is_null()
                    && [
                        s.status,
                        s.help,
                        s.button,
                        s.hide,
                        s.exit,
                        s.settings_button,
                    ]
                    .iter()
                    .all(|h| !h.is_null())
            });
            if !ready {
                unsafe {
                    DestroyWindow(hwnd);
                }
                std::process::exit(2);
            }
            toggle();
            let enabled = state(|s| s.enabled);
            stop();
            stop();
            let paused = state(|s| !s.enabled);
            fail("Smoke test: intentional fault".into());
            toggle();
            let fault_latched = state(|s| !s.enabled && s.failure.is_some());
            unsafe {
                DestroyWindow(hwnd);
            }
            if !enabled || !paused || !fault_latched {
                std::process::exit(3);
            }
            0
        }
        WM_COMMAND => {
            match w & 0xffff {
                TOGGLE => toggle(),
                SETTINGS => settings_menu(hwnd),
                HIDE => unsafe {
                    ShowWindow(hwnd, SW_HIDE);
                },
                EXIT => unsafe {
                    DestroyWindow(hwnd);
                },
                _ => (),
            };
            0
        }
        WM_HOTKEY => {
            if w == 1 || w == 2 {
                stop();
            }
            0
        }
        TRAY => {
            match l as u32 {
                WM_LBUTTONDBLCLK => show(hwnd),
                WM_RBUTTONUP | WM_CONTEXTMENU => menu(hwnd),
                _ => (),
            };
            0
        }
        FAILURE => {
            refresh();
            show(hwnd);
            0
        }
        WM_SIZE => {
            if !state(|s| s.status.is_null()) {
                layout(hwnd);
            }
            0
        }
        WM_GETMINMAXINFO => {
            let dpi = unsafe { GetDpiForWindow(hwnd) }.max(96) as i32;
            let info = unsafe { &mut *(l as *mut MINMAXINFO) };
            info.ptMinTrackSize.x = 620 * dpi / 96;
            info.ptMinTrackSize.y = 680 * dpi / 96;
            0
        }
        WM_DPICHANGED => {
            let r = unsafe { &*(l as *const RECT) };
            unsafe {
                SetWindowPos(
                    hwnd,
                    null_mut(),
                    r.left,
                    r.top,
                    r.right - r.left,
                    r.bottom - r.top,
                    SWP_NOZORDER | SWP_NOACTIVATE,
                );
            }
            layout(hwnd);
            0
        }
        WM_CLOSE => {
            unsafe {
                DestroyWindow(hwnd);
            }
            0
        }
        WM_DESTROY => {
            let (hook, hotkey, icon, font, added) = state(|s| {
                s.enabled = false;
                let hook = s.hook;
                s.hook = null_mut();
                (hook, s.hotkey, s.icon, s.font, s.tray_added)
            });
            unsafe {
                if !hook.is_null() {
                    UnhookWindowsHookEx(hook);
                }
                if hotkey {
                    UnregisterHotKey(hwnd, 1);
                    UnregisterHotKey(hwnd, 2);
                }
                if added {
                    Shell_NotifyIconW(NIM_DELETE, &tray_data(hwnd, icon, false));
                }
                if !font.is_null() {
                    DeleteObject(font);
                }
                PostQuitMessage(0);
            }
            0
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, w, l) },
    }
}
pub fn run() {
    let smoke = std::env::args().any(|arg| arg == "--smoke-test");
    unsafe {
        let mutex = CreateMutexW(null(), 0, wide("Local\\EnterGuard").as_ptr());
        if mutex.is_null() {
            if smoke {
                std::process::exit(4);
            }
            MessageBoxW(
                null_mut(),
                wide("起動ロックを作成できません。").as_ptr(),
                wide("EnterGuard").as_ptr(),
                MB_ICONERROR,
            );
            return;
        }
        if GetLastError() == ERROR_ALREADY_EXISTS {
            CloseHandle(mutex);
            if smoke {
                std::process::exit(5);
            }
            MessageBoxW(
                null_mut(),
                wide("すでに起動しています。通知領域を確認してください。").as_ptr(),
                wide("EnterGuard").as_ptr(),
                MB_OK,
            );
            return;
        }
        let instance = GetModuleHandleW(null());
        let icon = LoadIconW(instance, std::ptr::without_provenance(1));
        let class_name = wide("EnterGuardRust");
        let class = WNDCLASSW {
            lpfnWndProc: Some(wndproc),
            hInstance: instance,
            hIcon: icon,
            hCursor: LoadCursorW(null_mut(), IDC_ARROW),
            hbrBackground: (COLOR_WINDOW + 1) as HBRUSH,
            lpszClassName: class_name.as_ptr(),
            ..zeroed()
        };
        if RegisterClassW(&class) == 0 {
            CloseHandle(mutex);
            std::process::exit(6);
        }
        let hwnd = CreateWindowExW(
            0,
            class_name.as_ptr(),
            wide("EnterGuard — Rust 試用版 0.2.1").as_ptr(),
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            720,
            800,
            null_mut(),
            null_mut(),
            instance,
            null(),
        );
        if hwnd.is_null() {
            CloseHandle(mutex);
            std::process::exit(7);
        }
        let create = |class: &str, label: &str, style: u32, id: usize| {
            CreateWindowExW(
                0,
                wide(class).as_ptr(),
                wide(label).as_ptr(),
                WS_CHILD | WS_VISIBLE | style,
                0,
                0,
                1,
                1,
                hwnd,
                id as HMENU,
                instance,
                null(),
            )
        };
        let status = create("STATIC", "", SS_LEFT, 0);
        // Read-only scrollable help remains accessible when text wraps at high DPI.
        let help = create(
            "EDIT",
            HELP,
            ES_MULTILINE as u32
                | ES_READONLY as u32
                | ES_AUTOVSCROLL as u32
                | WS_VSCROLL
                | WS_TABSTOP,
            0,
        );
        let button = create("BUTTON", "", WS_TABSTOP | BS_PUSHBUTTON as u32, TOGGLE);
        let settings_button = create(
            "BUTTON",
            "設定",
            WS_TABSTOP | BS_PUSHBUTTON as u32,
            SETTINGS,
        );
        let hide = create(
            "BUTTON",
            "通知領域にしまう",
            WS_TABSTOP | BS_PUSHBUTTON as u32,
            HIDE,
        );
        let exit = create(
            "BUTTON",
            "終了する",
            WS_TABSTOP | BS_PUSHBUTTON as u32,
            EXIT,
        );
        state(|s| {
            s.hwnd = hwnd;
            s.status = status;
            s.help = help;
            s.button = button;
            s.hide = hide;
            s.exit = exit;
            s.settings_button = settings_button;
            s.icon = icon;
            s.smoke = smoke;
            s.taskbar_message = RegisterWindowMessageW(wide("TaskbarCreated").as_ptr());
        });
        let tray_added = Shell_NotifyIconW(NIM_ADD, &tray_data(hwnd, icon, false)) != 0;
        // A missing tray must not strand a hidden window.
        if !tray_added {
            EnableWindow(hide, 0);
        }
        if !smoke {
            match preferences::load() {
                Ok(settings) => state(|s| s.settings = settings),
                Err(e) => notice(
                    hwnd,
                    &format!("設定を読み込めません。停止状態・初期停止キーで起動します：{e}"),
                ),
            }
        }
        let stop_key = state(|s| s.settings.stop_key);
        let hotkey = RegisterHotKey(
            hwnd,
            1,
            MOD_CONTROL | MOD_ALT | MOD_NOREPEAT,
            VK_F1 as u32 + stop_key as u32 - 1,
        ) != 0;
        let hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard), instance, 0);
        state(|s| {
            s.hotkey = hotkey;
            s.hook = hook;
            s.tray_added = tray_added;
        });
        layout(hwnd);
        refresh();
        ShowWindow(hwnd, SW_SHOW);
        if smoke && SetTimer(hwnd, 1, 100, None) == 0 {
            DestroyWindow(hwnd);
            std::process::exit(8);
        }
        let mut message = zeroed();
        loop {
            let result = GetMessageW(&mut message, null_mut(), 0, 0);
            if result <= 0 {
                if result < 0 {
                    DestroyWindow(hwnd);
                    if smoke {
                        std::process::exit(9);
                    }
                }
                break;
            }
            if IsDialogMessageW(hwnd, &message) == 0 {
                TranslateMessage(&message);
                DispatchMessageW(&message);
            }
        }
        CloseHandle(mutex);
    }
}
