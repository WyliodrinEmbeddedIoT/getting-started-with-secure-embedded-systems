use core::cell::Cell;
use core::mem;
use kernel::grant::Grant;
use kernel::hil::led::Led;
use kernel::hil::time::{Alarm, AlarmClient, ConvertTicks};
use kernel::process::{Error, ProcessId};
use kernel::processbuffer::{ReadOnlyProcessBuffer, ReadableProcessBuffer};
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::utilities::cells::OptionalCell;
use kernel::ErrorCode;

/// The driver number 
///
/// As this is not one of Tock's standard drivers,
/// its number has to be higher or equal to 0xa0000.
/// 
/// Our previous driver was 0xa0001 so we use the
/// number available.
pub const DRIVER_NUM: usize = 0xa0002;

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

/// The data type that will be stored in each
/// process' grant.
#[derive(Default)]
pub struct AppData {
    /// The buffer shared by the process with the driver
    //// that contains the text that thr driver should display.
    buffer: ReadOnlyProcessBuffer,

    /// The position within the buffer that the driver will
    // display next
    position: usize,

    /// The length of the usefull data stored in the buffer
    len: usize,

    /// The number of milliseconds that each digit or letter
    /// will be displayed
    delay_ms: usize,
}

/// Structure representing the driver
pub struct TextDisplay<'a, L: Led, A: Alarm<'a>> {
    /// the a slice of Matrix LEDs 
    /// LED 0 is upper left, LED 24 is lower right
    leds: &'a [&'a L],

    /// The alarm used to implement the asynchronous deplay
    alarm: &'a A,

    /// The grant entrypoint
    ///
    /// The data type stored by the grant is `AppData` and
    /// it can register up to 1 upcall.
    grant: Grant<AppData, 1>,

    /// Stores whether the driver is in the middle of displaying a text
    in_progress: Cell<bool>,

    /// The ProcessId of the process for which the driver is currently
    /// displaying a text
    process_id: OptionalCell<ProcessId>,
}

