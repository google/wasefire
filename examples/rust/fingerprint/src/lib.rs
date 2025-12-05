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

//! Demonstrates simple fingerprint usage.
//!
//! The applet uses the platform protocol applet RPC system for its interface. It can be used with
//! the companion `host` binary.

#![no_std]
wasefire::applet!();

use alloc::vec::Vec;

use common::{Request, Response};
use wasefire::error::Code;
use wasefire::fingerprint::matcher::{
    self, Enroll, EnrollProgress, Identify, IdentifyResult, delete_template, list_templates,
};
use wasefire::fingerprint::sensor::{self, Capture, Image};
use wasefire::fingerprint::{Handler, Listener};
use wasefire::sync::Mutex;

fn main() {
    rpc::Listener::new(&platform::protocol::RpcProtocol, handler).leak();
}

fn handler(request: Vec<u8>) -> Vec<u8> {
    match handler_(&request) {
        Ok(Response::CaptureImage(data)) => data,
        x => wasefire_wire::encode(&x).unwrap().into(),
    }
}

fn handler_(request: &[u8]) -> Result<Response, Error> {
    let mut state = STATE.lock();
    match wasefire_wire::decode::<Request>(request)? {
        Request::Reset => {
            *state = State::Idle;
            Ok(Response::Reset)
        }
        Request::CaptureStart if sensor::is_supported() => {
            state.set(|| Ok(State::Capture(Capture::new()?)))?;
            Ok(Response::CaptureStart)
        }
        Request::CaptureStart => Err(Error::world(Code::NotImplemented)),
        Request::CaptureDone => {
            let State::Capture(capture) = &mut *state else {
                return Err(Error::user(Code::InvalidState));
            };
            if !capture.is_done() {
                return Ok(Response::CaptureDone(None));
            }
            let State::Capture(capture) = core::mem::take(&mut *state) else { unreachable!() };
            let Image { data, width } = capture.result()?;
            *state = State::Image(data.into());
            Ok(Response::CaptureDone(Some(width)))
        }
        Request::CaptureImage => {
            if !matches!(*state, State::Image(_)) {
                return Err(Error::user(Code::InvalidState));
            }
            let State::Image(data) = core::mem::take(&mut *state) else { unreachable!() };
            Ok(Response::CaptureImage(data))
        }
        Request::EnrollStart if matcher::is_supported() => {
            state.set(|| Ok(State::Enroll(Enroll::new()?)))?;
            Ok(Response::EnrollStart)
        }
        Request::EnrollStart => Err(Error::world(Code::NotImplemented)),
        Request::EnrollDone => {
            let State::Enroll(enroll) = &mut *state else {
                return Err(Error::user(Code::InvalidState));
            };
            if let Some(EnrollProgress { detected, remaining }) = enroll.progress() {
                return Ok(Response::EnrollDone(Err((detected, remaining))));
            }
            let State::Enroll(enroll) = core::mem::take(&mut *state) else { unreachable!() };
            Ok(Response::EnrollDone(Ok(enroll.result()?.into())))
        }
        Request::IdentifyStart(id) if matcher::is_supported() => {
            state.set(|| Ok(State::Identify(Identify::new(id.as_deref())?)))?;
            Ok(Response::IdentifyStart)
        }
        Request::IdentifyStart(_) => Err(Error::world(Code::NotImplemented)),
        Request::IdentifyDone => {
            let State::Identify(identify) = &mut *state else {
                return Err(Error::user(Code::InvalidState));
            };
            if !identify.is_done() {
                return Ok(Response::IdentifyDone(None));
            }
            let State::Identify(identify) = core::mem::take(&mut *state) else { unreachable!() };
            let result = match identify.result()? {
                IdentifyResult::NoMatch => None,
                IdentifyResult::Match { template } => Some(template.into()),
            };
            Ok(Response::IdentifyDone(Some(result)))
        }
        Request::Delete(id) if matcher::is_supported() => {
            delete_template(id.as_deref())?;
            Ok(Response::Delete)
        }
        Request::Delete(_) => Err(Error::world(Code::NotImplemented)),
        Request::List if matcher::is_supported() => {
            Ok(Response::List(list_templates()?.iter().map(|x| x.to_vec()).collect()))
        }
        Request::List => Err(Error::world(Code::NotImplemented)),
        Request::DetectStart if sensor::is_supported() => {
            state.set(|| Ok(State::Detect(Listener::new(Touch)?, false)))?;
            Ok(Response::DetectStart)
        }
        Request::DetectStart => Err(Error::world(Code::NotImplemented)),
        Request::DetectConsume => {
            let State::Detect(_, touched) = &mut *state else {
                return Err(Error::user(Code::InvalidState));
            };
            Ok(Response::DetectConsume(core::mem::take(touched)))
        }
        Request::DetectStop => {
            let State::Detect(listener, _) = core::mem::take(&mut *state) else {
                return Err(Error::user(Code::InvalidState));
            };
            listener.stop();
            Ok(Response::DetectStop)
        }
    }
}

#[derive(Default)]
enum State {
    #[default]
    Idle,
    Capture(Capture),
    Image(Vec<u8>),
    Enroll(Enroll),
    Identify(Identify),
    Detect(Listener<Touch>, bool),
}

impl State {
    fn set(&mut self, new: impl FnOnce() -> Result<Self, Error>) -> Result<(), Error> {
        drop(core::mem::take(self));
        *self = new()?;
        Ok(())
    }
}

static STATE: Mutex<State> = Mutex::new(State::Idle);

struct Touch;

impl Handler for Touch {
    fn event(&self) {
        if let State::Detect(_, touched) = &mut *STATE.lock() {
            *touched = true;
        }
    }
}
