use core::cell::Cell;
use core::cmp;
use kernel::dynamic_deferred_call::{
    DeferredCallHandle, DynamicDeferredCall, DynamicDeferredCallClient,
};
use kernel::hil::led::Led;
use kernel::hil::text_screen::{TextScreen, TextScreenClient};
use kernel::hil::time::{Alarm, AlarmClient, ConvertTicks};
use kernel::process::{Error, ProcessId};
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::ErrorCode;

/// The driver number
///
/// As this is not one of Tock's standard drivers,
/// its number has to be higher or equal to 0xa0000.
///
/// Our previous driver was 0xa0002 so we use the
/// number available.
pub const DRIVER_NUM: usize = 0xa0003;

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

/// The possible states
#[derive(Copy, Clone, PartialEq)]
enum Status {
    /// The driver can accept requests
    Idle,
    /// The driver executes a request
    ExecutesCommand,
    /// The driver executes the *print* request
    ExecutesPrint,
}

/// Structure representing the driver
pub struct LedMatrixText<'a, L: Led, A: Alarm<'a>> {
    /// the a slice of Matrix LEDs
    /// LED 0 is upper left, LED 24 is lower right
    leds: &'a [&'a L],

    /// The alarm used to implement the asynchronous deplay
    alarm: &'a A,

    /// An optional client (usually the caller) that the driver
    /// will notify when a request is done.
    client: OptionalCell<&'a dyn TextScreenClient>,

    /// The driver's buffer
    buffer: TakeCell<'a, [u8]>,

    /// The position within the driver's buffer that will
    /// be displayed next
    position: Cell<usize>,

    /// The length of the text stored in the driver's buffer
    len: Cell<usize>,

    /// A temporary buffer received by the driver from the client.
    /// The driver has to copy the data from this buffer to
    /// its own buffer and return this to the client.
    client_buffer: TakeCell<'static, [u8]>,

    /// The length of the text that the driver was able to copy to is
    /// own buffer.
    client_len: Cell<usize>,

    /// The speed at which the driver displays the text,
    /// expressed in milliseconds delay between to letters or digits.
    speed: Cell<u32>,

    /// The status of the driver.
    status: Cell<Status>,

    /// Stores if the driver is enabled or disabled.
    ///   - enabled means that it displays the text
    ///   - disabled means that it does not display that text
    is_enabled: Cell<bool>,

    /// A reference to the kernel's deferred caller used to schedule
    /// deferred callbacks (software interrupts)
    deferred_caller: &'a DynamicDeferredCall,

    /// The handle (position in the kernel's deferred callbacks array)
    /// to the driver's deferred callback function
    deferred_call_handle: OptionalCell<DeferredCallHandle>,
}

