"""Keyboard emulation via the Windows API SendInput().

From the point of view of any program (Unity, Unreal, Notepad, browser
game ...) the prototype behaves like a real keyboard. NO driver is needed -
SendInput injects the events directly into the Windows input queue.

Important for games: we send scancodes (KEYEVENTF_SCANCODE), not just
virtual keycodes, because many engines (Unity/Unreal, DirectInput) read at
the scancode level.

In dry-run mode nothing is sent, only logged.
"""

import ctypes
import time

INPUT_KEYBOARD = 1
KEYEVENTF_KEYUP = 0x0002
KEYEVENTF_SCANCODE = 0x0008
MAPVK_VK_TO_VSC = 0

VK = {
    "space": 0x20,  # jump
    "ctrl": 0xA2,   # VK_LCONTROL - crouch (held)
    "a": 0x41,      # lane left
    "d": 0x44,      # lane right
}


class _KEYBDINPUT(ctypes.Structure):
    _fields_ = [
        ("wVk", ctypes.c_ushort),
        ("wScan", ctypes.c_ushort),
        ("dwFlags", ctypes.c_ulong),
        ("time", ctypes.c_ulong),
        ("dwExtraInfo", ctypes.c_void_p),
    ]


class _INPUTUNION(ctypes.Union):
    # MOUSEINPUT is the largest union member (32 bytes on x64) - replicated as padding
    _fields_ = [("ki", _KEYBDINPUT), ("_padding", ctypes.c_ubyte * 32)]


class _INPUT(ctypes.Structure):
    _fields_ = [("type", ctypes.c_ulong), ("union", _INPUTUNION)]


class KeySender:
    def __init__(self, dry_run: bool = True):
        self.dry_run = dry_run
        if not dry_run:
            self._user32 = ctypes.windll.user32

    def _send(self, key: str, keyup: bool) -> None:
        vk = VK[key]
        if self.dry_run:
            print(f"      [DRY-RUN] {'RELEASE' if keyup else 'PRESS  '} {key}")
            return
        scan = self._user32.MapVirtualKeyW(vk, MAPVK_VK_TO_VSC)
        flags = KEYEVENTF_SCANCODE | (KEYEVENTF_KEYUP if keyup else 0)
        inp = _INPUT(type=INPUT_KEYBOARD)
        inp.union.ki = _KEYBDINPUT(0, scan, flags, 0, None)
        sent = self._user32.SendInput(1, ctypes.byref(inp), ctypes.sizeof(_INPUT))
        if sent != 1:
            print(f"      WARNUNG: SendInput fehlgeschlagen für {key}")

    def press(self, key: str) -> None:
        self._send(key, keyup=False)

    def release(self, key: str) -> None:
        self._send(key, keyup=True)

    def tap(self, key: str, hold_s: float = 0.04) -> None:
        """Short key press (press, hold briefly, release)."""
        self.press(key)
        if not self.dry_run:
            time.sleep(hold_s)  # some engines swallow 0 ms taps
        self.release(key)
