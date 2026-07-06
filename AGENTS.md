# Purremu

A simple project to learn about emulation and Rust.

## Working Style

- Don't change the code if user doesn't ask for it, if you think the code is wrong, you can point it out in your response.

## Legal and Branding Safety

- When assisting with coding, documentation, examples, tests, or assets, do not add official Nintendo visual assets, logos, screenshots, sprites, box art, UI captures, ROMs, BIOS or boot ROMs, keys, copied official manuals, or links to obtain copyrighted games.

## Decisions

### GameBoy

- Use Rust to implement a GameBoy emulator.
- Implement the CPU in m-cycles (1 m-cycle = 4 clock cycles) to make it easier to implement the PPU and APU.
