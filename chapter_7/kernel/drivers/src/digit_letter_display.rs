use kernel::hil::led::Led;
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::ErrorCode;
use kernel::process::{Error, ProcessId};

pub const DRIVER_NUM: usize = 0xa0001;

const DIGITS: [u32; 10] = [
    // 0
    0b11111_10011_10101_11001_11111,
    // 1
    0b00100_01100_00100_00100_01110,
    // 2
    0b11110_00001_01110_10000_11111,
    // 3
    0b11110_00001_11110_00001_11110,
    // 4
    0b10000_10000_10100_11111_00100,
    // 5
    0b11111_10000_11110_00001_11110,
    // 6
    0b11111_10000_11111_10001_11111,
    // 7
    0b11111_00001_00010_00100_00100,
    // 8
    0b11111_10001_11111_10001_11111,
    // 9
    0b11111_10001_11111_00001_11111,
];

const LETTERS: [u32; 26] = [
    // A
    0b01110_10001_11111_10001_10001,
    // B
    0b11111_10001_11110_10001_11111,
    // C
    0b11111_10000_10000_10000_11111,
    // D
    0b11110_10001_10001_10001_11110,
    // E
    0b11111_10000_11110_10000_11111,
    // F
    0b11111_10000_11110_10000_10000,
    // G
    0b11111_10000_10111_10001_11111,
    // H
    0b10001_10001_11111_10001_10001,
    // I
    0b11111_00100_00100_00100_11111,
    // J
    0b00011_00001_00001_10001_11111,
    // K
    0b10001_10010_11100_10010_10001,
    // L
    0b10000_10000_10000_10000_11111,
    // M
    0b10001_11011_10101_10001_10001,
    // N
    0b10001_11001_10101_10011_10001,
    // O
    0b01110_10001_10001_10001_01110,
    // P
    0b11110_10001_11110_10000_10000,
    // Q
    0b01110_10001_10001_01110_00011,
    // R
    0b11110_10001_11110_10001_10001,
    // S
    0b11111_10000_11111_00001_11111,
    // T
    0b11111_00100_00100_00100_00100,
    // U
    0b10001_10001_10001_10001_11111,
    // V
    0b10001_10001_01010_01010_00100,
    // W
    0b10001_10001_10101_10101_01010,
    // X
    0b10001_01010_00100_01010_10001,
    // Y
    0b10001_10001_01010_00100_00100,
    // Z
    0b11111_00010_00100_01000_11111,
];

pub struct DigitLetterDisplay<'a, L: Led> {
    leds: &'a [&'a L],
}

impl<'a, L: Led> DigitLetterDisplay<'a, L> {
    pub fn new(leds: &'a [&'a L]) -> Self {
        if leds.len() != 25 {
            panic!("Expecting 25 LEDs, {} supplied", leds.len());
        }
        DigitLetterDisplay { leds: leds }
    }

    fn print(&self, glyph: u32) {
        for index in 0..25 {
            match (glyph >> (24 - index)) & 0x01 {
                0 => self.leds[index].off(),
                _ => self.leds[index].on(),
            }
        }
    }

    fn clear(&self) {
        for index in 0..25 {
            self.leds[index].off();
        }
    }

    fn display(&self, character: char) -> Result<(), ErrorCode> {
        let displayed_character = character.to_ascii_uppercase();
        match displayed_character {
            '0'..='9' => {
                self.print(DIGITS[displayed_character as usize - '0' as usize]);
                Ok(())
            }
            'A'..='Z' => {
                self.print(LETTERS[displayed_character as usize - 'A' as usize]);
                Ok(())
            }
            _ => {
                self.clear();
                Err(ErrorCode::INVAL)
            }
        }
    }
}

impl<'a, L: Led> SyscallDriver for DigitLetterDisplay<'a, L> {

    fn allocate_grant (&self, _process_id: ProcessId) -> Result<(), Error> {
        Ok(())
    }

    fn command(
        &self,
        command_number: usize,
        r2: usize,
        _r3: usize,
        _process_id: ProcessId,
    ) -> CommandReturn {
        match command_number {
            0 => CommandReturn::success(),
            1 => match self.display(r2 as u8 as char) {
                Ok(()) => CommandReturn::success(),
                Err(err) => CommandReturn::failure(err),
            },
            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }
}
