// Copyright 2022 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Implements some CTAP-like API.
//!
//! The applet combines button, LED, store, and USB serial. It describes its own usage when
//! connecting on the USB serial.

#![no_std]
wasefire::applet!();

use alloc::collections::VecDeque;
use alloc::format;
use alloc::rc::Rc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::cell::{Cell, RefCell};

use wasefire::usb::serial;

const TOUCH_TIMEOUT_MS: usize = 5_000;
const BUTTON_TIMEOUT_MS: usize = 3_000;
const LED_BLINK_MS: usize = 200;
const MAX_ENTRIES: usize = 10;

fn main() {
    let console = Rc::new(Console::default());
    // TODO: Maybe the whole concept of touch should be in the HighApi (either implement by the
    // scheduler on top of the LowApi, or in the LowApi too) or in the prelude.
    let touch = match Touch::new(console.clone()) {
        Some(x) => x,
        None => return,
    };
    let store = Store::new(&console);
    console.write("Usage: list");
    console.write("Usage: read <name>");
    console.write("Usage: write <name> <value>");
    console.write("Usage: delete <name>");
    loop {
        let prompt = match console.read() {
            Some(x) => String::from_utf8(x).unwrap(),
            _ => continue,
        };
        let cmd = match Cmd::parse(&prompt) {
            Ok(x) => x,
            Err(e) => {
                console.write(&e);
                continue;
            }
        };
        if !touch.consume() {
            continue;
        }
        match cmd {
            Cmd::List => store.list(),
            Cmd::Read { name } => store.read(name),
            Cmd::Write { name, value } => store.write(name, value),
            Cmd::Delete { name } => store.delete(name),
        }
    }
}

