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

#[cfg(feature = "applet-api-platform-protocol")]
use wasefire_applet_api::platform::protocol as applet_api;
use wasefire_board_api::platform::protocol::Api as _;
use wasefire_board_api::{self as board, Api as Board};
use wasefire_error::{Code, Error};
use wasefire_logger as log;
use wasefire_protocol::applet::AppletId;
#[cfg(feature = "wasm")]
use wasefire_protocol::applet::ExitStatus;
use wasefire_protocol::{self as service, Api, ApiResult, Request, Service, VERSION};

use crate::{Applet, Scheduler};

#[derive(Debug, Default)]
pub struct State(StateImpl);

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
    match process_event_(scheduler, event, request) {
        Ok(()) => (),
        Err(error) => reply_error::<B>(error),
    }
}

fn process_event_<B: Board>(
    scheduler: &mut Scheduler<B>, event: board::Event<B>, request: Box<[u8]>,
) -> Result<(), Error> {
    match &scheduler.protocol.0 {
        Normal { .. } => (),
        Locked => return Err(Error::user(Code::InvalidState)),
        Tunnel { applet_id, delimiter } => {
            if request == *delimiter {
                scheduler.protocol = State::default();
                return Ok(reply::<B, service::AppletTunnel>(()));
            }
            return applet::<B>(scheduler, *applet_id)?.put_request(event, &request);
        }
    }
    let request = Api::<Request>::decode(&request)?;
    match request {
        Api::ApiVersion(()) => reply::<B, service::ApiVersion>(VERSION),
        Api::AppletRequest(service::applet::Request { applet_id, request }) => {
            applet::<B>(scheduler, applet_id)?.put_request(event, request)?;
            reply::<B, service::AppletRequest>(());
        }
        Api::AppletResponse(applet_id) => {
            let response = applet::<B>(scheduler, applet_id)?.get_response()?;
            reply::<B, service::AppletResponse>(response.as_deref());
        }
        Api::PlatformReboot(()) => {
            use wasefire_board_api::platform::Api as _;
            board::Platform::<B>::reboot()?;
        }
        Api::AppletTunnel(service::applet::Tunnel { applet_id, delimiter }) => {
            let delimiter = delimiter.to_vec().into_boxed_slice();
            scheduler.protocol.0 = Tunnel { applet_id, delimiter };
            reply::<B, service::AppletTunnel>(())
        }
        Api::PlatformInfo(()) => {
            use wasefire_board_api::platform::Api as _;
            reply::<B, service::PlatformInfo>(service::platform::Info {
                serial: &board::Platform::<B>::serial(),
                version: &board::Platform::<B>::version(),
            });
        }
        Api::PlatformVendor(request) => {
            use wasefire_board_api::platform::protocol::Api as _;
            let response = board::platform::Protocol::<B>::vendor(request)?;
            reply::<B, service::PlatformVendor>(&response);
        }
        Api::PlatformUpdateMetadata(()) => {
            use wasefire_board_api::platform::update::Api as _;
            let response = board::platform::Update::<B>::metadata()?;
            reply::<B, service::PlatformUpdateMetadata>(&response);
        }
        Api::PlatformUpdateTransfer(request) => process_update::<B>(scheduler, request)?,
        #[cfg(feature = "native")]
        Api::AppletInstall(_) | Api::AppletUninstall(_) => {
            return Err(Error::world(Code::NotImplemented));
        }
        #[cfg(feature = "wasm")]
        Api::AppletInstall(request) => process_install::<B>(scheduler, request)?,
        #[cfg(feature = "wasm")]
        Api::AppletUninstall(()) => {
            use wasefire_board_api::applet::Api as _;
            board::Applet::<B>::start(false)?;
            board::Applet::<B>::finish()?;
            scheduler.stop_applet(ExitStatus::Kill);
            reply::<B, service::AppletUninstall>(());
        }
        Api::AppletExitStatus(applet_id) => {
            use crate::applet::Slot;
            let service::applet::AppletId = applet_id;
            let status = match scheduler.applet {
                #[cfg(feature = "wasm")]
                Slot::Empty => return Err(Error::user(Code::NotFound)),
                Slot::Running(_) => None,
                Slot::Exited(x) => Some(x),
            };
            reply::<B, service::AppletExitStatus>(status);
        }
        Api::PlatformLock(()) => {
            scheduler.protocol.0 = Locked;
            reply::<B, service::PlatformLock>(());
        }
        #[cfg(not(feature = "_test"))]
        _ => return Err(Error::internal(Code::NotImplemented)),
    }
    Ok(())
}

