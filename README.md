# Purremu

A simple project to learn about emulation and Rust.

## Legal Notice

Purremu is an educational, independently developed emulator for Game Boy-compatible software.

Purremu is not affiliated with, endorsed by, or sponsored by Nintendo. Game Boy is a trademark of Nintendo.

This project does not include ROMs, BIOS or boot ROMs, keys, copyrighted game assets, or links to obtain copyrighted games. Users are responsible for ensuring they have the legal right to use any software they run with this emulator.

## Why Purremu?

Purremu is a combination of "purr" and "emulator".

Cat is liquid, it can flow into any shape. I want to be flexible and adaptable like a cat.

Purr is the sound that a cat makes when it is happy, so I named my project Purremu. Purr~

## Desktop Frontend

Run the desktop frontend with a ROM path as its only command-line argument:

```shell
cargo run -p purremu-gb-desktop -- path/to/your-rom.gb
```

Controls:

- `W`, `A`, `S`, `D`: D-pad
- `J`, `K`: A and B
- `U`, `I`: Select and Start
- `Escape`: Quit

## Roadmap

### GameBoy

- [ ] Simple Bus and memory map
- [ ] Basic CPU registers and instructions
- [ ] Serial output
- [ ] PPU: scanline、background、window、sprite
