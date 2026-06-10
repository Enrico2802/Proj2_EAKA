#include "key_sender.h"

#include <chrono>
#include <cstdio>
#include <thread>

#define WIN32_LEAN_AND_MEAN
#include <windows.h>

namespace {

// Virtueller Keycode der jeweiligen Taste.
//
// C++-Hinweis: <windows.h> liefert die echten Win32-Konstanten und -Strukturen
// direkt - die ctypes-Nachbauten aus dem Python-Prototyp entfallen komplett.
WORD vk_code(Key k) {
    switch (k) {
        case Key::Space: return VK_SPACE;     // Springen
        case Key::Ctrl:  return VK_LCONTROL;  // Ducken (gehalten)
        case Key::A:     return 'A';          // Spur links
        case Key::D:     return 'D';          // Spur rechts
    }
    return 0;
}

}  // namespace

const char* to_string(Key k) {
    switch (k) {
        case Key::Space: return "space";
        case Key::Ctrl:  return "ctrl";
        case Key::A:     return "a";
        case Key::D:     return "d";
    }
    return "?";
}

KeySender::KeySender(bool dry_run) : dry_run_(dry_run) {}

void KeySender::send(Key k, bool keyup) {
    if (dry_run_) {
        std::printf("      [DRY-RUN] %s %s\n", keyup ? "RELEASE" : "PRESS  ", to_string(k));
        return;
    }
    // Virtuellen Keycode in den Hardware-Scancode übersetzen und nur diesen
    // senden (KEYEVENTF_SCANCODE) - so lesen es auch DirectInput-Engines.
    const UINT scan = MapVirtualKeyW(vk_code(k), MAPVK_VK_TO_VSC);

    INPUT inp{};  // {} nullt alle Felder (wie die 0-Argumente im Python-Code)
    inp.type = INPUT_KEYBOARD;
    inp.ki.wScan = static_cast<WORD>(scan);
    inp.ki.dwFlags = KEYEVENTF_SCANCODE | (keyup ? KEYEVENTF_KEYUP : 0);

    if (SendInput(1, &inp, sizeof(INPUT)) != 1) {
        std::printf("      WARNUNG: SendInput fehlgeschlagen für %s\n", to_string(k));
    }
}

void KeySender::press(Key k) { send(k, /*keyup=*/false); }

void KeySender::release(Key k) { send(k, /*keyup=*/true); }

void KeySender::tap(Key k) {
    press(k);
    if (!dry_run_) {
        // manche Engines verschlucken 0ms-Taps
        std::this_thread::sleep_for(std::chrono::milliseconds(40));
    }
    release(k);
}
