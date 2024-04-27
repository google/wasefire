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

#![no_std]
wasefire::applet!();

use core::time::Duration;

use wasefire::sync::AtomicBool;
use wasefire::sync::Ordering::Relaxed;

fn main() -> ! {
    blink(5);
    press();
    blink(3);
    debug!("Rebooting...");
    platform::reboot();
}

fn press() {
    if button::count() == 0 {
        debug!("Not waiting for button press because there are no buttons.");
        return;
    }
    debug!("Waiting for a button press to reboot...");
    static PRESSED: AtomicBool = AtomicBool::new(false);
    let button = button::Listener::new(0, |event| match event {
        button::Pressed => PRESSED.store(true, Relaxed),
        button::Released => (),
    })
    .unwrap();
    scheduling::wait_until(|| PRESSED.load(Relaxed));
    drop(button);
}

fn blink(seconds: u64) {
    debug!("Waiting {seconds} seconds...");
    let duration = Duration::from_secs(seconds);
    if led::count() == 0 {
        timer::sleep(duration);
        return;
    }
    led::set(0, led::On);
    let blink = timer::Timer::new(|| led::set(0, !led::get(0)));
    blink.start(timer::Periodic, Duration::from_millis(300));
    timer::sleep(duration);
    drop(blink);
    led::set(0, led::Off);
}
