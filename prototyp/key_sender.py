"""Tastatur-Emulation über die Windows-API SendInput().

Damit verhält sich der Prototyp aus Sicht jedes Programms (Unity, Unreal,
Notepad, Browser-Spiel ...) wie eine echte Tastatur. Es wird KEIN Treiber
benötigt - SendInput injiziert die Events direkt in die Windows-Eingabequeue.

Wichtig für Spiele: Wir senden Scancodes (KEYEVENTF_SCANCODE), nicht nur
virtuelle Keycodes, weil viele Engines (Unity/Unreal, DirectInput) auf
Scancode-Ebene lesen.

Im Dry-Run-Modus wird nichts gesendet, sondern nur geloggt.
"""

import ctypes
import time

INPUT_KEYBOARD = 1
KEYEVENTF_KEYUP = 0x0002
KEYEVENTF_SCANCODE = 0x0008
MAPVK_VK_TO_VSC = 0

# Virtuelle Keycodes der Tasten, die wir brauchen
VK = {
    "space": 0x20,  # Springen
    "ctrl": 0xA2,   # VK_LCONTROL - Ducken (gehalten)
    "a": 0x41,      # Spur links
    "d": 0x44,      # Spur rechts
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
    # MOUSEINPUT ist das größte Union-Mitglied (32 Bytes auf x64) - als Padding nachgebildet
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
        """Kurzer Tastendruck (drücken, kurz halten, loslassen)."""
        self.press(key)
        if not self.dry_run:
            time.sleep(hold_s)  # manche Engines verschlucken 0ms-Taps
        self.release(key)