impl<'a, L: Led, A: Alarm<'a>> LedMatrixText<'a, L, A> {
    /// Initializes a new driver structure
    pub fn new(
        leds: &'a [&'a L],
        alarm: &'a A,
        buffer: &'a mut [u8],
        speed: u32,
        deferred_caller: &'a DynamicDeferredCall,
    ) -> Self {
        if leds.len() != 25 {
            panic!("Expecting 25 LEDs, {} supplied", leds.len());
        }
        LedMatrixText {
            leds: leds,
            alarm: alarm,
            buffer: TakeCell::new(buffer),
            client_buffer: TakeCell::empty(),
            client_len: Cell::new(0),
            position: Cell::new(0),
            speed: Cell::new(speed),
            len: Cell::new(0),
            status: Cell::new(Status::Idle),
            is_enabled: Cell::new(false),
            deferred_caller: deferred_caller,
            deferred_call_handle: OptionalCell::empty(),
            client: OptionalCell::empty(),
        }
    }

    /// Set the driver's deferred callback function
    pub fn initialize_callback_handle(&self, deferred_call_handle: DeferredCallHandle) {
        self.deferred_call_handle.replace(deferred_call_handle);
    }

    /// schedule a deferred callback (sfotware interrupt)
    fn schedule_deferred_callback(&self) {
        self.deferred_call_handle
            .map(|handle| self.deferred_caller.set(*handle));
    }

    /// Displays the next letter or digit from the driver's buffer
    fn display_next(&self) {
        // Verify if we are at the end of the buffer.
        if self.position.get() >= self.len.get() {
            // Reset the position to the start of the buffer.
            self.position.set(0);
        }
        // Verify if the current position is within the length
        // of the text. This can be false only if the length
        // of the text is 0.
        if self.position.get() < self.len.get() {
            if !self.buffer.map_or(false, |buffer| {
                // Make sure we are within the buffers length
                if self.position.get() < buffer.len() {
                    // Display the letter or digit.
                    let _ = self.display(buffer[self.position.get()] as char);
                    // We successfully displayed a letter or a digit,
                    // so we increase the current position
                    self.position.set(self.position.get() + 1);
                    true
                } else {
                    // We are overflowing the buffer
                    // This should never happen if our driver is correctly written.
                    false
                }
            }) {
                // We have no buffer, so we just clear what is displayed.
                // This should never happen as we do not send the driver's
                // buffer to any other driver.
                self.clear();
            }
        } else {
            // The length of the buffer is 0, so we have nothing to display.
            // Clear that is displayed right now.
            self.clear();
        }
        // If the length the text is greater then 0, set the next alarm.
        // If we have no letters or digits to display, the text's length
        // is 0, it is pointless to schedule an alarm as next
        // time the alarm fires there will still be no text to display.
        // Not setting the alarm allows the MCU to enter low power
        // modes (if there are no other taks pending).
        if self.len.get() > 0 {
            self.alarm
                .set_alarm(self.alarm.now(), self.alarm.ticks_from_ms(self.speed.get()));
        }
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
        if self.is_enabled.get() {
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
        } else {
            self.clear();
            Ok(())
        }
    }

    /// Returns the length of the driver's buffer
    fn get_buffer_len(&self) -> usize {
        self.buffer.map_or(0, |buffer| buffer.len())
    }
}

/// This implementation allows `LedMatrixText` to use an alarm.
impl<'a, L: Led, A: Alarm<'a>> AlarmClient for LedMatrixText<'a, L, A> {
    /// Called when the alarm expires
    fn alarm(&self) {
        // The alarm has expired, the current letter or digit has been displayed enugh,
        // display the next letter or digit
        self.display_next();
    }
}

/// This implementation allows `LedMatrixText` to receive deferred callbacks (software interrupts)
impl<'a, L: Led, A: Alarm<'a>> DynamicDeferredCallClient for LedMatrixText<'a, L, A> {
    /// The deferred callback (software interrupt) handler
    fn call(&self, _handle: DeferredCallHandle) {
        match self.status.get() {
            // We should not get here, we ignore it.
            Status::Idle => {}
            // The driver has performed a command, inform the client
            // that the action is done.
            Status::ExecutesCommand => {
                self.client.map(|client| client.command_complete(Ok(())));
            }
            // The driver has performed a *print* command, inform the client
            // that the action is done and return the buffer and the
            // written text length.
            Status::ExecutesPrint => {
                self.client.map(|client| {
                    self.client_buffer
                        .take()
                        .map(|buffer| client.write_complete(buffer, self.client_len.get(), Ok(())));
                });
            }
        }
        // The driver is ready to take new requests.
        self.status.set(Status::Idle);
    }
}

/// This implementation allows `LedMatrixText` to be used as a service driver to `TextSceen`.
impl<'a, L: Led, A: Alarm<'a>> TextScreen<'a> for LedMatrixText<'a, L, A> {
    fn set_client(&self, client: Option<&'a dyn TextScreenClient>) {
        if let Some(client) = client {
            self.client.set(client);
        } else {
            self.client.clear();
        }
    }

    fn get_size(&self) -> (usize, usize) {
        // Our simulated screen has 1 row that can display as many characters
        // as the capacity of the driver's buffer.
        (self.get_buffer_len(), 1)
    }

