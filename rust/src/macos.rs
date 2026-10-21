//! Native AppKit UI and a session event tap. All callbacks run on the main run loop.
//! The FFI boundary is deliberately small; no text/AX element inspection or injection.
#![allow(unsafe_op_in_unsafe_fn)]
use enter_guard::mac_policy::{COMMAND, Decision, OPTION, Policy};
use std::{
    cell::RefCell,
    ffi::{CStr, c_char, c_void},
    ptr,
};
type Id = *mut c_void;
type Sel = *mut c_void;
#[repr(C)]
#[derive(Clone, Copy)]
struct Point {
    x: f64,
    y: f64,
}
#[repr(C)]
#[derive(Clone, Copy)]
struct Size {
    width: f64,
    height: f64,
}
#[repr(C)]
#[derive(Clone, Copy)]
struct Rect {
    origin: Point,
    size: Size,
}
fn rect(x: f64, y: f64, width: f64, height: f64) -> Rect {
    Rect {
        origin: Point { x, y },
        size: Size { width, height },
    }
}
#[link(name = "objc")]
unsafe extern "C" {
    fn objc_getClass(name: *const c_char) -> Id;
    fn sel_registerName(name: *const c_char) -> Sel;
    fn objc_msgSend();
    fn objc_allocateClassPair(superclass: Id, name: *const c_char, extra: usize) -> Id;
    fn objc_registerClassPair(class: Id);
    fn class_addMethod(
        class: Id,
        selector: Sel,
        imp: unsafe extern "C" fn(Id, Sel, Id),
        types: *const c_char,
    ) -> i8;
}
#[link(name = "AppKit", kind = "framework")]
unsafe extern "C" {}
#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
    fn AXIsProcessTrusted() -> bool;
    fn CGEventTapCreate(
        location: u32,
        place: u32,
        options: u32,
        mask: u64,
        callback: unsafe extern "C" fn(Id, u32, Id, Id) -> Id,
        info: Id,
    ) -> Id;
    fn CGEventTapEnable(tap: Id, enabled: bool);
    fn CGEventTapIsEnabled(tap: Id) -> bool;
    fn CGEventGetFlags(event: Id) -> u64;
    fn CGEventSetFlags(event: Id, flags: u64);
    fn CGEventGetIntegerValueField(event: Id, field: u32) -> i64;
}
#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    static kCFRunLoopCommonModes: Id;
    fn CFRunLoopGetMain() -> Id;
    fn CFMachPortCreateRunLoopSource(allocator: Id, port: Id, order: isize) -> Id;
    fn CFRunLoopAddSource(run_loop: Id, source: Id, mode: Id);
    fn CFRelease(object: Id);
}
// Each invocation specifies the actual Objective-C method ABI, including CGRect.
macro_rules! msg {
    ($obj:expr, $sel:expr, $ret:ty $(; $arg:expr => $ty:ty)*) => {{
        let f: unsafe extern "C" fn(Id, Sel $(, $ty)*) -> $ret =
            std::mem::transmute(objc_msgSend as unsafe extern "C" fn());
        f($obj, sel_registerName($sel.as_ptr()) $(, $arg)*)
    }};
}
unsafe fn class(name: &CStr) -> Id {
    objc_getClass(name.as_ptr())
}
unsafe fn string(text: &str) -> Id {
    msg!(class(c"NSString"), c"stringWithBytes:length:encoding:", Id;
        text.as_ptr() => *const u8; text.len() => usize; 4usize => usize)
}
struct State {
    enabled: bool,
    faulted: bool,
    policy: Policy,
    tap: Id,
    window: Id,
    status: Id,
    app: Id,
    smoke: bool,
}
impl Default for State {
    fn default() -> Self {
        Self {
            enabled: false,
            faulted: false,
            policy: Policy::default(),
            tap: ptr::null_mut(),
            window: ptr::null_mut(),
            status: ptr::null_mut(),
            app: ptr::null_mut(),
            smoke: false,
        }
    }
}
thread_local! { static STATE: RefCell<State> = RefCell::new(State::default()); }
unsafe fn status(text: &str) {
    let field = STATE.with(|s| s.borrow().status);
    msg!(field, c"setStringValue:", (); string(text) => Id);
}
unsafe fn pause(fault: bool) {
    let faulted = STATE.with(|s| {
        let mut s = s.borrow_mut();
        s.enabled = false;
        s.faulted |= fault;
        s.faulted
    });
    status(if faulted {
        "異常により停止：入力を確認し、アプリを再起動してください"
    } else {
        "停止中：Enterの動作を変更していません"
    });
    if fault {
        let window = STATE.with(|s| s.borrow().window);
        msg!(window, c"orderFrontRegardless", ());
    }
}
unsafe fn foreground() -> Option<i32> {
    let workspace = msg!(class(c"NSWorkspace"), c"sharedWorkspace", Id);
    let app = msg!(workspace, c"frontmostApplication", Id);
    if app.is_null() {
        return None;
    }
    let bundle = msg!(app, c"bundleIdentifier", Id);
    if bundle.is_null() {
        return None;
    }
    let matches = msg!(bundle, c"isEqualToString:", i8; string("com.hnc.Discord") => Id);
    if matches == 0 {
        None
    } else {
        Some(msg!(app, c"processIdentifier", i32))
    }
}
unsafe extern "C" fn event_callback(_: Id, kind: u32, event: Id, _: Id) -> Id {
    // Never auto-reenable a timed-out or user-disabled tap.
    if kind == 0xffff_fffe || kind == 0xffff_ffff {
        pause(true);
        return event;
    }
    if event.is_null() || (kind != 10 && kind != 11) {
        return event;
    }
    let key = CGEventGetIntegerValueField(event, 9) as u16;
    let flags = CGEventGetFlags(event);
    if kind == 10 && key == 109 && flags & (COMMAND | OPTION) == COMMAND | OPTION {
        pause(false);
        return event;
    }
    if key != 36 && key != 76 {
        return event;
    }
    let enabled = STATE.with(|s| s.borrow().enabled);
    // Query only application identity, never the focused input field or its text.
    let pool = msg!(class(c"NSAutoreleasePool"), c"new", Id);
    let target = foreground();
    msg!(pool, c"drain", ());
    let decision = STATE.with(|s| {
        s.borrow_mut().policy.handle(
            key,
            kind == 10,
            CGEventGetIntegerValueField(event, 8) != 0,
            flags,
            target,
            enabled,
        )
    });
    match decision {
        Decision::Pass => event,
        Decision::Suppress => ptr::null_mut(),
        Decision::Flags(new_flags) => {
            CGEventSetFlags(event, new_flags);
            event
        }
    }
}
unsafe extern "C" fn enable(_: Id, _: Sel, _: Id) {
    if STATE.with(|s| s.borrow().faulted) {
        status("異常後は再開できません。入力を確認し、アプリを再起動してください");
        return;
    }
    // Do not allow multiple installed copies to chain their event transformations.
    // Direct developer binaries can smoke-test but must be packaged to enable.
    let bundle = msg!(class(c"NSBundle"), c"mainBundle", Id);
    let identifier = msg!(bundle, c"bundleIdentifier", Id);
    if identifier.is_null() {
        status("停止中：配布用のEnterGuard.appから起動してください");
        return;
    }
    let instances = msg!(class(c"NSRunningApplication"), c"runningApplicationsWithBundleIdentifier:", Id;
        identifier => Id);
    if msg!(instances, c"count", usize) != 1 {
        pause(false);
        status("停止中：他のEnterGuardを終了してから有効にしてください");
        return;
    }
    if !AXIsProcessTrusted() {
        pause(false);
        status("停止中：システム設定でアクセシビリティを許可後、再起動してください");
        return;
    }
    let mut tap = STATE.with(|s| s.borrow().tap);
    if tap.is_null() {
        // Session-level active filter, not a privileged HID tap; only keyboard events.
        tap = CGEventTapCreate(
            1,
            0,
            0,
            (1 << 10) | (1 << 11),
            event_callback,
            ptr::null_mut(),
        );
        if tap.is_null() {
            pause(true);
            return;
        }
        let source = CFMachPortCreateRunLoopSource(ptr::null_mut(), tap, 0);
        if source.is_null() {
            CFRelease(tap);
            pause(true);
            return;
        }
        CFRunLoopAddSource(CFRunLoopGetMain(), source, kCFRunLoopCommonModes);
        CFRelease(source);
        STATE.with(|s| s.borrow_mut().tap = tap);
    }
    CGEventTapEnable(tap, true);
    if !CGEventTapIsEnabled(tap) {
        pause(true);
        return;
    }
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        s.enabled = true;
    });
    status("有効：Enterで改行 ／ ⌘Enterで送信");
}
unsafe extern "C" fn stop(_: Id, _: Sel, _: Id) {
    pause(false);
}
unsafe extern "C" fn quit(_: Id, _: Sel, _: Id) {
    pause(false);
    let app = STATE.with(|s| s.borrow().app);
    msg!(app, c"terminate:", (); ptr::null_mut() => Id);
}
unsafe extern "C" fn show(_: Id, _: Sel, _: Id) {
    let (window, app) = STATE.with(|s| {
        let s = s.borrow();
        (s.window, s.app)
    });
    msg!(window, c"makeKeyAndOrderFront:", (); ptr::null_mut() => Id);
    msg!(app, c"activateIgnoringOtherApps:", (); 1i8 => i8);
}
unsafe extern "C" fn permissions(_: Id, _: Sel, _: Id) {
    pause(false);
    let workspace = msg!(class(c"NSWorkspace"), c"sharedWorkspace", Id);
    let url = msg!(class(c"NSURL"), c"URLWithString:", Id;
        string("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility") => Id);
    let _ = msg!(workspace, c"openURL:", i8; url => Id);
}
unsafe extern "C" fn tick(_: Id, _: Sel, _: Id) {
    let (smoke, enabled, tap) = STATE.with(|s| {
        let s = s.borrow();
        (s.smoke, s.enabled, s.tap)
    });
    if smoke {
        // Exercise AppKit creation/run-loop/paused controls only. No tap or prompt.
        assert!(!enabled && tap.is_null());
        let (window, field) = STATE.with(|s| {
            let s = s.borrow();
            (s.window, s.status)
        });
        assert!(!window.is_null() && !field.is_null());
        assert_ne!(msg!(window, c"isVisible", i8), 0);
        pause(false);
        println!("EnterGuard macOS UI smoke test: paused, no event tap, OK");
        quit(ptr::null_mut(), ptr::null_mut(), ptr::null_mut());
        return;
    }
    if enabled && (!AXIsProcessTrusted() || !CGEventTapIsEnabled(tap)) {
        pause(true);
    }
}
unsafe fn label(view: Id, text: &str, frame: Rect) -> Id {
    let field = msg!(class(c"NSTextField"), c"labelWithString:", Id; string(text) => Id);
    msg!(field, c"setFrame:", (); frame => Rect);
    msg!(view, c"addSubview:", (); field => Id);
    field
}
unsafe fn button(view: Id, target: Id, text: &str, action: &CStr, frame: Rect) {
    let button = msg!(class(c"NSButton"), c"buttonWithTitle:target:action:", Id;
        string(text) => Id; target => Id; sel_registerName(action.as_ptr()) => Sel);
    msg!(button, c"setFrame:", (); frame => Rect);
    msg!(view, c"addSubview:", (); button => Id);
}
unsafe fn menu_item(menu: Id, target: Id, title: &str, action: &CStr, key: &str) {
    let item = msg!(class(c"NSMenuItem"), c"alloc", Id);
    let item = msg!(item, c"initWithTitle:action:keyEquivalent:", Id;
        string(title) => Id; sel_registerName(action.as_ptr()) => Sel; string(key) => Id);
    msg!(item, c"setTarget:", (); target => Id);
    msg!(menu, c"addItem:", (); item => Id);
    msg!(item, c"release", ());
}
pub fn run() {
    // Native AppKit, the event tap, timers and action target share the main thread.
    unsafe {
        if std::env::args().any(|a| a == "--permission-check") {
            println!("Accessibility permission: {}", AXIsProcessTrusted());
            return;
        }
        let smoke = std::env::args().any(|a| a == "--smoke-test");
        let pool = msg!(class(c"NSAutoreleasePool"), c"new", Id);
        let app = msg!(class(c"NSApplication"), c"sharedApplication", Id);
        let _ = msg!(app, c"setActivationPolicy:", i8; 0isize => isize);
        let cls = objc_allocateClassPair(class(c"NSObject"), c"EnterGuardActions".as_ptr(), 0);
        assert!(!cls.is_null());
        for (name, action) in [
            (c"enable:", enable as unsafe extern "C" fn(Id, Sel, Id)),
            (c"pause:", stop),
            (c"quit:", quit),
            (c"show:", show),
            (c"permissions:", permissions),
            (c"tick:", tick),
        ] {
            assert_ne!(
                class_addMethod(
                    cls,
                    sel_registerName(name.as_ptr()),
                    action,
                    c"v@:@".as_ptr()
                ),
                0
            );
        }
        objc_registerClassPair(cls);
        let target = msg!(cls, c"new", Id);
        let window = msg!(class(c"NSWindow"), c"alloc", Id);
        let window = msg!(window, c"initWithContentRect:styleMask:backing:defer:", Id;
            rect(0., 0., 760., 420.) => Rect; 1usize | 2 | 4 => usize; 2usize => usize; 0i8 => i8);
        msg!(window, c"setTitle:", (); string("EnterGuard — macOS 試用版") => Id);
        // Keep window available via the application menu after the red close button.
        msg!(window, c"setReleasedWhenClosed:", (); 0i8 => i8);
        let view = msg!(window, c"contentView", Id);
        let status_label = label(
            view,
            "停止中：Enterの動作を変更していません",
            rect(24., 366., 712., 30.),
        );
        label(
            view,
            "通常版Discordが最前面のときだけ：Enter → Shift+Enter ／ ⌘Enter → Enter",
            rect(24., 322., 712., 28.),
        );
        label(
            view,
            "最初は自分だけのチャンネルで、改行と送信を試してください。",
            rect(24., 282., 712., 28.),
        );
        label(
            view,
            "日本語IMEの変換中・入力欄の種類は判定しません。",
            rect(24., 242., 712., 28.),
        );
        label(
            view,
            "変換確定、検索、ログイン、パスワード入力、通話操作の前に停止してください。",
            rect(24., 212., 712., 28.),
        );
        label(
            view,
            "緊急停止：⌘⌥F10（環境によっては fn も押す）。停止キーでは再開しません。",
            rect(24., 168., 712., 28.),
        );
        label(
            view,
            "閉じるボタンは画面を隠します。完全終了はメニューまたは「終了する」。",
            rect(24., 138., 712., 28.),
        );
        button(
            view,
            target,
            "有効にする",
            c"enable:",
            rect(24., 83., 300., 38.),
        );
        button(
            view,
            target,
            "一時停止する",
            c"pause:",
            rect(340., 83., 180., 38.),
        );
        button(
            view,
            target,
            "終了する",
            c"quit:",
            rect(536., 83., 180., 38.),
        );
        button(
            view,
            target,
            "アクセシビリティ設定を開く（許可は本人が操作）",
            c"permissions:",
            rect(24., 28., 470., 38.),
        );
        let bar = msg!(class(c"NSMenu"), c"new", Id);
        let root = msg!(class(c"NSMenuItem"), c"new", Id);
        let menu = msg!(class(c"NSMenu"), c"new", Id);
        menu_item(menu, target, "画面を開く", c"show:", "");
        menu_item(menu, target, "一時停止する", c"pause:", "");
        menu_item(menu, target, "EnterGuardを終了", c"quit:", "q");
        msg!(root, c"setSubmenu:", (); menu => Id);
        msg!(bar, c"addItem:", (); root => Id);
        msg!(app, c"setMainMenu:", (); bar => Id);
        STATE.with(|s| {
            let mut s = s.borrow_mut();
            s.app = app;
            s.window = window;
            s.status = status_label;
            s.smoke = smoke;
        });
        let _timer = msg!(class(c"NSTimer"), c"scheduledTimerWithTimeInterval:target:selector:userInfo:repeats:", Id;
            0.5f64 => f64; target => Id; sel_registerName(c"tick:".as_ptr()) => Sel;
            ptr::null_mut() => Id; 1i8 => i8);
        msg!(window, c"center", ());
        show(ptr::null_mut(), ptr::null_mut(), ptr::null_mut());
        msg!(app, c"run", ());
        msg!(pool, c"drain", ());
    }
}
