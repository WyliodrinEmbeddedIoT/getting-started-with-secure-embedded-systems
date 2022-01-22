use kernel::hil::led::Led;
use kernel::process::{Error, ProcessId};
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::ErrorCode;

/// The driver number
///
/// As this is not one of Tock's standard drivers,
/// its number has to be higher or equal to 0xa0000.
pub const DRIVER_NUM: usize = 0xa0001;

/// Font glyph definition for digits
///
/// A font glyph is a set of bits that represents that
/// state of the LEDs
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

/// Font glyph definition for capital letters
///
/// A font glyph is a set of bits that represents that
/// state of the LEDs
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

/// Structure representing the driver
pub struct DigitLetterDisplay<'a, L: Led> {
    /// The a slice of Matrix LEDs
    /// LED 0 is upper left, LED 24 is lower right
    leds: &'a [&'a L; 25],
}

impl<'a, L: Led> DigitLetterDisplay<'a, L> {
    /// Initializes a new driver structure
    pub fn new(leds: &'a [&'a L; 25]) -> Self {
        DigitLetterDisplay { leds }
    }

    /// Prints the a font `glyph` by setting LEDs
    /// on and off depending on the glyph's bits
    ///
    /// A font glyph is a set of bits that represents that
    /// state of the LEDs
    fn print(&self, glyph: u32) {
        for index in 0..25 {
            match (glyph >> (24 - index)) & 0x01 {
                0 => self.leds[index].off(),
                _ => self.leds[index].on(),
            }
        }
    }

    /// Clears the displayed glyph by turning off
    /// all the LEDs
    fn clear(&self) {
        for index in 0..25 {
            self.leds[index].off();
        }
    }

    /// Displays a character
    fn display(&self, character: char) -> Result<(), ErrorCode> {
        // As the font has only capital letters, we make sure
        // that we only ask it to display uppercase letters
        let displayed_character = character.to_ascii_uppercase();
        match displayed_character {
            // display a number
            '0'..='9' => {
                self.print(DIGITS[displayed_character as usize - '0' as usize]);
                Ok(())
            }
            // display a letter
            'A'..='Z' => {
                self.print(LETTERS[displayed_character as usize - 'A' as usize]);
                Ok(())
            }
            // we don't know how to display this character,
            // so we display an *empty* character and
            // return an error
            _ => {
                self.clear();
                Err(ErrorCode::INVAL)
            }
        }
    }
}

/// The implementation of `SyscallDriver` makes `DigitLetterDisplay` a syscall driver
impl<'a, L: Led> SyscallDriver for DigitLetterDisplay<'a, L> {
    fn allocate_grant(&self, _process_id: ProcessId) -> Result<(), Error> {
        // there is no grant used by this driver, we just ignore
        // the function call and return success
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
            // Tock's convention states that all syscall drivers must return *success* or *success_...* for
            // command number 0. This allows processes to verify if a driver is present.
            0 => CommandReturn::success(),
            // Display the character received in *r2*
            // We cannot directly convert a *usize* to *char* as not all numbers are valid
            // UTF8 code points. As our driver only displays digits and letters from the ASCII
            // code, we can safely converet the *usize* to an *u8* as all ASCII characters fit
            // into a one byte (*u8*). As all ASCII characters are valid UTF-8 code points,
            // Rust allows us to safely converty an *u8* to a *char*.
            1 => match self.display(r2 as u8 as char) {
                Ok(()) => CommandReturn::success(),
                Err(err) => CommandReturn::failure(err),
            },
            // Inform the process that we do not understand the command
            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    /* the default implementation of the *allow_...* functions is used */
}