    /// This is a *print* request from the `TextScreen` driver
    fn print(
        &self,
        buffer: &'static mut [u8],
        len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        // Verify that we do no have another action in progress.
        if self.status.get() == Status::Idle {
            // Verify if the length of the usefull text does not overflow the received buffer.
            if len <= buffer.len() {
                // Start *print* action
                self.status.set(Status::ExecutesPrint);
                // Store the previous length of the text we store in the driver's buffer.
                let previous_len = self.len.get();
                // Copy the text to the driver's buffer.
                let printed_len = self.buffer.map_or(0, |buf| {
                    // Compute how many characters we can copy to the driver's buffer.
                    let max_len = cmp::min(len, buf.len());
                    for position in 0..max_len {
                        buf[position] = buffer[position];
                    }
                    // Compute the new length of the text sored in the driver's buffer
                    self.len.set(cmp::max(max_len, self.len.get()));
                    // Make printed_length = max_len, the number of characters that
                    // we have copied to thed driver's buffer.
                    max_len
                });
                // Store the received buffer in a field so that we can
                // return it to TextScreen from the deferred callback.
                self.client_buffer.replace(buffer);
                // Store the the number of copied characters into field so that
                // we can return it to TextScreen from the deferred callback.
                self.client_len.set(printed_len);
                // Ask the kernel to send us a deferred callback (software interrupt)
                // as we are not allowed to call TextScreen's *write_complete* function
                // before we return from the current function.
                self.schedule_deferred_callback();
                // If the previous length of the text was 0 the driver's
                // alarm is most probably disabled, so *display_next* will
                // not be automatically called. If the new length of the text
                // is different from 0, we can immedialty print the next
                // letter or digit.
                if previous_len == 0 && printed_len != 0 {
                    self.display_next();
                }
                Ok(())
            } else {
                // Inform the TextScreen that it sent us an invalid length
                // for the text it wants us to display.
                Err((ErrorCode::SIZE, buffer))
            }
        } else {
            // Inform the TextScreen that we have another action in progress
            // and that it should try again later.
            Err((ErrorCode::BUSY, buffer))
        }
    }

    /* Actions that are not supported */

    fn set_cursor(&self, _x_position: usize, _y_position: usize) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }

    fn hide_cursor(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }

    fn show_cursor(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }

    fn blink_cursor_on(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }

    fn blink_cursor_off(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }

    /* Display commands */

    fn display_on(&self) -> Result<(), ErrorCode> {
        // Verify that we do no have another action in progress.
        if self.status.get() == Status::Idle {
            // Start a new command action
            self.status.set(Status::ExecutesCommand);
            // Enable the display of the text
            self.is_enabled.set(true);
            // Ask the kernel to send us a deferred callback (software interrupt)
            // as we are not allowed to call TextScreen's *command_complete* function
            // before we return from the current function.
            self.schedule_deferred_callback();
            Ok(())
        } else {
            // Inform the TextScreen that we have another action in progress
            // and that it should try again later.
            Err(ErrorCode::BUSY)
        }
    }

    fn display_off(&self) -> Result<(), ErrorCode> {
        // Verify that we do no have another action in progress.
        if self.status.get() == Status::Idle {
            // Start a new command action
            self.status.set(Status::ExecutesCommand);
            // Disable the display of the text
            self.is_enabled.set(false);
            // Ask the kernel to send us a deferred callback (software interrupt)
            // as we are not allowed to call TextScreen's *command_complete* function
            // before we return from the current function.
            self.schedule_deferred_callback();
            Ok(())
        } else {
            // Inform the TextScreen that we have another action in progress
            // and that it should try again later.
            Err(ErrorCode::BUSY)
        }
    }

    fn clear(&self) -> Result<(), ErrorCode> {
        // Verify that we do no have another action in progress.
        if self.status.get() == Status::Idle {
            // Start a new command action
            self.status.set(Status::ExecutesCommand);
            // Reset the position
            self.position.set(0);
            // Set the text's length to 0
            self.len.set(0);
            // Clear what is currently displayed on the LED matrix
            self.clear();
            // Ask the kernel to send us a deferred callback (software interrupt)
            // as we are not allowed to call TextScreen's *command_complete* function
            // before we return from the current function.
            self.schedule_deferred_callback();
            Ok(())
        } else {
            // Inform the TextScreen that we have another action in progress
            // and that it should try again later.
            Err(ErrorCode::BUSY)
        }
    }
}

/// This implementation allows `LedMatrixText` to expose a setup syscall API
impl<'a, L: Led, A: Alarm<'a>> SyscallDriver for LedMatrixText<'a, L, A> {
    fn allocate_grant(&self, _: ProcessId) -> Result<(), Error> {
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
            // Set the speed at which letters and digits are displayed to the value stored in *r2*.
            1 => {
                self.speed.set(r2 as u32);
                CommandReturn::success()
            }
            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    /* the default implementation of the *allow_...* functions is used */
}
