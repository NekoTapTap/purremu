# Hello World ROM

This package generates a small emulator-only Game Boy ROM for exercising
Purremu. It uses only original glyph data and intentionally omits the official
boot logo and a hardware-valid cartridge header.

The ROM performs two independent demonstrations:

- prints `Hello, World!` through the serial registers;
- uploads original `HELLO WORLD` tiles to `$8000` and places their tile IDs in
  the background map at `$9800`.

It uses a fixed background configuration: no scrolling, alternate VRAM address
selection, window, objects, DMA, or interrupts.

Generate the ROM from the workspace root:

```sh
cargo run -p purremu-gb-hello-world-rom
```

The generated `target/hello-world.gb` file is a build artifact and must not be
committed. Run it through the emulator's post-boot entry path:

```sh
cargo run -p purremu-gb-cli -- target/hello-world.gb
```

The memory layout and LCD setup follow the
[Pan Docs memory map](https://gbdev.io/pandocs/Memory_Map.html),
[tile data format](https://gbdev.io/pandocs/Tile_Data.html), and
[LCD control register](https://gbdev.io/pandocs/LCDC.html).
