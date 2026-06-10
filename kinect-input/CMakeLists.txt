# C++-Port des Kinect-Steuerungs-Prototyps (siehe ../prototyp fuer die Python-Referenz).
#
# CMake-Konzept: Diese Datei beschreibt WAS gebaut wird (Targets + deren
# Abhaengigkeiten), nicht WIE - daraus generiert CMake die eigentlichen
# Build-Dateien (Ninja, Makefiles, Visual-Studio-Projekte ...).

cmake_minimum_required(VERSION 3.16)
project(kinect_input CXX)

# C++20 wegen Designated Initializers ({.calib_frames = 10}) und std::numbers::pi
set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

if(MSVC)
    add_compile_options(/W4 /utf-8)
else()
    add_compile_options(-Wall -Wextra)
endif()

# Target 1: die Kernlogik als statische Bibliothek, damit Hauptprogramm UND
# Tests denselben Code teilen (entspricht den importierbaren Python-Modulen).
add_library(kinect_core
    src/gesture_detector.cpp
    src/key_sender.cpp
    src/sources/mock_source.cpp
    src/sources/manual_source.cpp
)
# PUBLIC: auch wer kinect_core benutzt, darf #include "gesture_detector.h" etc.
target_include_directories(kinect_core PUBLIC src)
target_link_libraries(kinect_core PUBLIC user32)  # Win32: SendInput() / MapVirtualKeyW()

# Target 2: das Hauptprogramm (Pendant zu main.py)
add_executable(kinect-input src/main.cpp)
target_link_libraries(kinect-input PRIVATE kinect_core)

# Target 3: assert-basiertes Test-Programm, ueber CTest aufrufbar
# (Exit-Code != 0 => Test fehlgeschlagen)
enable_testing()
add_executable(test_gesture_detector tests/test_gesture_detector.cpp)
target_link_libraries(test_gesture_detector PRIVATE kinect_core)
add_test(NAME gesture_detector COMMAND test_gesture_detector)
