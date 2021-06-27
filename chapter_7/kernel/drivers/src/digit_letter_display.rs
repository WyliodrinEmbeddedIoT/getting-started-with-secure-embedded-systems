#![forbid(unsafe_code)]
use kernel::hil::led::Led;
use kernel::{CommandReturn, Driver, ErrorCode, ProcessId};

pub const DRIVER_NUM: usize = 0xa0001;

const DIGITS: [u32; 10] = [
    // 0
    0b11111_10011_10101_11001_11111,
    // 1
    0b00100_01100_00100_00100_01110,
    // 2
    0b11110_00001_01110_10000_11111,
    // 3
    0b11110_00001_11110_00010_11111,
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

const LETTERS: [u32; 1] = [
    // A
    0b01110_10001_11111_10001_10001,
    // ...
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
            match glyph >> (25 - index) {
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

impl<'a, L: Led> Driver for DigitLetterDisplay<'a, L> {
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
