// Copyright 2024 Google LLC
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

use wasefire_board_api::platform::protocol::Api as _;
use wasefire_board_api::{self as board, Api as Board};
use wasefire_error::{Code, Error};
use wasefire_logger as log;
use wasefire_protocol::{self as service, Api, Request, Response};

use crate::Scheduler;

#[derive(Debug, Default)]
pub enum State {
    #[default]
    Normal,
    Tunnel {
        applet_id: service::applet::AppletId,
        delimiter: Box<[u8]>,
    },
}

pub fn enable<B: Board>() {
    if let Err(error) = board::platform::Protocol::<B>::enable() {
        log::warn!("Failed to enable platform protocol: {}", error);
    }
}

pub fn should_process_event<B: Board>(event: &board::Event<B>) -> bool {
    *event == board::Event::from(board::platform::protocol::Event)
}

pub fn process_event<B: Board>(scheduler: &mut Scheduler<B>, event: board::Event<B>) {
    let request = match board::platform::Protocol::<B>::read() {
        Ok(Some(x)) => x,
        Ok(None) => return log::warn!("Expected platform protocol request, but found none."),
        Err(error) => return log::warn!("Failed to read platform protocol request: {}", error),
    };
    match &scheduler.protocol {
        State::Normal => (),
        State::Tunnel { applet_id, delimiter } => {
            if request == *delimiter {
                scheduler.protocol = State::Normal;
                return reply::<B>(&Api::AppletTunnel(()));
            }
            let service::applet::AppletId = applet_id;
            match scheduler.applet.put_request(event, &request) {
                Ok(()) => (),
                Err(error) => {
                    log::warn!("Failed to put platform protocol request: {}", error);
                    reply_error::<B>(error);
                }
            }
            return;
        }
    }
    let request = match Api::<Request>::decode(&request) {
        Ok(x) => x,
        Err(error) => {
            log::warn!("Failed to deserialize platform protocol request: {}", error);
            return reply_error::<B>(error);
        }
    };
    let owned_response;
    let response = match request {
        Api::DeviceError(x) => match x {},
        Api::AppletRequest(service::applet::Request { applet_id, request }) => {
            let service::applet::AppletId = applet_id;
            match scheduler.applet.put_request(event, request) {
                Ok(()) => Api::AppletRequest(()),
                Err(e) => Api::DeviceError(e),
            }
        }
        Api::AppletResponse(applet_id) => {
            let service::applet::AppletId = applet_id;
            match scheduler.applet.get_response() {
                Ok(response) => {
                    let response = match response {
                        None => None,
                        Some(response) => {
                            owned_response = response;
                            Some(&owned_response[..])
                        }
                    };
                    Api::AppletResponse(service::applet::Response { response })
                }
                Err(e) => Api::DeviceError(e),
            }
        }
        #[cfg(feature = "board-api-platform")]
        Api::PlatformReboot(()) => {
            use wasefire_board_api::platform::Api as _;
            match board::Platform::<B>::reboot() {
                Ok(x) => x,
                Err(e) => Api::DeviceError(e),
            }
        }
        Api::AppletTunnel(service::applet::Tunnel { applet_id, delimiter }) => {
            match scheduler.protocol {
                State::Normal => {
                    let delimiter = delimiter.to_vec().into_boxed_slice();
                    scheduler.protocol = State::Tunnel { applet_id, delimiter };
                    Api::AppletTunnel(())
                }
                State::Tunnel { .. } => unreachable!(),
            }
        }
        #[cfg(not(feature = "_test"))]
        _ => Api::DeviceError(Error::internal(Code::NotImplemented)),
    };
    reply::<B>(&response);
}

pub fn put_response<B: Board>(
    scheduler: &mut Scheduler<B>, response: Box<[u8]>,
) -> Result<(), Error> {
    match scheduler.applet.put_response(response) {
        Ok(()) => (),
        Err(error) if error == Error::world(Code::InvalidState) => {
            // The response was discarded because there is a new request. Send the event.
            scheduler.applet.push(board::platform::protocol::Event.into());
            return Ok(());
        }
        Err(error) => return Err(error),
    }
    match &scheduler.protocol {
        State::Normal => Ok(()),
        State::Tunnel { applet_id, .. } => {
            let service::applet::AppletId = applet_id;
            match scheduler.applet.get_response()? {
                Some(response) => board::platform::Protocol::<B>::write(&response),
                None => {
                    log::error!("Failed to read response back.");
                    Err(Error::internal(Code::InvalidState))
                }
            }
        }
    }
}

fn reply<B: Board>(response: &Api<Response>) {
    match response.encode() {
        Ok(response) => write::<B>(&response),
        Err(error) => reply_error::<B>(Error::internal(error.code())),
    }
}

fn reply_error<B: Board>(error: Error) {
    write::<B>(&Api::<Response>::DeviceError(error).encode().unwrap());
}

fn write<B: Board>(data: &[u8]) {
    if let Err(error) = board::platform::Protocol::<B>::write(data) {
        log::warn!("Failed to send platform protocol response: {}", error);
    }
}
