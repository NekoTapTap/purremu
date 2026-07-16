use std::collections::HashMap;

const ROM_SIZE: usize = 0x8000;
const ENTRY_POINT: usize = 0x0100;
const PROGRAM_START: usize = 0x0150;
const TILE_DATA_DESTINATION: u16 = 0x8000;
const TILE_MAP_START: u16 = 0x9800;
const TEXT_DESTINATION: u16 = TILE_MAP_START + 8 * 32 + 4;

const BACKGROUND_TEXT: &[u8] = &[1, 2, 3, 3, 4, 0, 5, 4, 6, 3, 7];
const SERIAL_TEXT: &[u8] = b"Hello, World!\n\0";

// Original 5x7 glyphs centered in 8x8 tiles. Each set bit is emitted into
// both DMG bitplanes, selecting color index 3.
const GLYPHS: [[u8; 8]; 7] = [
    [0x42, 0x42, 0x42, 0x7e, 0x42, 0x42, 0x42, 0x00], // H
    [0x7e, 0x40, 0x40, 0x7c, 0x40, 0x40, 0x7e, 0x00], // E
    [0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x7e, 0x00], // L
    [0x3c, 0x42, 0x42, 0x42, 0x42, 0x42, 0x3c, 0x00], // O
    [0x42, 0x42, 0x42, 0x42, 0x5a, 0x66, 0x42, 0x00], // W
    [0x7c, 0x42, 0x42, 0x7c, 0x48, 0x44, 0x42, 0x00], // R
    [0x78, 0x44, 0x42, 0x42, 0x42, 0x44, 0x78, 0x00], // D
];

#[derive(Clone, Copy)]
enum FixupKind {
    Absolute16,
    Relative8,
}

struct Fixup {
    offset: usize,
    label: &'static str,
    kind: FixupKind,
}

struct Assembler {
    rom: Vec<u8>,
    position: usize,
    labels: HashMap<&'static str, usize>,
    fixups: Vec<Fixup>,
}

impl Assembler {
    fn new() -> Self {
        Self {
            rom: vec![0; ROM_SIZE],
            position: 0,
            labels: HashMap::new(),
            fixups: Vec::new(),
        }
    }

    fn seek(&mut self, position: usize) {
        assert!(position < self.rom.len(), "ROM position is out of bounds");
        self.position = position;
    }

    fn label(&mut self, label: &'static str) {
        assert!(
            self.labels.insert(label, self.position).is_none(),
            "duplicate label: {label}"
        );
    }

    fn emit(&mut self, bytes: &[u8]) {
        let end = self.position + bytes.len();
        assert!(end <= self.rom.len(), "ROM program exceeds 32 KiB");
        self.rom[self.position..end].copy_from_slice(bytes);
        self.position = end;
    }

    fn emit_absolute16(&mut self, opcode: u8, label: &'static str) {
        self.emit(&[opcode]);
        let offset = self.position;
        self.emit(&[0, 0]);
        self.fixups.push(Fixup {
            offset,
            label,
            kind: FixupKind::Absolute16,
        });
    }

    fn emit_relative8(&mut self, opcode: u8, label: &'static str) {
        self.emit(&[opcode]);
        let offset = self.position;
        self.emit(&[0]);
        self.fixups.push(Fixup {
            offset,
            label,
            kind: FixupKind::Relative8,
        });
    }

    fn finish(mut self) -> Vec<u8> {
        for fixup in self.fixups {
            let target = *self
                .labels
                .get(fixup.label)
                .unwrap_or_else(|| panic!("unknown label: {}", fixup.label));

            match fixup.kind {
                FixupKind::Absolute16 => {
                    let target = u16::try_from(target).expect("absolute address exceeds 16 bits");
                    self.rom[fixup.offset..fixup.offset + 2].copy_from_slice(&target.to_le_bytes());
                }
                FixupKind::Relative8 => {
                    let instruction_end = fixup.offset + 1;
                    let displacement = target as isize - instruction_end as isize;
                    let displacement = i8::try_from(displacement).unwrap_or_else(|_| {
                        panic!("relative jump to {} is out of range", fixup.label)
                    });
                    self.rom[fixup.offset] = displacement as u8;
                }
            }
        }

        self.rom
    }
}

