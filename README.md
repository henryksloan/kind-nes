# KindNES
KindNES is a reasonably accurate NES emulator written in Rust. It strives for portability, performance, and a balance of hardware accuracy and codebase clarity. The code should pair well with [NESdev](https://wiki.nesdev.com/) as a resource for learning about the NES.

### Usage
Either give a .NES ROM file as a command line argument, or use the File > Open ROM menu bar option (currently only on the Windows version).

| Button | Key |
| --- | --- |
| D-Pad | Arrow keys |
| A button | X |
| B button | Z |
| Start | Enter |
| Select | Right shift |

# Downloads
*WIP*

Downloads will be available via Github releases as soon as all platforms have a packaged MVP.

# Building
## Dependencies
Both the Windows and cross-platform frontends require Rust-SDL2. To build the frontends, first check out the dependency instructions here: https://github.com/Rust-SDL2/rust-sdl2#sdl20-development-libraries

## Windows
To run the native Windows frontend, run:

`cargo run --release --bin windows-ui`

To build a binary, run:

`cargo build --release --bin windows-ui`

This should produce a `.exe` file in `target/release/`. Place SDL2.dll from the runtime binary [here](https://www.libsdl.org/download-2.0.php) in the same directory as the exe.


## Linux, MacOS, etc.
To run the cross-platform frontend, run:

`cargo run --release --bin sdl-ui <NES ROM file>`

Replacing `<NES ROM file>` with a `.nes` file.

To build a binary, run:

`cargo build --release --bin sdl-ui`

This should produce an executable file in `target/release/`.

# Progress
KindNES supports most of the common NES mappers, meaning that it supports the majority of licensed titles. Most supported games run smoothly with minimal glitches. The basic gameplay experience is in a semi-complete state, so progress moving forward will add UI/UX improvements and improved game/peripheral support.

## Next steps
- Improved UI
    - More menubar features
        - Pause, (soft/hard) reset
    - An improved cross-platform UI with the same menubar features as the Windows version
        - First priority: Centralize basic features like [file dialogs](https://github.com/EmbarkStudios/nfd2) to sdl-ui shortcuts
        - Still looking for a GUI framework with great menubar support and SDL2 integration
- Saving
    - Safe panic handler that displays an error message and autosaves

## Eventual goals
- More mappers
- Speed control
    - First, get perfect FPS control (it currently sleeps slightly too long at the end of frames)
    - Variable audio sample rate
- Controls
    - Gamepad support
    - Modifiable controls
    - Local multiplayer?

## Stretch goals
- Web version (likely using WebAssembly)
- Mobile versions (possibly merged with web version?)
- Netplay
- Better hardware accuracy
    - Automated validation with test ROMs
- Video filters
- Extended controller support
    - Popup windows for R.O.B., etc.
    - Light gun with mouse
- Debug features
    - PPU viewer
    - Sound channel mixer
    - Step-in debugger
        - A GDB-style command prompt would be awesome
- Cheats
- TAS creation