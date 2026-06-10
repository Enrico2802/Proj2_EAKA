#pragma once

// Tastatur-Emulation über die Windows-API SendInput().
//
// Damit verhält sich das Programm aus Sicht jedes anderen Programms (Unity,
// Unreal, Notepad, Browser-Spiel ...) wie eine echte Tastatur. Es wird KEIN
// Treiber benötigt - SendInput injiziert die Events direkt in die
// Windows-Eingabequeue.
//
// Wichtig für Spiele: Wir senden Scancodes (KEYEVENTF_SCANCODE), nicht nur
// virtuelle Keycodes, weil viele Engines (Unity/Unreal, DirectInput) auf
// Scancode-Ebene lesen.
//
// Im Dry-Run-Modus wird nichts gesendet, sondern nur geloggt.
//
// C++-Hinweis: Dieser Header bleibt bewusst frei von <windows.h> - die ganze
// Win32-Maschinerie ist ein Implementierungsdetail von key_sender.cpp.
// (Header/Source-Trennung: Wer KeySender benutzt, sieht nur die Schnittstelle.)

// Die Tasten, die wir brauchen (Tastenbelegung laut Team-Absprache).
enum class Key {
    Space,  // Springen (Tap)
    Ctrl,   // Ducken (gehalten)
    A,      // Spur links (Tap)
    D,      // Spur rechts (Tap)
};

const char* to_string(Key k);

class KeySender {
public:
    explicit KeySender(bool dry_run = true);

    void press(Key k);
    void release(Key k);

    // Kurzer Tastendruck (drücken, 40ms halten, loslassen).
    void tap(Key k);

private:
    void send(Key k, bool keyup);

    bool dry_run_;
};
