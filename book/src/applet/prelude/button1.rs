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

//! Demonstrates simple button usage.
//!
//! The applet prints the button state to the debug output on any button event.

//{ ANCHOR: all
#![no_std]
wasefire::applet!();

fn main() {
    //{ ANCHOR: count
    // Make sure there is at least one button.
    let count = button::count();
    assert!(count > 0, "Board has no buttons.");
    //} ANCHOR_END: count

    //{ ANCHOR: loop
    // For each button on the board.
    for index in 0 .. count {
    //} ANCHOR_END: loop
        //{ ANCHOR: handler
        // We define a button handler printing the new state.
        let handler = move |state| debug!("Button {index} has been {state:?}.");
        //} ANCHOR_END: handler

        //{ ANCHOR: listener
        // We start listening for state changes with the handler.
        let listener = button::Listener::new(index, handler);
        //} ANCHOR_END: listener

        //{ ANCHOR: leak
        // We leak the listener to continue listening for events.
        listener.leak();
        //} ANCHOR_END: leak
    }

    //{ ANCHOR: wait
    // We indefinitely wait for callbacks.
    scheduling::wait_indefinitely();
    //} ANCHOR_END: wait
}
//} ANCHOR_END: all
