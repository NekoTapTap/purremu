# Purremu

A simple project to learn about emulation and Rust.

## Working Style

- Don't change the code if user doesn't ask for it, if you think the code is wrong, you can point it out in your response.

## Decisions

### GameBoy

- Use Rust to implement a GameBoy emulator.
- Implement the CPU in m-cycles (1 m-cycle = 4 clock cycles) to make it easier to implement the PPU and APU.