#[cfg(feature = "applet-api-platform-protocol")]
pub fn put_response<B: Board>(
    call: &mut crate::SchedulerCall<B, applet_api::write::Sig>, response: Box<[u8]>,
) -> Result<(), Error> {
    match call.applet().put_response(response) {
        Ok(()) => (),
        Err(error) if error == Error::world(Code::InvalidState) => {
            // The response was discarded because there is a new request. Send the event.
            call.applet().push(board::platform::protocol::Event.into());
            return Ok(());
        }
        Err(error) => return Err(error),
    }
    match &call.scheduler().protocol.0 {
        Normal { .. } => Ok(()),
        Locked => Err(Error::world(Code::InvalidState)),
        Tunnel { applet_id, .. } => {
            let service::applet::AppletId = applet_id;
            match call.applet().get_response()? {
                Some(response) => board::platform::Protocol::<B>::write(&response),
                None => {
                    log::error!("Failed to read response back.");
                    Err(Error::internal(Code::InvalidState))
                }
            }
        }
    }
}

fn process_update<B: Board>(
    scheduler: &mut Scheduler<B>, request: service::transfer::Request,
) -> Result<(), Error> {
    use wasefire_board_api::platform::update::Api as _;
    match request {
        service::transfer::Request::Start { dry_run } => {
            *scheduler.protocol.0.update() = TransferState::Running { dry_run };
            board::platform::Update::<B>::initialize(dry_run)?;
        }
        service::transfer::Request::Write { chunk } => {
            let _ = scheduler.protocol.0.update().dry_run()?;
            board::platform::Update::<B>::process(chunk)?;
        }
        service::transfer::Request::Finish => {
            let _ = scheduler.protocol.0.update().dry_run()?;
            board::platform::Update::<B>::finalize()?;
            *scheduler.protocol.0.update() = TransferState::Ready;
        }
    }
    reply::<B, service::PlatformUpdateTransfer>(());
    Ok(())
}

#[cfg(feature = "wasm")]
fn process_install<B: Board>(
    scheduler: &mut Scheduler<B>, request: service::transfer::Request,
) -> Result<(), Error> {
    use wasefire_board_api::applet::Api as _;
    match request {
        service::transfer::Request::Start { dry_run } => {
            if !dry_run {
                scheduler.stop_applet(ExitStatus::Kill);
            }
            *scheduler.protocol.0.install() = TransferState::Running { dry_run };
            board::Applet::<B>::start(dry_run)?;
        }
        service::transfer::Request::Write { chunk } => {
            let _ = scheduler.protocol.0.install().dry_run()?;
            board::Applet::<B>::write(chunk)?;
        }
        service::transfer::Request::Finish => {
            let dry_run = scheduler.protocol.0.install().dry_run()?;
            board::Applet::<B>::finish()?;
            *scheduler.protocol.0.install() = TransferState::Ready;
            if !dry_run {
                scheduler.start_applet()?;
            }
        }
    }
    reply::<B, service::AppletInstall>(());
    Ok(())
}

#[derive(Debug)]
enum TransferState {
    Ready,
    Running { dry_run: bool },
}

impl TransferState {
    fn dry_run(&self) -> Result<bool, Error> {
        match self {
            TransferState::Ready => Err(Error::user(Code::InvalidState)),
            TransferState::Running { dry_run } => Ok(*dry_run),
        }
    }
}

#[derive(Debug)]
enum StateImpl {
    Normal {
        update: TransferState,
        #[cfg(feature = "wasm")]
        install: TransferState,
    },
    Locked,
    Tunnel {
        applet_id: service::applet::AppletId,
        delimiter: Box<[u8]>,
    },
}
use StateImpl::*;

impl Default for StateImpl {
    fn default() -> Self {
        Normal {
            update: TransferState::Ready,
            #[cfg(feature = "wasm")]
            install: TransferState::Ready,
        }
    }
}

impl StateImpl {
    fn update(&mut self) -> &mut TransferState {
        match self {
            Normal { update, .. } => update,
            _ => unreachable!(),
        }
    }

    #[cfg(feature = "wasm")]
    fn install(&mut self) -> &mut TransferState {
        match self {
            Normal { install, .. } => install,
            _ => unreachable!(),
        }
    }
}

fn applet<B: Board>(
    scheduler: &mut Scheduler<B>, applet_id: AppletId,
) -> Result<&mut Applet<B>, Error> {
    let AppletId = applet_id;
    scheduler.applet.get().ok_or_else(|| {
        log::warn!("Failed to find applet");
        Error::world(Code::NotFound)
    })
}

fn reply<B: Board, T: Service>(response: T::Response<'_>) {
    write::<B, T>(ApiResult::Ok(response))
}

fn reply_error<B: Board>(error: Error) {
    write::<B, service::ApiVersion>(ApiResult::Err(error))
}

fn write<B: Board, T: Service>(response: ApiResult<T>) {
    if let Err(error) = board::platform::Protocol::<B>::write(&response.encode().unwrap()) {
        log::warn!("Failed to send platform protocol response: {}", error);
    }
}