pub fn build_rom() -> Vec<u8> {
    let mut assembler = Assembler::new();

    assembler.seek(ENTRY_POINT);
    assembler.emit_absolute16(0xc3, "main"); // JP main

    assembler.seek(PROGRAM_START);
    assembler.label("main");
    assembler.emit(&[0xf3]); // DI
    assembler.emit(&[0x31, 0xfe, 0xff]); // LD SP, $FFFE
    assembler.emit(&[0xaf]); // XOR A
    assembler.emit(&[0xe0, 0x40]); // LDH [$FF40], A; disable LCD

    assembler.emit_absolute16(0x21, "serial_text"); // LD HL, serial_text
    assembler.label("serial_next");
    assembler.emit(&[0x2a]); // LD A, [HL+]
    assembler.emit(&[0xfe, 0x00]); // CP 0
    assembler.emit_relative8(0x28, "after_serial"); // JR Z, after_serial
    assembler.emit(&[0xe0, 0x01]); // LDH [$FF01], A; SB
    assembler.emit(&[0x3e, 0x81]); // LD A, $81
    assembler.emit(&[0xe0, 0x02]); // LDH [$FF02], A; SC
    assembler.label("serial_wait");
    assembler.emit(&[0xf0, 0x02]); // LDH A, [$FF02]
    assembler.emit(&[0xe6, 0x80]); // AND $80
    assembler.emit_relative8(0x20, "serial_wait"); // JR NZ, serial_wait
    assembler.emit_relative8(0x18, "serial_next"); // JR serial_next
    assembler.label("after_serial");

    assembler.emit_absolute16(0x11, "tile_data"); // LD DE, tile_data
    assembler.emit(&[0x21]); // LD HL, $8000
    assembler.emit(&TILE_DATA_DESTINATION.to_le_bytes());
    assembler.emit(&[0x01]); // LD BC, tile_data_end - tile_data
    assembler.emit(&(u16::try_from(tile_data().len()).unwrap()).to_le_bytes());
    assembler.label("copy_tile_data");
    assembler.emit(&[0x1a]); // LD A, [DE]
    assembler.emit(&[0x13]); // INC DE
    assembler.emit(&[0x22]); // LD [HL+], A
    assembler.emit(&[0x0b]); // DEC BC
    assembler.emit(&[0x78]); // LD A, B
    assembler.emit(&[0xb1]); // OR C
    assembler.emit_relative8(0x20, "copy_tile_data"); // JR NZ, copy_tile_data

    assembler.emit(&[0x21]); // LD HL, $9800
    assembler.emit(&TILE_MAP_START.to_le_bytes());
    assembler.emit(&[0x01, 0x00, 0x04]); // LD BC, $0400
    assembler.label("clear_tile_map");
    assembler.emit(&[0x36, 0x00]); // LD [HL], 0
    assembler.emit(&[0x23]); // INC HL
    assembler.emit(&[0x0b]); // DEC BC
    assembler.emit(&[0x78]); // LD A, B
    assembler.emit(&[0xb1]); // OR C
    assembler.emit_relative8(0x20, "clear_tile_map"); // JR NZ, clear_tile_map

    assembler.emit_absolute16(0x11, "background_text"); // LD DE, background_text
    assembler.emit(&[0x21]); // LD HL, text position in BG map
    assembler.emit(&TEXT_DESTINATION.to_le_bytes());
    assembler.emit(&[0x01]); // LD BC, background text length
    assembler.emit(&(u16::try_from(BACKGROUND_TEXT.len()).unwrap()).to_le_bytes());
    assembler.label("copy_background_text");
    assembler.emit(&[0x1a]); // LD A, [DE]
    assembler.emit(&[0x13]); // INC DE
    assembler.emit(&[0x22]); // LD [HL+], A
    assembler.emit(&[0x0b]); // DEC BC
    assembler.emit(&[0x78]); // LD A, B
    assembler.emit(&[0xb1]); // OR C
    assembler.emit_relative8(0x20, "copy_background_text"); // JR NZ, copy_background_text

    assembler.emit(&[0x3e, 0xe4]); // LD A, $E4; identity DMG palette
    assembler.emit(&[0xe0, 0x47]); // LDH [$FF47], A; BGP
    assembler.emit(&[0x3e, 0x91]); // LD A, $91; LCD on, $8000 tiles, $9800 BG
    assembler.emit(&[0xe0, 0x40]); // LDH [$FF40], A; LCDC

    assembler.label("done");
    assembler.emit_relative8(0x18, "done"); // JR done

    assembler.label("tile_data");
    assembler.emit(&tile_data());
    assembler.label("background_text");
    assembler.emit(BACKGROUND_TEXT);
    assembler.label("serial_text");
    assembler.emit(SERIAL_TEXT);

    assembler.finish()
}

fn tile_data() -> Vec<u8> {
    let mut tiles = Vec::with_capacity((GLYPHS.len() + 1) * 16);
    tiles.extend_from_slice(&[0; 16]);

    for glyph in GLYPHS {
        for row in glyph {
            tiles.extend_from_slice(&[row, row]);
        }
    }

    tiles
}

#[cfg(test)]
mod tests {
    use purremu_gb_core::{Event, GameBoy, Joypad};

    use super::{ENTRY_POINT, PROGRAM_START, ROM_SIZE, build_rom};

    #[test]
    fn builds_a_32_kib_emulator_only_rom_without_an_official_logo() {
        let rom = build_rom();

        assert_eq!(rom.len(), ROM_SIZE);
        assert_eq!(rom[ENTRY_POINT], 0xc3);
        assert_eq!(
            u16::from_le_bytes([rom[ENTRY_POINT + 1], rom[ENTRY_POINT + 2]]),
            PROGRAM_START as u16
        );
        assert!(rom[0x0104..0x0134].iter().all(|byte| *byte == 0));
    }

    #[test]
    fn prints_hello_world_over_the_serial_device() {
        let mut gameboy = GameBoy::new_post_boot(build_rom());
        let joypad = Joypad::new();
        let mut output = Vec::new();

        for _ in 0..300_000 {
            for event in gameboy.step(&joypad) {
                if let Event::SerialByte(byte) = event {
                    output.push(byte);
                }
            }

            if output.ends_with(b"Hello, World!\n") {
                break;
            }
        }

        assert_eq!(output, b"Hello, World!\n");
    }
}
