# SDVX Controller Firmware
DIY SDVX Controller Firmware for RP235X microcontroller family.

## Key Features
- Gamepad, keyboard, mouse input methods.
- Dynamic keymapping using [Via](https://www.usevia.app/).
- Multithreaded LED control.
- Efficient cooperative multitasking architecture via async Rust([Embassy](https://embassy.dev/)).

## Requirements
- Rust Toolchain (for development. With `thumbv8m.main-none-eabihf` target)
- picotool (for flashing the firmware)

## Flashing
To build and flash firmware, use the following command on workspace root:
```bash
cargo flash --release
```

## GPIO Pinouts
0. Button1
1. Button2
2. Button3
3. Button4
4. FX Button1
5. FX Button2
6. Start Button
7. unused
8. Button1 LED Control
9. Button2 LED Control
10. Button3 LED Control
11. Button4 LED Control
12. FX Button1 LED Control
13. FX Button2 LED Control
14. Start Button LED Control
15. unused
16. unused
17. unused
18. unused
19. unused
20. unused
21. unused
22. unused
23. unused
24. unused
25. unused
26. Left knob
27. Right knob
28. unused

## License
The firmware source code is licensed under GPL-2.0.