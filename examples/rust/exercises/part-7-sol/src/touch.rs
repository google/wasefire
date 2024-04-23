// Copyright 2023 Google LLC
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

use alloc::rc::Rc;
use alloc::string::String;
use core::cell::{Cell, RefCell};
use core::time::Duration;

use wasefire::{button, debug, led, scheduling, timer};

#[derive(Default, Clone)]
pub struct Touch(Rc<RefCell<Option<TouchImpl>>>);

struct TouchImpl {
    touched: bool,
    _button: button::Listener<Touch>,
    blink: timer::Timer<Blink>,
    timeout: timer::Timer<Touch>,
}

struct Blink;

impl Touch {
    pub fn new() -> Self {
        let touch = Touch::default();
        let _button = button::Listener::new(0, touch.clone()).unwrap();
        let blink = timer::Timer::new(Blink);
        let timeout = timer::Timer::new(touch.clone());
        let touch_impl = TouchImpl { touched: false, _button, blink, timeout };
        assert!(touch.0.borrow_mut().replace(touch_impl).is_none());
        touch
    }

    pub fn consume(&self) -> Result<(), String> {
        let mut touch_guard = self.0.borrow_mut();
        let touch = touch_guard.as_mut().unwrap();
        if touch.touched {
            touch.touched = false;
            touch.blink_stop();
            touch.timeout.stop();
            debug!("Consumed touch.");
            return Ok(());
        }
        debug!("Waiting touch.");
        touch.blink_start();
        let timed_out = Rc::new(Cell::new(false));
        let timer = timer::Timer::new({
            let timed_out = timed_out.clone();
            move || timed_out.set(true)
        });
        timer.start(timer::Oneshot, Duration::from_secs(5));
        drop(touch_guard);
        loop {
            scheduling::wait_for_callback();
            let mut touch_guard = self.0.borrow_mut();
            let touch = touch_guard.as_mut().unwrap();
            if touch.touched {
                debug!("Received touch.");
                touch.touched = false;
                touch.blink_stop();
                return Ok(());
            }
            if timed_out.get() {
                debug!("Touch timed out.");
                touch.blink_stop();
                Err("touch timeout")?;
            }
        }
    }
}

impl button::Handler for Touch {
    fn event(&self, state: button::State) {
        if state != button::Pressed {
            return;
        }
        debug!("Received touch.");
        let mut touch_guard = self.0.borrow_mut();
        let touch = touch_guard.as_mut().unwrap();
        touch.touched = true;
        touch.blink_start();
        touch.timeout.start(timer::Oneshot, Duration::from_secs(3));
    }
}

impl timer::Handler for Blink {
    fn event(&self) {
        led::set(0, !led::get(0));
    }
}

impl timer::Handler for Touch {
    fn event(&self) {
        let mut touch_guard = self.0.borrow_mut();
        let touch = touch_guard.as_mut().unwrap();
        if !touch.touched {
            return;
        }
        touch.touched = false;
        touch.blink_stop();
        debug!("Touch timed out.");
    }
}

impl TouchImpl {
    fn blink_start(&self) {
        led::set(0, led::On);
        self.blink.start(timer::Periodic, Duration::from_millis(200));
    }

    fn blink_stop(&self) {
        self.blink.stop();
        led::set(0, led::Off);
    }
}