impl<'a, L: Led, A: Alarm<'a>> TextDisplay<'a, L, A> {
    /// Initializes a new driver structure 
    pub fn new(leds: &'a [&'a L], alarm: &'a A, grant: Grant<AppData, 1>) -> Self {
        if leds.len() != 25 {
            panic!("Expecting 25 LEDs, {} supplied", leds.len());
        }
        TextDisplay {
            leds: leds,
            alarm: alarm,
            grant: grant,
            in_progress: Cell::new(false),
            process_id: OptionalCell::empty(),
        }
    }

    /// Displays the next letter or digit from the process' buffer
    fn display_next(&self) {
        // Verify if there is a display in progress. 
        // If not, calling this function was an error,
        /// so just ignore it.
        if self.in_progress.get() {
            // Verify if the process that has requested the display is
            // still valid.
            self.process_id.map_or_else(
                || {
                    // The process is not valid anymore, it has been stopped or restarted,
                    // the grant are where the driver stores all the data about the current 
                    // display is not valid anymore. We consider that the display
                    // cannot continue and mark that we are free to take another
                    // display request.
                    self.in_progress.set(false);
                    // panic!("Display in progress with no process id");
                },
                |process_id| {
                    // The process is still valid, so we try to enter its grant area.
                    let res = self.grant.enter(*process_id, |app, upcalls| {
                        // Verify if there are still letters or digites to display
                        if app.position < app.len {
                            // Access the buffer shared by the process and display the next
                            // letter or digit.
                            let res = app
                                .buffer
                                .enter(|buffer| {
                                    // Call the display function to set the LEDs.
                                    let _ = self.display(buffer[app.position].get() as char);
                                    // Set up an alarm after the specified milliseconds.
                                    self.alarm.set_alarm(
                                        self.alarm.now(),
                                        self.alarm.ticks_from_ms(app.delay_ms as u32),
                                    );
                                    // Return success
                                    // This will set res = true
                                    true
                                })
                                // If we cannot access the buffer, return false
                                // This will set res = false
                                .unwrap_or(false);
                            if res {
                                // We successfully displayed a letter or a digit,
                                // so we increase the current position
                                app.position = app.position + 1;
                            } else {
                                // There was an error when we tried to display 
                                // a letter or a digit, we we cannot continue
                                // the current action.
                                self.in_progress.set(false);
                                // Inform the process that the display has failed, 
                                // due to a buffer access error.
                                let _ = upcalls.schedule_upcall(0, (ErrorCode::NOMEM.into(), 0, 0));
                            }
                        } else {
                            // We have displayed all the letters and digits from the
                            // buffer, we are done.
                            self.in_progress.set(false);
                            // Inform the process that the display is done.
                            let _ = upcalls.schedule_upcall(0, (0, 0, 0));
                        }
                    });
                    match res {
                        // We entered the grant are and displayed the next letter or digit.
                        Ok(()) => {}
                        // We where not abler to enter the grant are, so we consider that we
                        // cannot continue the current display action and are ready to 
                        // take a new one.
                        // We cannot infom the process about the failure as the process
                        // that has requested the action is not valid anymore.
                        Err(_) => self.in_progress.set(false),
                    }
                },
            );
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

/// The implementation of `SyscallDriver` makes `TextDisplay` a syscall driver
impl<'a, L: Led, A: Alarm<'a>> SyscallDriver for TextDisplay<'a, L, A> {
    fn allow_readonly(
        &self,
        process_id: ProcessId,
        allow_number: usize,
        mut buffer: ReadOnlyProcessBuffer,
    ) -> Result<ReadOnlyProcessBuffer, (ReadOnlyProcessBuffer, ErrorCode)> {
        match allow_number {
            // The process has shared or unshared (if buffer is indirectly None) a buffer with us
            0 => {
                // Enter the grant and try to swap the previous buffer
                // with the one that we have just recevied.
                let res = self.grant.enter(process_id, |app, _| {
                    // buffer will become app.buffer
                    // app.buffer will become buffer
                    mem::swap(&mut app.buffer, &mut buffer);
                    // reset the positions as this is a new buffer
                    app.len = 0;
                    app.position = 0;
                    app.delay_ms = 0;
                });
                match res {
                    // We have registered the new buffer
                    // and we return the old one.
                    // The actual value of buffer was swapped (mem::swap) whe
                    // we registered the new buffer, so the buffer
                    // argument now stores the old buffer
                    Ok(()) => Ok(buffer),
                    // We did not register the buffer, we return an
                    // error and the new buffer.
                    Err(err) => Err((buffer, err.into())),
                }
            }
            // We only know what to do with buffer number 0,
            // so we return an error is a process tries to
            // share with us a buffer with another number.
            _ => Err((buffer, ErrorCode::NOSUPPORT)),
        }
    }

    fn allocate_grant(&self, process_id: ProcessId) -> Result<(), Error> {
        // The kernel asked us to allocate the grant, all we have to
        // do is to try to enter it. The kernel will do the task of
        // allocating it for us.
        //
        // The kernel requires us to call the *enter* ourselves as we are
        // the only ones that know the exact size of our grant data. By calling
        // this function, the kernel recevies the actual data type of grant.
        self.grant.enter(process_id, |_, _| {})
    }

    fn command(
        &self,
        command_number: usize,
        r2: usize,
        r3: usize,
        process_id: ProcessId,
    ) -> CommandReturn {
        match command_number {
            // Tock's convention states that all syscall drivers must return *success* or *success_...* for
            // command number 0. This allows processes to verify if a driver is present.
            0 => CommandReturn::success(),
            // Display the text from the buffer
            //  r2 - is the length of the text
            //  r3 - is the time in milliseconds that a letter or digit is displayed
            1 => {
                // Verify if there is another display action in progress.
                if !self.in_progress.get() {
                    // If there is no action in progress,
                    // we can start.
                    // We enter the process' grant data to set the
                    // parameters.
                    let res = self.grant.enter(process_id, |app, _| {
                        // Verify is the process has previously shared a buffer.
                        if app.buffer.len() > 0 {
                            // Verify that the length that the process is requesting us to
                            // display is less or equal to the capacity of the buffer.
                            if app.buffer.len() >= r2 {
                                // Reset the parameters
                                app.position = 0;
                                app.len = r2;
                                app.delay_ms = r3;
                                // We can start displaying.
                                // res = Ok(())
                                Ok(())
                            } else {
                                // The buffer is to small.
                                // res = Err(ErrorCode::SIZE)
                                Err(ErrorCode::SIZE)
                            }
                        } else {
                            // The process has not shared with us a buffer.
                            // res = Err(ErrorCode::NOMEM)
                            Err(ErrorCode::NOMEM)
                        }
                    });
                    match res {
                        // If we can start displaying
                        Ok(Ok(())) => {
                            // Store the ProcessId if the requesting process
                            self.process_id.set(process_id);
                            // Set that we have a display in progress
                            self.in_progress.set(true);
                            // Display the next digit or letter
                            self.display_next();
                            // Inform the process that we have started the displaying
                            // of the text
                            CommandReturn::success()
                        }
                        // The buffer is probably too small, inform the process that we cannot display.
                        Ok(Err(err)) => CommandReturn::failure(err),
                        // There is no shared buffer, inform the process that we cannot display.
                        Err(err) => CommandReturn::failure(err.into()),
                    }
                } else {
                    // If another display action is in progress,
                    // inform the process that we are busy and
                    // that it should try again later.
                    CommandReturn::failure(ErrorCode::BUSY)
                }
            }
            // Inform the process that we do not understand the command
            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }
}

/// This implementation allows `TextDisplay` to use an alarm.
impl<'a, L: Led, A: Alarm<'a>> AlarmClient for TextDisplay<'a, L, A> {
    /// Called when the alarm expires
    fn alarm(&self) {
        // The alarm has expired, the current letter or digit has been displayed enugh,
        // display the next letter or digit
        self.display_next();
    }
}
