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

use alloc::borrow::Cow;
use alloc::boxed::Box;

#[cfg(feature = "applet-api-platform-protocol")]
use wasefire_applet_api::platform::protocol as applet_api;
use wasefire_board_api::platform::protocol::Api as _;
use wasefire_board_api::transfer::Api as _;
use wasefire_board_api::{self as board, Api as Board};
use wasefire_error::{Code, Error};
use wasefire_logger as log;
#[cfg(feature = "applet-api-platform-protocol")]
use wasefire_protocol::applet::AppletId;
#[cfg(any(feature = "pulley", feature = "wasm"))]
use wasefire_protocol::applet::ExitStatus;
use wasefire_protocol::common::AppletKind;
use wasefire_protocol::{self as service, Api, ApiResult, Request, Service, VERSION};

use crate::Scheduler;

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
    #[cfg(not(feature = "applet-api-platform-protocol"))]
    let _ = event;
    match &scheduler.protocol.0 {
        Normal { .. } => (),
        Locked => return Err(Error::user(Code::InvalidState)),
        #[cfg(feature = "applet-api-platform-protocol")]
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
        #[cfg(not(feature = "applet-api-platform-protocol"))]
        Api::AppletRequest(_) => return Err(Error::world(Code::NotImplemented)),
        #[cfg(feature = "applet-api-platform-protocol")]
        Api::AppletRequest(service::applet::Request { applet_id, request }) => {
            applet::<B>(scheduler, applet_id)?.put_request(event, &request)?;
            reply::<B, service::AppletRequest>(());
        }
        #[cfg(not(feature = "applet-api-platform-protocol"))]
        Api::AppletResponse(_) => return Err(Error::world(Code::NotImplemented)),
        #[cfg(feature = "applet-api-platform-protocol")]
        Api::AppletResponse(applet_id) => {
            let response = applet::<B>(scheduler, applet_id)?.get_response()?;
            let response = response.map(|x| Cow::Owned(x.into_vec()));
            reply::<B, service::AppletResponse>(response);
        }
        Api::PlatformReboot(()) => {
            use wasefire_board_api::platform::Api as _;
            board::Platform::<B>::reboot()?;
        }
        #[cfg(not(feature = "applet-api-platform-protocol"))]
        Api::AppletTunnel(_) => return Err(Error::world(Code::NotImplemented)),
        #[cfg(feature = "applet-api-platform-protocol")]
        Api::AppletTunnel(service::applet::Tunnel { applet_id, delimiter }) => {
            let delimiter = delimiter.to_vec().into_boxed_slice();
            scheduler.protocol.0 = Tunnel { applet_id, delimiter };
            reply::<B, service::AppletTunnel>(())
        }
        Api::PlatformInfo(()) => {
            use wasefire_board_api::platform::Api as _;
            reply::<B, service::PlatformInfo>(service::platform::Info {
                serial: board::Platform::<B>::serial(),
                #[cfg(feature = "wasm")]
                applet_kind: AppletKind::Wasm,
                #[cfg(feature = "pulley")]
                applet_kind: AppletKind::Pulley,
                #[cfg(feature = "native")]
                applet_kind: AppletKind::Native,
                running_side: board::Platform::<B>::running_side(),
                running_info: board::Platform::<B>::running_info(),
                opposite_info: board::Platform::<B>::opposite_info(),
            });
        }
        Api::PlatformVendor(request) => {
            use wasefire_board_api::platform::protocol::Api as _;
            let response = board::platform::Protocol::<B>::vendor(&request)?;
            reply::<B, service::PlatformVendor>(Cow::Owned(response.into_vec()));
        }
        Api::PlatformUpdate(request) => {
            process_transfer::<B, service::PlatformUpdate>(scheduler, request)?
        }
        #[cfg(feature = "native")]
        Api::AppletInstall(_) => return Err(Error::world(Code::NotImplemented)),
        #[cfg(any(feature = "pulley", feature = "wasm"))]
        Api::AppletInstall(request) => {
            process_transfer::<B, service::AppletInstall>(scheduler, request)?
        }
        Api::AppletExitStatus(applet_id) => {
            use crate::applet::Slot;
            let service::applet::AppletId = applet_id;
            let status = match scheduler.applet {
                #[cfg(any(feature = "pulley", feature = "wasm"))]
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
        #[cfg(feature = "board-api-storage")]
        Api::PlatformClearStore(min_key) => {
            scheduler.store.clear(min_key)?;
            reply::<B, service::PlatformClearStore>(());
        }
        #[cfg(not(feature = "board-api-storage"))]
        Api::PlatformClearStore(_) => return Err(Error::world(Code::NotImplemented)),
        #[cfg(feature = "native")]
        Api::AppletReboot(_) => return Err(Error::world(Code::NotImplemented)),
        #[cfg(any(feature = "pulley", feature = "wasm"))]
        Api::AppletReboot(applet_id) => {
            let service::applet::AppletId = applet_id;
            scheduler.stop_applet(ExitStatus::Kill);
            scheduler.start_applet();
            reply::<B, service::AppletReboot>(());
        }
        #[cfg(feature = "native")]
        Api::AppletMetadata(_) => return Err(Error::world(Code::NotImplemented)),
        #[cfg(any(feature = "pulley", feature = "wasm"))]
        Api::AppletMetadata(applet_id) => {
            let service::applet::AppletId = applet_id;
            let Some(metadata) = &scheduler.applet_metadata else {
                return Err(Error::user(Code::NotFound));
            };
            reply::<B, service::AppletMetadata>(metadata.clone());
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

trait TransferKind<B: Board>
where Self: for<'a> Service<Response<'a> = service::transfer::Response>
{
    type Api: board::transfer::Api;
    fn start(scheduler: &mut Scheduler<B>);
    fn finish(scheduler: &mut Scheduler<B>);
    fn state(scheduler: &mut Scheduler<B>) -> &mut TransferState;
}

impl<B: Board> TransferKind<B> for service::PlatformUpdate {
    type Api = board::platform::Update<B>;
    fn start(_: &mut Scheduler<B>) {}
    fn finish(_: &mut Scheduler<B>) {}
    fn state(scheduler: &mut Scheduler<B>) -> &mut TransferState {
        scheduler.protocol.0.update()
    }
}

#[cfg(any(feature = "pulley", feature = "wasm"))]
impl<B: Board> TransferKind<B> for service::AppletInstall {
    type Api = board::applet::Install<B>;
    fn start(scheduler: &mut Scheduler<B>) {
        scheduler.stop_applet(ExitStatus::Kill);
    }
    fn finish(scheduler: &mut Scheduler<B>) {
        scheduler.start_applet();
    }
    fn state(scheduler: &mut Scheduler<B>) -> &mut TransferState {
        scheduler.protocol.0.install()
    }
}

fn process_transfer<B: Board, T: TransferKind<B>>(
    scheduler: &mut Scheduler<B>, request: service::transfer::Request,
) -> Result<(), Error> {
    let response = match request {
        service::transfer::Request::Start { dry_run } => {
            if !dry_run {
                T::start(scheduler);
            }
            *T::state(scheduler) = TransferState::Running { dry_run };
            let num_pages = T::Api::start(dry_run)?;
            let chunk_size = T::Api::CHUNK_SIZE;
            service::transfer::Response::Start { chunk_size, num_pages }
        }
        service::transfer::Request::Erase => {
            let _ = T::state(scheduler).dry_run()?;
            T::Api::erase()?;
            service::transfer::Response::Erase
        }
        service::transfer::Request::Write { chunk } => {
            let _ = T::state(scheduler).dry_run()?;
            T::Api::write(&chunk)?;
            service::transfer::Response::Write
        }
        service::transfer::Request::Finish => {
            let dry_run = T::state(scheduler).dry_run()?;
            T::Api::finish()?;
            *T::state(scheduler) = TransferState::Ready;
            if !dry_run {
                T::finish(scheduler);
            }
            service::transfer::Response::Finish
        }
    };
    reply::<B, T>(response);
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
        #[cfg(any(feature = "pulley", feature = "wasm"))]
        install: TransferState,
    },
    Locked,
    #[cfg(feature = "applet-api-platform-protocol")]
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
            #[cfg(any(feature = "pulley", feature = "wasm"))]
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

    #[cfg(any(feature = "pulley", feature = "wasm"))]
    fn install(&mut self) -> &mut TransferState {
        match self {
            Normal { install, .. } => install,
            _ => unreachable!(),
        }
    }
}

#[cfg(feature = "applet-api-platform-protocol")]
fn applet<B: Board>(
    scheduler: &mut Scheduler<B>, applet_id: AppletId,
) -> Result<&mut crate::Applet<B>, Error> {
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
