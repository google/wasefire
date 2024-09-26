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

//! Device side of the protocol.

use std::future::Future;
use std::marker::PhantomData;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::{Arc, Mutex};

use anyhow::Result;
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream, UnixListener, UnixStream};
use tokio::select;
use tokio::sync::Notify;
use tokio::task::JoinHandle;
use wasefire_board_api::platform::protocol::{Api, Event};
use wasefire_error::{Code, Error};
use wasefire_logger as log;

use crate::common::{read, write};

/// Platform protocol implementation.
pub struct Impl<T: HasPipe> {
    _never: !,
    _phantom: PhantomData<T>,
}

/// Simplified platform protocol interface.
pub trait HasPipe {
    /// Executes a function with exclusive access to the pipe.
    fn with_pipe<R>(f: impl FnOnce(&mut Pipe) -> R) -> R;

    /// Handles vendor-specific requests.
    fn vendor(request: &[u8]) -> Result<Box<[u8]>, Error>;
}

pub struct Pipe {
    handle: JoinHandle<!>,
    shared: Shared,
}

pub trait Push = Fn(Event) + Send + Sync + 'static;

impl Drop for Pipe {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

impl Pipe {
    pub async fn new_unix<P: Push>(path: &Path, push: P) -> Result<Self> {
        Self::new::<UnixListener, P>(path, push).await
    }

    pub async fn new_tcp<P: Push>(addr: SocketAddr, push: P) -> Result<Self> {
        Self::new::<TcpListener, P>(addr, push).await
    }

    async fn new<L: Listener, P: Push>(bind: L::Name<'_>, push: P) -> Result<Self> {
        let shared = Shared {
            notify: Arc::new(Notify::new()),
            state: Arc::new(Mutex::new(State::Disabled)),
        };
        let handle =
            tokio::spawn(Self::manage_listener(L::bind(bind).await?, shared.clone(), push));
        Ok(Pipe { handle, shared })
    }

    async fn manage_listener<L: Listener, P: Push>(listener: L, shared: Shared, push: P) -> ! {
        loop {
            // We dissociate the branches from the match to make it clear that the mutex is not held
            // through an await point.
            let enabled = match *shared.state.lock().unwrap() {
                State::Disabled => false,
                State::Accept => true,
                _ => unreachable!(),
            };
            if enabled {
                log::info!("Listening for a {} connection.", L::NAME);
                let mut stream = select! {
                    x = listener.accept() => x.unwrap(),
                    () = shared.notify.notified() => continue,
                };
                log::info!("Accepted a {} connection.", L::NAME);
                match &mut *shared.state.lock().unwrap() {
                    x @ State::Accept => *x = State::Ready,
                    _ => continue,
                }
                match Self::manage_stream::<L, P>(&mut stream, &shared, &push).await {
                    Ok(()) => (),
                    Err(()) => match &mut *shared.state.lock().unwrap() {
                        State::Disabled => (),
                        x => *x = State::Accept,
                    },
                }
                log::info!("Shutting down {} connection.", L::NAME);
                let _ = stream.shutdown().await;
            } else {
                shared.notify.notified().await;
            }
        }
    }

    async fn manage_stream<L: Listener, P: Push>(
        stream: &mut L::Stream, shared: &Shared, push: &P,
    ) -> Result<(), ()> {
        loop {
            enum Action {
                Receive,
                Wait,
                Send { response: Box<[u8]> },
            }
            let action = {
                let mut state = shared.state.lock().unwrap();
                match &mut *state {
                    State::Disabled | State::Accept => return Ok(()),
                    State::Ready => Action::Receive,
                    State::Request { .. } | State::Process => Action::Wait,
                    State::Response { response } => {
                        let response = std::mem::take(response);
                        *state = State::Ready;
                        Action::Send { response }
                    }
                }
            };
            match action {
                Action::Receive => {
                    let request = read(stream).await?;
                    log::debug!("Received a request of {} bytes.", request.len());
                    if let x @ State::Ready = &mut *shared.state.lock().unwrap() {
                        *x = State::Request { request };
                        push(Event);
                    }
                }
                Action::Wait => shared.notify.notified().await,
                Action::Send { response } => {
                    log::debug!("Writing a response of {} bytes.", response.len());
                    write(stream, &response).await?;
                }
            }
        }
    }
}

impl<T: HasPipe> Api for Impl<T> {
    fn read() -> Result<Option<Box<[u8]>>, Error> {
        T::with_pipe(|x| {
            let mut state = x.shared.state.lock().unwrap();
            match &mut *state {
                State::Disabled | State::Process => Err(Error::user(Code::InvalidState)),
                State::Accept | State::Ready | State::Response { .. } => Ok(None),
                State::Request { request } => {
                    let request = std::mem::take(request);
                    *state = State::Process;
                    x.shared.notify.notify_one();
                    Ok(Some(request))
                }
            }
        })
    }

    fn write(response: &[u8]) -> Result<(), Error> {
        T::with_pipe(|x| {
            let mut state = x.shared.state.lock().unwrap();
            match &mut *state {
                State::Process => {
                    let response = response.to_vec().into_boxed_slice();
                    *state = State::Response { response };
                    x.shared.notify.notify_one();
                    Ok(())
                }
                _ => Err(Error::user(Code::InvalidState)),
            }
        })
    }

    fn enable() -> Result<(), Error> {
        T::with_pipe(|x| {
            let mut state = x.shared.state.lock().unwrap();
            match &mut *state {
                State::Disabled => {
                    *state = State::Accept;
                    x.shared.notify.notify_one();
                    Ok(())
                }
                _ => Err(Error::user(Code::InvalidState)),
            }
        })
    }

    fn vendor(request: &[u8]) -> Result<Box<[u8]>, Error> {
        T::vendor(request)
    }
}

enum State {
    Disabled,
    Accept,
    Ready,
    Request { request: Box<[u8]> },
    Process,
    Response { response: Box<[u8]> },
}

#[derive(Clone)]
struct Shared {
    state: Arc<Mutex<State>>,
    notify: Arc<Notify>,
}

trait Listener: Sized + Send + 'static {
    const NAME: &str;
    type Stream: AsyncRead + AsyncWrite + Send + Unpin;
    type Name<'a>: Copy;
    async fn bind(name: Self::Name<'_>) -> std::io::Result<Self>;
    fn accept(&self) -> impl Future<Output = std::io::Result<Self::Stream>> + Send;
}

impl Listener for UnixListener {
    const NAME: &str = "Unix";
    type Stream = UnixStream;
    type Name<'a> = &'a Path;
    async fn bind(name: Self::Name<'_>) -> std::io::Result<Self> {
        Self::bind(name)
    }
    async fn accept(&self) -> std::io::Result<Self::Stream> {
        Ok(self.accept().await?.0)
    }
}

impl Listener for TcpListener {
    const NAME: &str = "TCP";
    type Stream = TcpStream;
    type Name<'a> = SocketAddr;
    async fn bind(name: Self::Name<'_>) -> std::io::Result<Self> {
        Self::bind(name).await
    }
    async fn accept(&self) -> std::io::Result<Self::Stream> {
        Ok(self.accept().await?.0)
    }
}
