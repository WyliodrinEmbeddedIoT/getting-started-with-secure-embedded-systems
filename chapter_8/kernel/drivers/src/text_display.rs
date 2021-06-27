#![forbid(unsafe_code)]
use core::cell::Cell;
use core::mem;
use kernel::common::cells::OptionalCell;
use kernel::hil::led::Led;
use kernel::hil::time::{Alarm, AlarmClient};
use kernel::{CommandReturn, Driver, ErrorCode, Grant, ProcessId, Read, ReadOnlyAppSlice, Upcall};

pub const DRIVER_NUM: usize = 0xa0002;

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

#[derive(Default)]
pub struct AppData {
    upcall: Upcall,
    buffer: ReadOnlyAppSlice,
    position: usize,
    len: usize,
    delay_ms: usize,
}

pub struct TextDisplay<'a, L: Led, A: Alarm<'a>> {
    leds: &'a [&'a L],
    alarm: &'a A,
    grant: Grant<AppData>,
    in_progress: Cell<bool>,
    process_id: OptionalCell<ProcessId>,
}

impl<'a, L: Led, A: Alarm<'a>> TextDisplay<'a, L, A> {
    pub fn new(leds: &'a [&'a L], alarm: &'a A, grant: Grant<AppData>) -> Self {
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

    fn display_next(&self) {
        if self.in_progress.get() {
            self.process_id.map_or_else(
                || {
                    self.in_progress.set(false);
                    // panic!("Display in progress with no process id");
                },
                |process_id| {
                    let res = self.grant.enter(*process_id, |app| {
                        if app.position < app.len {
                            let res = app.buffer.map_or(false, |buffer| {
                                let _ = self.display(buffer[app.position] as char);
                                self.alarm.set_alarm(
                                    self.alarm.now(),
                                    A::ticks_from_ms(app.delay_ms as u32),
                                );
                                true
                            });
                            if res {
                                app.position = app.position + 1;
                            } else {
                                self.in_progress.set(false);
                                app.upcall.schedule(ErrorCode::NOMEM.into(), 0, 0);
                            }
                        } else {
                            self.in_progress.set(false);
                            app.upcall.schedule(0, 0, 0);
                        }
                    });
                    match res {
                        Ok(()) => {}
                        Err(_) => self.in_progress.set(false),
                    }
                },
            );
        }
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

impl<'a, L: Led, A: Alarm<'a>> Driver for TextDisplay<'a, L, A> {
    fn allow_readonly(
        &self,
        process_id: ProcessId,
        allow_number: usize,
        mut buffer: ReadOnlyAppSlice,
    ) -> Result<ReadOnlyAppSlice, (ReadOnlyAppSlice, ErrorCode)> {
        match allow_number {
            0 => {
                let res = self.grant.enter(process_id, |app| {
                    mem::swap(&mut app.buffer, &mut buffer);
                    app.len = 0;
                    app.position = 0;
                    app.delay_ms = 0;
                });
                match res {
                    Ok(()) => Ok(buffer),
                    Err(err) => Err((buffer, err.into())),
                }
            }
            _ => Err((buffer, ErrorCode::NOSUPPORT)),
        }
    }

    fn subscribe(
        &self,
        subscribe_number: usize,
        mut upcall: Upcall,
        process_id: ProcessId,
    ) -> Result<Upcall, (Upcall, ErrorCode)> {
        match subscribe_number {
            0 => {
                let res = self.grant.enter(process_id, |app| {
                    mem::swap(&mut app.upcall, &mut upcall);
                });
                match res {
                    Ok(()) => Ok(upcall),
                    Err(err) => Err((upcall, err.into())),
                }
            }
            _ => Err((upcall, ErrorCode::NOSUPPORT)),
        }
    }

    fn command(
        &self,
        command_number: usize,
        r2: usize,
        r3: usize,
        process_id: ProcessId,
    ) -> CommandReturn {
        match command_number {
            0 => CommandReturn::success(),
            1 => {
                if !self.in_progress.get() {
                    let res = self.grant.enter(process_id, |app| {
                        if app.buffer.len() > 0 {
                            if app.buffer.len() >= r2 {
                                app.position = 0;
                                app.len = r2;
                                app.delay_ms = r3;
                                Ok(())
                            } else {
                                Err(ErrorCode::SIZE)
                            }
                        } else {
                            Err(ErrorCode::NOMEM)
                        }
                    });
                    match res {
                        Ok(Ok(())) => {
                            self.process_id.set(process_id);
                            self.in_progress.set(true);
                            self.display_next();
                            CommandReturn::success()
                        }
                        Ok(Err(err)) => CommandReturn::failure(err),
                        Err(err) => CommandReturn::failure(err.into()),
                    }
                } else {
                    CommandReturn::failure(ErrorCode::BUSY)
                }
            }
            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }
}

/* ... */

impl<'a, L: Led, A: Alarm<'a>> AlarmClient for TextDisplay<'a, L, A> {
    fn alarm(&self) {
        self.display_next();
    }
}
