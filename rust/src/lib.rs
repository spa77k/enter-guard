//! OS-independent key policy and partial-insertion recovery, ported from v0.1.0.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Pass,
    Suppress,
    Newline,
    Send,
}
#[derive(Debug, Clone, Copy, Default)]
pub struct Modifiers {
    pub shift: bool,
    pub left: bool,
    pub right: bool,
    pub alt: bool,
    pub windows: bool,
}
impl Modifiers {
    pub fn control(self) -> bool {
        self.left || self.right
    }
    pub fn can_remap(self) -> bool {
        !self.shift && !self.alt && !self.windows
    }
}
#[derive(Default)]
pub struct Policy {
    held: bool,
    repeats: bool,
    release: bool,
}
impl Policy {
    pub fn handle(
        &mut self,
        down: bool,
        enabled: bool,
        discord: bool,
        m: Modifiers,
        injected: bool,
    ) -> Action {
        if injected {
            return Action::Pass;
        }
        if !down {
            let action = if self.held && self.release {
                Action::Suppress
            } else {
                Action::Pass
            };
            *self = Self::default();
            return action;
        }
        let eligible = enabled && discord && m.can_remap();
        if self.held {
            self.repeats |= eligible;
            return if self.repeats {
                Action::Suppress
            } else {
                Action::Pass
            };
        }
        self.held = true;
        self.repeats = eligible;
        self.release = eligible;
        if !eligible {
            Action::Pass
        } else if m.control() {
            Action::Send
        } else {
            Action::Newline
        }
    }
}
pub const ENTER: u16 = 0x0D;
pub const SHIFT: u16 = 0xA0;
pub const LEFT_CTRL: u16 = 0xA2;
pub const RIGHT_CTRL: u16 = 0xA3;
pub const INJECTION_TAG: usize = 0x44454731;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Key {
    pub vk: u16,
    pub up: bool,
    pub extended: bool,
}
impl Key {
    pub fn new(vk: u16, up: bool, extended: bool) -> Self {
        Self { vk, up, extended }
    }
}
pub fn sequence(action: Action, m: Modifiers, keypad: bool) -> Option<Vec<Key>> {
    if !m.can_remap()
        || !matches!(action, Action::Newline | Action::Send)
        || (action == Action::Newline && m.control())
        || (action == Action::Send && !m.control())
    {
        return None;
    }
    let mut keys = Vec::with_capacity(6);
    if action == Action::Newline {
        keys.push(Key::new(SHIFT, false, false));
    } else {
        if m.left {
            keys.push(Key::new(LEFT_CTRL, true, false));
        }
        if m.right {
            keys.push(Key::new(RIGHT_CTRL, true, true));
        }
    }
    keys.push(Key::new(ENTER, false, keypad));
    keys.push(Key::new(ENTER, true, keypad));
    if action == Action::Newline {
        keys.push(Key::new(SHIFT, true, false));
    } else {
        if m.left {
            keys.push(Key::new(LEFT_CTRL, false, false));
        }
        if m.right {
            keys.push(Key::new(RIGHT_CTRL, false, true));
        }
    }
    Some(keys)
}
/// Restore only state changed by the accepted prefix. Never retry Enter-down.
pub fn cleanup(keys: &[Key], inserted: usize, original: Modifiers) -> Vec<Key> {
    let (mut enter, mut shift, mut left, mut right, mut keypad) =
        (false, false, original.left, original.right, false);
    for key in keys.iter().take(inserted) {
        match key.vk {
            ENTER => {
                enter = !key.up;
                keypad = key.extended;
            }
            SHIFT => shift = !key.up,
            LEFT_CTRL => left = !key.up,
            RIGHT_CTRL => right = !key.up,
            _ => (),
        }
    }
    let mut result = Vec::with_capacity(4);
    if enter {
        result.push(Key::new(ENTER, true, keypad));
    }
    if shift {
        result.push(Key::new(SHIFT, true, false));
    }
    if original.left && !left {
        result.push(Key::new(LEFT_CTRL, false, false));
    }
    if original.right && !right {
        result.push(Key::new(RIGHT_CTRL, false, true));
    }
    result
}
pub fn is_discord_path(path: &str) -> bool {
    path.rsplit(['/', '\\'])
        .next()
        .is_some_and(|name| name.eq_ignore_ascii_case("Discord.exe"))
}
#[cfg(windows)]
pub fn encode(keys: &[Key]) -> Vec<windows_sys::Win32::UI::Input::KeyboardAndMouse::INPUT> {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::*;
    keys.iter()
        .map(|k| INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: k.vk,
                    wScan: 0,
                    dwFlags: if k.up { KEYEVENTF_KEYUP } else { 0 }
                        | if k.extended { KEYEVENTF_EXTENDEDKEY } else { 0 },
                    time: 0,
                    dwExtraInfo: INJECTION_TAG,
                },
            },
        })
        .collect()
}
