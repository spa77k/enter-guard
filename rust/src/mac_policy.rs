//! macOS event policy. Changes flags, never synthesizes modifier or text events.
pub const SHIFT: u64 = 1 << 17;
pub const CONTROL: u64 = 1 << 18;
pub const OPTION: u64 = 1 << 19;
pub const COMMAND: u64 = 1 << 20;
const DEVICE_COMMAND: u64 = 0x08 | 0x10;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Decision {
    Pass,
    Flags(u64),
    Suppress,
}
#[derive(Clone, Copy, Debug)]
struct Held {
    pid: i32,
    send: bool,
}
#[derive(Default)]
pub struct Policy {
    // Return and keypad Enter are separate physical keys.
    held: [Option<Held>; 2],
}
impl Policy {
    pub fn handle(
        &mut self,
        key: u16,
        down: bool,
        repeat: bool,
        flags: u64,
        target: Option<i32>,
        enabled: bool,
    ) -> Decision {
        let slot = match key {
            36 => 0,
            76 => 1,
            _ => return Decision::Pass,
        };
        if !down {
            let held = self.held[slot].take();
            return match held {
                Some(h) if target == Some(h.pid) => Decision::Flags(remap(flags, h.send)),
                _ => Decision::Pass,
            };
        }
        if let Some(h) = self.held[slot] {
            // Do not affect another application if focus changes during a press.
            return if target == Some(h.pid) {
                Decision::Suppress
            } else {
                Decision::Pass
            };
        }
        if !enabled {
            return Decision::Pass;
        }
        // Never start transforming a key that was already held before activation.
        if flags & (SHIFT | CONTROL | OPTION) != 0 {
            return Decision::Pass;
        }
        if let Some(pid) = target {
            if repeat {
                return Decision::Suppress;
            }
            let send = flags & COMMAND != 0;
            self.held[slot] = Some(Held { pid, send });
            Decision::Flags(remap(flags, send))
        } else {
            Decision::Pass
        }
    }
}
fn remap(flags: u64, send: bool) -> u64 {
    if send {
        flags & !(COMMAND | DEVICE_COMMAND)
    } else {
        flags | SHIFT
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn newline_release_and_repeat() {
        let mut p = Policy::default();
        assert_eq!(
            p.handle(36, true, false, 0, Some(4), true),
            Decision::Flags(SHIFT)
        );
        assert_eq!(
            p.handle(36, true, true, 0, Some(4), true),
            Decision::Suppress
        );
        assert_eq!(
            p.handle(36, false, false, 0, Some(4), true),
            Decision::Flags(SHIFT)
        );
    }
    #[test]
    fn command_send_preserves_unrelated_flags() {
        let mut p = Policy::default();
        let caps = 1 << 16;
        assert_eq!(
            p.handle(
                76,
                true,
                false,
                COMMAND | DEVICE_COMMAND | caps,
                Some(4),
                true
            ),
            Decision::Flags(caps)
        );
        assert_eq!(
            p.handle(76, false, false, COMMAND, Some(4), true),
            Decision::Flags(0)
        );
    }
    #[test]
    fn modifiers_and_other_apps_unchanged() {
        for flags in [SHIFT, CONTROL, OPTION, SHIFT | COMMAND, CONTROL | COMMAND] {
            assert_eq!(
                Policy::default().handle(36, true, false, flags, Some(4), true),
                Decision::Pass
            );
        }
        assert_eq!(
            Policy::default().handle(36, true, false, 0, None, true),
            Decision::Pass
        );
        assert_eq!(
            Policy::default().handle(0, true, false, 0, Some(4), true),
            Decision::Pass
        );
    }
    #[test]
    fn focus_pause_and_activation_mid_hold_do_not_send() {
        let mut p = Policy::default();
        assert_eq!(
            p.handle(36, true, true, COMMAND, Some(4), true),
            Decision::Suppress
        );
        p.handle(36, true, false, COMMAND, Some(4), true);
        assert_eq!(
            p.handle(36, true, true, COMMAND, Some(8), true),
            Decision::Pass
        );
        assert_eq!(
            p.handle(36, true, true, COMMAND, None, true),
            Decision::Pass
        );
        assert_eq!(
            p.handle(36, true, true, COMMAND, Some(4), true),
            Decision::Suppress
        );
        assert_eq!(
            p.handle(36, false, false, COMMAND, Some(8), true),
            Decision::Pass
        );
        assert_eq!(
            p.handle(36, true, false, 0, Some(4), true),
            Decision::Flags(SHIFT)
        );
    }
    #[test]
    fn pause_drains_held_key_without_starting_new_transforms() {
        let mut p = Policy::default();
        assert_eq!(
            p.handle(36, true, false, COMMAND, Some(4), true),
            Decision::Flags(0)
        );
        assert_eq!(
            p.handle(36, true, true, COMMAND, Some(4), false),
            Decision::Suppress
        );
        assert_eq!(p.handle(76, true, false, 0, Some(4), false), Decision::Pass);
        assert_eq!(
            p.handle(36, true, true, COMMAND, Some(8), false),
            Decision::Pass
        );
        // Re-enable while held does not create another Enter-down.
        assert_eq!(
            p.handle(36, true, true, COMMAND, Some(4), true),
            Decision::Suppress
        );
        assert_eq!(
            p.handle(36, false, false, COMMAND, Some(4), false),
            Decision::Flags(0)
        );
        assert_eq!(
            p.handle(36, true, false, COMMAND, Some(4), false),
            Decision::Pass
        );
        assert_eq!(
            p.handle(36, false, false, COMMAND, Some(4), false),
            Decision::Pass
        );
        assert_eq!(
            p.handle(36, true, false, COMMAND, Some(4), true),
            Decision::Flags(0)
        );
    }
    #[test]
    fn return_and_keypad_have_independent_lifecycles() {
        let mut p = Policy::default();
        assert_eq!(
            p.handle(36, true, false, 0, Some(4), true),
            Decision::Flags(SHIFT)
        );
        assert_eq!(
            p.handle(76, true, false, COMMAND, Some(4), true),
            Decision::Flags(0)
        );
        assert_eq!(
            p.handle(36, false, false, 0, Some(4), true),
            Decision::Flags(SHIFT)
        );
        assert_eq!(
            p.handle(76, true, true, COMMAND, Some(4), true),
            Decision::Suppress
        );
    }
}