enum Cmd<'a> {
    List,
    Read { name: &'a str },
    Write { name: &'a str, value: &'a str },
    Delete { name: &'a str },
}

impl<'a> Cmd<'a> {
    fn parse(mut input: &'a str) -> Result<Cmd<'a>, String> {
        let cmd = match Cmd::parse_word("command", &mut input)? {
            "list" => Cmd::List,
            "read" => {
                let name = Cmd::parse_word("name", &mut input)?;
                Cmd::Read { name }
            }
            "write" => {
                let name = Cmd::parse_word("name", &mut input)?;
                let value = Cmd::parse_word("value", &mut input)?;
                Cmd::Write { name, value }
            }
            "delete" => {
                let name = Cmd::parse_word("name", &mut input)?;
                Cmd::Delete { name }
            }
            x => return Err(format!("Unrecognized command {x}.")),
        };
        if !input.is_empty() {
            return Err("Trailing arguments.".to_string());
        }
        Ok(cmd)
    }

    fn parse_word(name: &str, input: &mut &'a str) -> Result<&'a str, String> {
        if input.is_empty() {
            return Err(format!("Expected a {name}."));
        }
        let (x, r) = match input.split_once(' ') {
            Some((x, r)) => (x, r),
            None => (*input, ""),
        };
        *input = r;
        Ok(x)
    }
}

#[derive(Default)]
struct Console {
    prompt: RefCell<Vec<u8>>,
    /// Contains a backlog of strings to print if busy.
    busy: RefCell<Option<VecDeque<String>>>,
}

impl Console {
    fn read(&self) -> Option<Vec<u8>> {
        match serial::read_byte().unwrap() {
            c @ (b' ' | b'0' ..= b'9' | b'A' ..= b'Z' | b'a' ..= b'z') => {
                self.prompt.borrow_mut().push(c);
                serial::write_all(&[c]).unwrap();
                None
            }
            // Backspace
            0x7f => {
                if self.prompt.borrow_mut().pop().is_some() {
                    // Note sure if there's something more idiomatic.
                    serial::write_all(b"\x08 \x08").unwrap();
                }
                None
            }
            // Return
            0x0d => {
                let prompt = self.prompt.borrow_mut().split_off(0);
                if prompt.is_empty() {
                    None
                } else {
                    serial::write_all(b"\r\n").unwrap();
                    Some(prompt)
                }
            }
            c => {
                self.write(&format!("Unexpected input {c:#02x}."));
                None
            }
        }
    }

    fn write(&self, msg: &str) {
        if let Some(backlog) = self.busy.borrow_mut().as_mut() {
            // We don't have ownership so we push to the backlog.
            backlog.push_back(msg.to_string());
            return;
        }
        // We can take ownership and create an empty backlog.
        *self.busy.borrow_mut() = Some(VecDeque::new());
        // We write our message.
        self.write_msg(msg);
        // We empty the backlog.
        while let Some(msg) = self.busy.borrow_mut().as_mut().unwrap().pop_front() {
            self.write_msg(&msg);
        }
        // We release ownership.
        *self.busy.borrow_mut() = None;
    }

    fn write_msg(&self, msg: &str) {
        serial::write_all(format!("\r\x1b[K{msg}\r\n> ").as_bytes()).unwrap();
        let mut prompt = self.prompt.borrow_mut().split_off(0);
        serial::write_all(&prompt).unwrap();
        prompt.append(&mut *self.prompt.borrow_mut());
        *self.prompt.borrow_mut() = prompt;
    }
}

struct Touch {
    console: Rc<Console>,
    light: Rc<Cell<led::Status>>,
    touched: Rc<Cell<bool>>,
    blink: Rc<clock::Timer<BlinkHandler>>,
    timeout: Rc<clock::Timer<TimeoutHandler>>,
    _button: button::Listener<ButtonHandler>,
}

struct BlinkHandler {
    light: Rc<Cell<led::Status>>,
}

impl clock::Handler for BlinkHandler {
    fn event(&self) {
        self.light.set(!self.light.get());
        led::set(0, self.light.get());
    }
}

struct TimeoutHandler {
    console: Rc<Console>,
    light: Rc<Cell<led::Status>>,
    touched: Rc<Cell<bool>>,
    blink: Rc<clock::Timer<BlinkHandler>>,
}

impl clock::Handler for TimeoutHandler {
    fn event(&self) {
        if !self.touched.get() {
            return;
        }
        self.touched.set(false);
        Touch::blink_stop(&self.light, &self.blink);
        self.console.write("Button timed out.");
    }
}

struct ButtonHandler {
    console: Rc<Console>,
    light: Rc<Cell<led::Status>>,
    touched: Rc<Cell<bool>>,
    blink: Rc<clock::Timer<BlinkHandler>>,
    timeout: Rc<clock::Timer<TimeoutHandler>>,
}

impl button::Handler for ButtonHandler {
    fn event(&self, state: button::State) {
        if state != button::Pressed {
            return;
        }
        self.console.write("Button pressed.");
        self.touched.set(true);
        Touch::blink_start(&self.light, &self.blink);
        self.timeout.start_ms(clock::Oneshot, BUTTON_TIMEOUT_MS);
    }
}

impl Touch {
    fn new(console: Rc<Console>) -> Option<Self> {
        if button::count() == 0 {
            console.write("Needs at least one button.");
            return None;
        }
        if led::count() == 0 {
            console.write("Needs at least one LED.");
            return None;
        }
        let light = Rc::new(Cell::new(led::Off));
        let touched = Rc::new(Cell::new(false));
        let blink = Rc::new(clock::Timer::new(BlinkHandler { light: light.clone() }));
        let timeout = Rc::new(clock::Timer::new(TimeoutHandler {
            console: console.clone(),
            light: light.clone(),
            touched: touched.clone(),
            blink: blink.clone(),
        }));
        let _button = button::Listener::new(
            0,
            ButtonHandler {
                console: console.clone(),
                light: light.clone(),
                touched: touched.clone(),
                blink: blink.clone(),
                timeout: timeout.clone(),
            },
        );
        Some(Self { console, light, touched, blink, timeout, _button })
    }

    fn consume(&self) -> bool {
        if self.touched.get() {
            self.console.write("Button consumed.");
            self.touched.set(false);
            Self::blink_stop(&self.light, &self.blink);
            self.timeout.stop();
            return true;
        }
        self.console.write("Waiting for touch...");
        Self::blink_start(&self.light, &self.blink);
        let timed_out = Rc::new(Cell::new(false));
        let timer = clock::Timer::new({
            let timed_out = timed_out.clone();
            move || timed_out.set(true)
        });
        timer.start_ms(clock::Oneshot, TOUCH_TIMEOUT_MS);
        loop {
            scheduling::wait_for_callback();
            if self.touched.get() {
                self.touched.set(false);
                Self::blink_stop(&self.light, &self.blink);
                return true;
            }
            if timed_out.get() {
                self.console.write("Timed out.");
                Self::blink_stop(&self.light, &self.blink);
                return false;
            }
        }
    }

    fn blink_start(light: &Cell<led::Status>, blink: &clock::Timer<BlinkHandler>) {
        light.set(led::On);
        led::set(0, light.get());
        blink.start_ms(clock::Periodic, LED_BLINK_MS);
    }

    fn blink_stop(light: &Cell<led::Status>, blink: &clock::Timer<BlinkHandler>) {
        blink.stop();
        light.set(led::Off);
        led::set(0, light.get());
    }
}

struct Store<'a> {
    console: &'a Console,
}

impl<'a> Store<'a> {
    fn new(console: &'a Console) -> Self {
        Self { console }
    }

    fn list(&self) {
        let mut empty = true;
        for key in (0 .. 2 * MAX_ENTRIES).step_by(2) {
            if let Some(x) = store::find(key).unwrap() {
                empty = false;
                let x = core::str::from_utf8(&x).unwrap_or("(corrupted)");
                self.console.write(&format!("- {x}"));
            }
        }
        if empty {
            self.console.write("No entries.");
        }
    }

    fn read(&self, name: &str) {
        let key = match self.find(name) {
            Ok(x) => x,
            Err(_) => {
                self.console.write("Not found.");
                return;
            }
        };
        let value = match store::find(key + 1).unwrap() {
            Some(x) => x,
            None => {
                self.console.write("Missing value.");
                return;
            }
        };
        let value = match core::str::from_utf8(&value) {
            Ok(x) => x,
            Err(_) => {
                self.console.write("Corrupted value (not UTF-8).");
                return;
            }
        };
        self.console.write(&format!("Read {value} from {name}."));
    }

    fn write(&self, name: &str, value: &str) {
        let key = match self.find(name) {
            Ok(x) => x,
            Err(x) => x,
        };
        store::insert(key, name.as_bytes()).unwrap();
        store::insert(key + 1, value.as_bytes()).unwrap();
        self.console.write(&format!("Write {value} into {name}."));
    }

    fn delete(&self, name: &str) {
        let key = match self.find(name) {
            Ok(x) => x,
            Err(_) => {
                self.console.write("Not found.");
                return;
            }
        };
        store::remove(key).unwrap();
        store::remove(key + 1).unwrap();
        self.console.write(&format!("Delete {name}."));
    }

    fn find(&self, name: &str) -> Result<usize, usize> {
        for key in (0 .. 2 * MAX_ENTRIES).step_by(2) {
            match store::find(key).unwrap() {
                Some(x) if *name.as_bytes() == *x => return Ok(key),
                Some(_) => (),
                None => return Err(key),
            }
        }
        Err(MAX_ENTRIES)
    }
}
