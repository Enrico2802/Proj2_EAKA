use std::fmt;
use std::io;
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Space,
    Ctrl,
    A,
    D,
}

impl Key {
    pub fn as_str(self) -> &'static str {
        match self {
            Key::Space => "space",
            Key::Ctrl => "ctrl",
            Key::A => "a",
            Key::D => "d",
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
    dry_run: bool,
}

impl KeySender {
    pub fn new(dry_run: bool) -> Self {
        Self { dry_run }
    }

    pub fn press(&self, key: Key) -> io::Result<()> {
        self.send(key, false)
    }

    pub fn release(&self, key: Key) -> io::Result<()> {
        self.send(key, true)
    }

    pub fn tap(&self, key: Key) -> io::Result<()> {
        self.press(key)?;
        if !self.dry_run {
            thread::sleep(Duration::from_millis(40));
        }
        self.release(key)
    }

    fn send(&self, key: Key, keyup: bool) -> io::Result<()> {
        if self.dry_run {
            println!(
                "      [DRY-RUN] {} {}",
                if keyup { "RELEASE" } else { "PRESS  " },
                key
            );
            return Ok(());
        }

        platform::send_key(key, keyup)
    }
}

#[cfg(windows)]
mod platform {
    use std::io;
    use std::mem;

    use super::Key;

    const INPUT_KEYBOARD: u32 = 1;
    const KEYEVENTF_KEYUP: u32 = 0x0002;
    const KEYEVENTF_SCANCODE: u32 = 0x0008;
    const MAPVK_VK_TO_VSC: u32 = 0;

    const VK_SPACE: u32 = 0x20;
    const VK_LCONTROL: u32 = 0xA2;

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

    pub fn send_key(key: Key, keyup: bool) -> io::Result<()> {
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
        if sent == 1 {
            Ok(())
        } else {
            Err(io::Error::last_os_error())
        }
    }

    fn vk_code(key: Key) -> u32 {
        match key {
            Key::Space => VK_SPACE,
            Key::Ctrl => VK_LCONTROL,
            Key::A => b'A' as u32,
            Key::D => b'D' as u32,
        }
    }
}

#[cfg(not(windows))]
mod platform {
    use std::io;

    use super::Key;

    pub fn send_key(_key: Key, _keyup: bool) -> io::Result<()> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "real key sending is only implemented on Windows",
        ))
    }
}
