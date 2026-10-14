use enter_guard::*;
fn mods(mask: u8) -> Modifiers {
    Modifiers {
        shift: mask & 1 != 0,
        left: mask & 2 != 0,
        right: mask & 4 != 0,
        alt: mask & 8 != 0,
        windows: mask & 16 != 0,
    }
}
#[test]
fn every_initial_combination() {
    for enabled in [false, true] {
        for discord in [false, true] {
            for injected in [false, true] {
                for mask in 0..32 {
                    let m = mods(mask);
                    let eligible = enabled && discord && !injected && m.can_remap();
                    let first = if eligible {
                        if m.control() {
                            Action::Send
                        } else {
                            Action::Newline
                        }
                    } else {
                        Action::Pass
                    };
                    let later = if eligible {
                        Action::Suppress
                    } else {
                        Action::Pass
                    };
                    let mut p = Policy::default();
                    assert_eq!(p.handle(true, enabled, discord, m, injected), first);
                    assert_eq!(p.handle(true, enabled, discord, m, injected), later);
                    assert_eq!(p.handle(false, enabled, discord, m, injected), later);
                }
            }
        }
    }
}
#[test]
fn every_held_transition() {
    // Exhaust every enable/foreground/modifier transition while held. No second action.
    for first_mask in 0..32 {
        for later_mask in 0..32 {
            for first_enabled in [false, true] {
                for first_discord in [false, true] {
                    for later_enabled in [false, true] {
                        for later_discord in [false, true] {
                            let a = mods(first_mask);
                            let b = mods(later_mask);
                            let mapped = first_enabled && first_discord && a.can_remap();
                            let mut p = Policy::default();
                            p.handle(true, first_enabled, first_discord, a, false);
                            let suppressed =
                                mapped || (later_enabled && later_discord && b.can_remap());
                            assert_eq!(
                                p.handle(true, later_enabled, later_discord, b, false),
                                if suppressed {
                                    Action::Suppress
                                } else {
                                    Action::Pass
                                }
                            );
                            // Once suppressed, stay so even if stopped or moved to another app.
                            assert_eq!(
                                p.handle(true, false, false, mods(1), false),
                                if suppressed {
                                    Action::Suppress
                                } else {
                                    Action::Pass
                                }
                            );
                            assert_eq!(
                                p.handle(false, later_enabled, later_discord, b, false),
                                if mapped {
                                    Action::Suppress
                                } else {
                                    Action::Pass
                                }
                            );
                            assert_eq!(p.handle(false, true, true, b, false), Action::Pass);
                        }
                    }
                }
            }
        }
    }
}
#[test]
fn injected_events_and_separate_enters() {
    let mut main = Policy::default();
    let mut keypad = Policy::default();
    assert_eq!(
        main.handle(true, true, true, mods(0), false),
        Action::Newline
    );
    assert_eq!(
        keypad.handle(true, true, true, mods(2), false),
        Action::Send
    );
    assert_eq!(main.handle(false, true, true, mods(0), true), Action::Pass);
    assert_eq!(main.handle(true, true, true, mods(2), true), Action::Pass);
    assert_eq!(
        keypad.handle(false, true, true, mods(0), false),
        Action::Suppress
    );
    assert_eq!(
        main.handle(true, true, true, mods(2), false),
        Action::Suppress
    );
    assert_eq!(
        main.handle(false, false, false, mods(0), false),
        Action::Suppress
    );
    assert_eq!(main.handle(true, true, true, mods(2), false), Action::Send);
}
#[test]
fn exact_sequences() {
    for keypad in [false, true] {
        assert_eq!(
            sequence(Action::Newline, mods(0), keypad).unwrap(),
            vec![
                Key::new(SHIFT, false, false),
                Key::new(ENTER, false, keypad),
                Key::new(ENTER, true, keypad),
                Key::new(SHIFT, true, false)
            ]
        );
        for mask in [2, 4, 6] {
            let m = mods(mask);
            let keys = sequence(Action::Send, m, keypad).unwrap();
            let mut expected = Vec::new();
            if m.left {
                expected.push(Key::new(LEFT_CTRL, true, false));
            }
            if m.right {
                expected.push(Key::new(RIGHT_CTRL, true, true));
            }
            expected.extend([
                Key::new(ENTER, false, keypad),
                Key::new(ENTER, true, keypad),
            ]);
            if m.left {
                expected.push(Key::new(LEFT_CTRL, false, false));
            }
            if m.right {
                expected.push(Key::new(RIGHT_CTRL, false, true));
            }
            assert_eq!(keys, expected);
        }
    }
}
#[test]
fn every_partial_insertion_recovers_without_resending() {
    for keypad in [false, true] {
        for mask in [0, 2, 4, 6] {
            let m = mods(mask);
            let keys = sequence(
                if m.control() {
                    Action::Send
                } else {
                    Action::Newline
                },
                m,
                keypad,
            )
            .unwrap();
            for prefix in 0..=keys.len() {
                let repair = cleanup(&keys, prefix, m);
                assert!(repair.iter().all(|k| k.vk != ENTER || k.up));
                assert!(
                    repair
                        .iter()
                        .filter(|k| k.vk == ENTER)
                        .all(|k| k.extended == keypad)
                );
                assert!(
                    repair
                        .iter()
                        .filter(|k| k.vk == RIGHT_CTRL)
                        .all(|k| k.extended)
                );
                if prefix == 0 || prefix == keys.len() {
                    assert!(repair.is_empty());
                }
                let (mut enter, mut shift, mut left, mut right) = (false, false, m.left, m.right);
                for k in keys.iter().take(prefix).chain(repair.iter()) {
                    match k.vk {
                        ENTER => enter = !k.up,
                        SHIFT => shift = !k.up,
                        LEFT_CTRL => left = !k.up,
                        RIGHT_CTRL => right = !k.up,
                        _ => panic!("unexpected key"),
                    }
                }
                assert!(!enter && !shift && left == m.left && right == m.right);
            }
        }
    }
}
#[test]
fn reject_unsafe_sequences_and_processes() {
    for mask in 0..32 {
        for action in [
            Action::Pass,
            Action::Suppress,
            Action::Newline,
            Action::Send,
        ] {
            let m = mods(mask);
            let valid = m.can_remap()
                && ((action == Action::Send && m.control())
                    || (action == Action::Newline && !m.control()));
            assert_eq!(sequence(action, m, false).is_some(), valid);
        }
    }
    for path in [r"C:\Users\user\Discord\Discord.exe", r"C:\Discord.EXE"] {
        assert!(is_discord_path(path));
    }
    for path in [
        r"C:\DiscordCanary.exe",
        r"C:\DiscordPTB.exe",
        r"C:\Discord.exe\chrome.exe",
        r"C:\NotDiscord.exe",
        "Discord",
        "",
    ] {
        assert!(!is_discord_path(path));
    }
}
#[cfg(windows)]
#[test]
fn native_encoding_and_layout() {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::*;
    assert_eq!(std::mem::size_of::<INPUT>(), 40);
    assert_eq!(std::mem::offset_of!(INPUT, Anonymous), 8);
    assert_eq!(std::mem::size_of::<KEYBDINPUT>(), 24);
    for keypad in [false, true] {
        for mask in [0, 2, 4, 6] {
            let m = mods(mask);
            let keys = sequence(
                if m.control() {
                    Action::Send
                } else {
                    Action::Newline
                },
                m,
                keypad,
            )
            .unwrap();
            let inputs = encode(&keys);
            assert_eq!(inputs.len(), keys.len());
            for (input, key) in inputs.iter().zip(keys) {
                assert_eq!(input.r#type, INPUT_KEYBOARD);
                let k = unsafe { input.Anonymous.ki };
                assert_eq!(k.wVk, key.vk);
                assert_eq!(k.wScan, 0);
                assert_eq!(k.time, 0);
                assert_eq!(
                    k.dwFlags,
                    if key.up { KEYEVENTF_KEYUP } else { 0 }
                        | if key.extended {
                            KEYEVENTF_EXTENDEDKEY
                        } else {
                            0
                        }
                );
                assert_eq!(k.dwExtraInfo, INJECTION_TAG);
            }
        }
    }
}
