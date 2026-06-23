"""Tastatur-Emulation: Dry-Run (nur Logging) und echte Windows-Tasten.

WinKeySender uebernimmt die ctypes-SendInput-Mechanik aus dem Kinect-Prototyp
(prototyp/key_sender.py): Scancodes via KEYEVENTF_SCANCODE, weil Spiele
(Unity/Unreal/DirectInput) auf Scancode-Ebene lesen.

Gemeinsames Interface beider Sender:
    tap(key)            kurzer Druck (druecken + kurz halten + loslassen)
    hold(key, down)     down=True druecken/halten, down=False loslassen
    release_all()       Not-Aus: alle aktuell gehaltenen Tasten loslassen
"""

import ctypes
import time

import config

INPUT_KEYBOARD = 1
KEYEVENTF_KEYUP = 0x0002
KEYEVENTF_SCANCODE = 0x0008
MAPVK_VK_TO_VSC = 0

# Virtuelle Keycodes der benoetigten Tasten
VK = {
    "space": 0x20,
    "a": 0x41,
    "d": 0x44,
    "s": 0x53,
    "ctrl": 0xA2,   # VK_LCONTROL (falls jemand Ducken zurueck auf Strg legt)
}


class DryRunKeySender:
    """Druckt nichts - schreibt die Aktionen nur in die Konsole."""

    def __init__(self):
        self._held = set()

    def tap(self, key: str) -> None:
        print(f"      [DRY-RUN] TAP   {key}")

    def hold(self, key: str, down: bool) -> None:
        if down:
            self._held.add(key)
        else:
            self._held.discard(key)
        print(f"      [DRY-RUN] HOLD  {key} down={down}")

    def release_all(self) -> None:
        for key in list(self._held):
            print(f"      [DRY-RUN] RELEASE {key} (Not-Aus)")
        self._held.clear()


class _KEYBDINPUT(ctypes.Structure):
    _fields_ = [
        ("wVk", ctypes.c_ushort),
        ("wScan", ctypes.c_ushort),
        ("dwFlags", ctypes.c_ulong),
        ("time", ctypes.c_ulong),
        ("dwExtraInfo", ctypes.c_void_p),
    ]


class _INPUTUNION(ctypes.Union):
    # MOUSEINPUT ist das groesste Union-Mitglied (32 Bytes auf x64) - als Padding
    _fields_ = [("ki", _KEYBDINPUT), ("_padding", ctypes.c_ubyte * 32)]


class _INPUT(ctypes.Structure):
    _fields_ = [("type", ctypes.c_ulong), ("union", _INPUTUNION)]


class WinKeySender:
    """Echte Tastendruecke ueber die Windows-API SendInput()."""

    def __init__(self):
        self._user32 = ctypes.windll.user32
        self._held = set()

    def _send(self, key: str, keyup: bool) -> None:
        vk = VK[key]
        scan = self._user32.MapVirtualKeyW(vk, MAPVK_VK_TO_VSC)
        flags = KEYEVENTF_SCANCODE | (KEYEVENTF_KEYUP if keyup else 0)
        inp = _INPUT(type=INPUT_KEYBOARD)
        inp.union.ki = _KEYBDINPUT(0, scan, flags, 0, None)
        sent = self._user32.SendInput(1, ctypes.byref(inp), ctypes.sizeof(_INPUT))
        if sent != 1:
            print(f"      WARNUNG: SendInput fehlgeschlagen fuer {key}")

    def tap(self, key: str) -> None:
        self._send(key, keyup=False)
        time.sleep(config.TAP_HOLD_MS / 1000.0)   # manche Engines verschlucken 0ms-Taps
        self._send(key, keyup=True)

    def hold(self, key: str, down: bool) -> None:
        if down:
            self._held.add(key)
            self._send(key, keyup=False)
        else:
            self._held.discard(key)
            self._send(key, keyup=True)

    def release_all(self) -> None:
        for key in list(self._held):
            self._send(key, keyup=True)
        self._held.clear()


class SwitchableKeySender:
    """Sender, der zur Laufzeit zwischen Dry-Run und echtem Senden umschaltet.

    So kann der Send-Modus im Overlay per Taste aktiviert werden, ohne das
    Programm neu zu starten. Beim Ausschalten werden gehaltene ECHTE Tasten
    sicher losgelassen (Not-Aus).
    """

    def __init__(self, send_enabled: bool = False):
        self._win = WinKeySender()
        self.send_enabled = send_enabled

    def set_send(self, enabled: bool) -> None:
        if self.send_enabled and not enabled:
            self._win.release_all()   # echte gehaltene Tasten loslassen
        self.send_enabled = enabled
        print(f"      >> SEND-Modus {'AN (echte Tasten!)' if enabled else 'AUS (Dry-Run)'}")

    def tap(self, key: str) -> None:
        if self.send_enabled:
            self._win.tap(key)
        else:
            print(f"      [DRY-RUN] TAP   {key}")

    def hold(self, key: str, down: bool) -> None:
        if self.send_enabled:
            self._win.hold(key, down)
        else:
            print(f"      [DRY-RUN] HOLD  {key} down={down}")

    def release_all(self) -> None:
        self._win.release_all()
