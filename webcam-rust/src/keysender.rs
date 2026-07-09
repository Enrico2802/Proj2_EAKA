//! Keyboard emulation: switchable at runtime between dry run and real
//! sending (port of keysender.py / SwitchableKeySender).
//!
//! Real sending uses Windows SendInput with SCANCODES (raw FFI, no extra
//! crate) because games read scancodes more reliably.

use std::fmt;
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    W,
    A,
    D,
    S,
}

impl Key {
    pub fn as_str(self) -> &'static str {
        match self {
            Key::W => "w",
            Key::A => "a",
            Key::D => "d",
            Key::S => "s",
        }
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug)]
pub struct KeySender {
    send_enabled: bool,
    held: Vec<Key>,
}

impl KeySender {
    pub fn new(send_enabled: bool) -> Self {
        Self { send_enabled, held: Vec::new() }
    }

    pub fn send_enabled(&self) -> bool {
        self.send_enabled
    }

    pub fn set_send(&mut self, enabled: bool) {
        if self.send_enabled && !enabled {
            self.release_all(); // release real held keys before switching off
        }
        self.send_enabled = enabled;
        println!(
            "      >> SEND-Modus {}",
            if enabled { "AN (echte Tasten!)" } else { "AUS (Dry-Run)" }
        );
    }

    pub fn tap(&self, key: Key) {
        if !self.send_enabled {
            println!("      [DRY-RUN] TAP   {key}");
            return;
        }
        platform::send_key(key, false);
        thread::sleep(Duration::from_millis(40));
        platform::send_key(key, true);
    }

    pub fn hold(&mut self, key: Key, down: bool) {
        if !self.send_enabled {
            println!("      [DRY-RUN] HOLD  {key} down={down}");
            return;
        }
        if down {
            if !self.held.contains(&key) {
                self.held.push(key);
            }
            platform::send_key(key, false);
        } else {
            self.held.retain(|&k| k != key);
            platform::send_key(key, true);
        }
    }

    /// Emergency stop: release all currently held real keys.
    pub fn release_all(&mut self) {
        for &key in &self.held {
            platform::send_key(key, true);
        }
        self.held.clear();
    }
}

#[cfg(windows)]
mod platform {
    use std::mem;

    use super::Key;

    const INPUT_KEYBOARD: u32 = 1;
    const KEYEVENTF_KEYUP: u32 = 0x0002;
    const KEYEVENTF_SCANCODE: u32 = 0x0008;
    const MAPVK_VK_TO_VSC: u32 = 0;

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct KeybdInput {
        w_vk: u16,
        w_scan: u16,
        dw_flags: u32,
        time: u32,
        dw_extra_info: usize,
    }

    #[repr(C)]
    union InputUnion {
        ki: KeybdInput,
        _padding: [u8; 32],
    }

    #[repr(C)]
    struct Input {
        r#type: u32,
        u: InputUnion,
    }

    #[link(name = "user32")]
    extern "system" {
        fn MapVirtualKeyW(u_code: u32, u_map_type: u32) -> u32;
        fn SendInput(c_inputs: u32, p_inputs: *const Input, cb_size: i32) -> u32;
    }

    pub fn send_key(key: Key, keyup: bool) {
        let scan = unsafe { MapVirtualKeyW(vk_code(key), MAPVK_VK_TO_VSC) };
        let input = Input {
            r#type: INPUT_KEYBOARD,
            u: InputUnion {
                ki: KeybdInput {
                    w_vk: 0,
                    w_scan: scan as u16,
                    dw_flags: KEYEVENTF_SCANCODE | if keyup { KEYEVENTF_KEYUP } else { 0 },
                    time: 0,
                    dw_extra_info: 0,
                },
            },
        };
        let sent = unsafe { SendInput(1, &input, mem::size_of::<Input>() as i32) };
        if sent != 1 {
            eprintln!("      WARNUNG: SendInput fehlgeschlagen fuer {key}");
        }
    }

    fn vk_code(key: Key) -> u32 {
        match key {
            Key::W => b'W' as u32,
            Key::A => b'A' as u32,
            Key::D => b'D' as u32,
            Key::S => b'S' as u32,
        }
    }
}

#[cfg(not(windows))]
mod platform {
    use super::Key;

    pub fn send_key(_key: Key, _keyup: bool) {
        eprintln!("      (echtes Senden nur unter Windows implementiert)");
    }
}
