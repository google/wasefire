// Copyright 2025 Google LLC
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

use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::sync::atomic::Ordering::Relaxed;

use wasefire::button;
use wasefire::sync::{AtomicBool, Mutex};

pub(crate) type Client = Box<dyn FnOnce() + Send>;

pub(crate) struct Touch {
    touched: Arc<AtomicBool>,
}

impl Touch {
    pub(crate) fn new(client: Option<Client>) -> Self {
        Touch { touched: State::start(&mut STATE.lock(), client) }
    }

    pub(crate) fn is_present(&self) -> bool {
        self.touched.load(Relaxed)
    }
}

static STATE: Mutex<Option<State>> = Mutex::new(None);

struct State {
    button: button::Listener<Handler>,
    touched: Arc<AtomicBool>,
    clients: Vec<Box<dyn FnOnce() + Send>>,
}

impl State {
    fn start(this: &mut Option<State>, client: Option<Client>) -> Arc<AtomicBool> {
        match this {
            None => {
                let button = button::Listener::new(0, Handler).unwrap();
                let touched = Arc::new(AtomicBool::new(false));
                let clients = client.into_iter().collect();
                *this = Some(State { button, touched: touched.clone(), clients });
                touched
            }
            Some(state) => state.touched.clone(),
        }
    }

    fn touch(this: &mut Option<State>) {
        let Some(state) = this.take() else { unreachable!() };
        state.button.stop();
        state.touched.store(true, Relaxed);
        for client in state.clients {
            client();
        }
    }
}

struct Handler;

impl button::Handler for Handler {
    fn event(&self, state: button::State) {
        if matches!(state, button::State::Pressed) {
            State::touch(&mut STATE.lock());
        }
    }
}
