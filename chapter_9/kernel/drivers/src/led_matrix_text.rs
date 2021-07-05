#![forbid(unsafe_code)]
use core::cell::Cell;
use core::cmp;
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::common::dynamic_deferred_call::{
    DeferredCallHandle, DynamicDeferredCall, DynamicDeferredCallClient,
};
use kernel::hil::led::Led;
use kernel::hil::text_screen::{TextScreen, TextScreenClient};
use kernel::hil::time::{Alarm, AlarmClient};
use kernel::ErrorCode;
use kernel::{CommandReturn, Driver, ProcessId};

use kernel::debug;

pub const DRIVER_NUM: usize = 0xa0003;

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

const LETTERS: [u32; 1] = [
    // A
    0b01110_10001_11111_10001_10001,
    // ...
];

#[derive(Copy, Clone, PartialEq)]
enum Status {
    Idle,
    ExecutesCommand,
    ExecutesPrint,
}

pub struct LedMatrixText<'a, L: Led, A: Alarm<'a>> {
    leds: &'a [&'a L],
    alarm: &'a A,
    buffer: TakeCell<'a, [u8]>,
    client_buffer: TakeCell<'static, [u8]>,
    client_len: Cell<usize>,
    position: Cell<usize>,
    len: Cell<usize>,
    speed: Cell<u32>,
    status: Cell<Status>,
    is_enabled: Cell<bool>,
    deferred_caller: &'a DynamicDeferredCall,
    deferred_call_handle: OptionalCell<DeferredCallHandle>,
    client: OptionalCell<&'a dyn TextScreenClient>,
}

impl<'a, L: Led, A: Alarm<'a>> LedMatrixText<'a, L, A> {
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

    pub fn initialize_callback_handle(&self, deferred_call_handle: DeferredCallHandle) {
        self.deferred_call_handle.replace(deferred_call_handle);
    }

    fn schedule_deferred_callback(&self) {
        self.deferred_call_handle
            .map(|handle| self.deferred_caller.set(*handle));
    }

    fn display_next(&self) {
        if self.position.get() >= self.len.get() {
            self.position.set(0);
        }
        debug!("display_next {} of {}", self.position.get(), self.len.get());
        if self.position.get() < self.len.get() {
            if !self.buffer.map_or(false, |buffer| {
                if self.position.get() < buffer.len() {
                    let _ = self.display(buffer[self.position.get()] as char);
                    self.position.set(self.position.get() + 1);
                    true
                } else {
                    false
                }
            }) {
                self.clear();
            }
        }
        if self.len.get() > 0 {
            self.alarm
                .set_alarm(self.alarm.now(), A::ticks_from_ms(self.speed.get()));
        }
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
        if self.is_enabled.get() {
            let displayed_character = character.to_ascii_uppercase();
            debug!("display {}", displayed_character);
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

    fn get_buffer_len(&self) -> usize {
        self.buffer.map_or(0, |buffer| buffer.len())
    }
}

impl<'a, L: Led, A: Alarm<'a>> AlarmClient for LedMatrixText<'a, L, A> {
    fn alarm(&self) {
        self.display_next();
    }
}

impl<'a, L: Led, A: Alarm<'a>> DynamicDeferredCallClient for LedMatrixText<'a, L, A> {
    fn call(&self, _handle: DeferredCallHandle) {
        match self.status.get() {
            Status::Idle => {}
            Status::ExecutesCommand => {
                self.client.map(|client| client.command_complete(Ok(())));
            }
            Status::ExecutesPrint => {
                self.client.map(|client| {
                    self.client_buffer
                        .take()
                        .map(|buffer| client.write_complete(buffer, self.client_len.get(), Ok(())));
                });
            }
        }
        self.status.set(Status::Idle);
    }
}

impl<'a, L: Led, A: Alarm<'a>> TextScreen<'a> for LedMatrixText<'a, L, A> {
    fn set_client(&self, client: Option<&'a dyn TextScreenClient>) {
        if let Some(client) = client {
            self.client.set(client);
        } else {
            self.client.clear();
        }
    }

    fn get_size(&self) -> (usize, usize) {
        (self.get_buffer_len(), 1)
    }

    fn print(
        &self,
        buffer: &'static mut [u8],
        len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        if self.status.get() == Status::Idle {
            if len <= buffer.len() {
                let previous_len = self.len.get();
                self.buffer.map(|buf| {
                    for position in 0..len {
                        buf[position] = buffer[position];
                    }
                    self.len.set(cmp::max(len, self.len.get()));
                });
                self.client_buffer.replace(buffer);
                self.client_len.set(len);
                self.status.set(Status::ExecutesPrint);
                self.schedule_deferred_callback();
                if previous_len == 0 {
                    self.display_next();
                }
                Ok(())
            } else {
                Err((ErrorCode::SIZE, buffer))
            }
        } else {
            Err((ErrorCode::BUSY, buffer))
        }
    }

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

    fn display_on(&self) -> Result<(), ErrorCode> {
        if self.status.get() == Status::Idle {
            self.is_enabled.set(true);
            self.status.set(Status::ExecutesCommand);
            self.schedule_deferred_callback();
            Ok(())
        } else {
            Err(ErrorCode::BUSY)
        }
    }

    fn display_off(&self) -> Result<(), ErrorCode> {
        if self.status.get() == Status::Idle {
            self.is_enabled.set(false);
            self.status.set(Status::ExecutesCommand);
            self.schedule_deferred_callback();
            Ok(())
        } else {
            Err(ErrorCode::BUSY)
        }
    }

    fn clear(&self) -> Result<(), ErrorCode> {
        if self.status.get() == Status::Idle {
            self.position.set(0);
            self.len.set(0);
            self.clear();
            self.status.set(Status::ExecutesCommand);
            self.schedule_deferred_callback();
            Ok(())
        } else {
            Err(ErrorCode::BUSY)
        }
    }
}

impl<'a, L: Led, A: Alarm<'a>> Driver for LedMatrixText<'a, L, A> {
    fn command(
        &self,
        command_number: usize,
        r2: usize,
        _r3: usize,
        _process_id: ProcessId,
    ) -> CommandReturn {
        match command_number {
            0 => CommandReturn::success(),
            1 => {
                self.speed.set(r2 as u32);
                CommandReturn::success()
            }
            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }
}
