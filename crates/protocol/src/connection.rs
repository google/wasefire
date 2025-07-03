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
use alloc::format;
use core::any::Any;
use core::future::Future;
use core::pin::Pin;

use anyhow::Context;
use wasefire_wire::Yoke;

use crate::{Api, ApiResult, Request, Service};

pub type DynFuture<'a, T> = Pin<Box<dyn Future<Output = anyhow::Result<T>> + 'a>>;

pub trait Connection: Any + Send {
    /// Receives a raw response (possibly tunneled) from the device.
    fn read(&mut self) -> DynFuture<'_, Box<[u8]>>;

    /// Sends a raw request (possibly tunneled) to the device.
    fn write<'a>(&'a mut self, response: &'a [u8]) -> DynFuture<'a, ()>;
}

impl Connection for Box<dyn Connection> {
    fn read(&mut self) -> DynFuture<'_, Box<[u8]>> {
        (**self).read()
    }

    fn write<'a>(&'a mut self, response: &'a [u8]) -> DynFuture<'a, ()> {
        (**self).write(response)
    }
}

pub trait ConnectionExt: Connection {
    /// Calls a service on the device.
    fn call<S: Service>(
        &mut self, request: S::Request<'_>,
    ) -> impl Future<Output = anyhow::Result<Yoke<S::Response<'static>>>> {
        async { self.call_ref::<S>(&S::request(request)).await }
    }

    fn call_ref<S: Service>(
        &mut self, request: &Api<Request>,
    ) -> impl Future<Output = anyhow::Result<Yoke<S::Response<'static>>>> {
        async {
            self.send(request).await.with_context(|| format!("sending {}", S::NAME))?;
            self.receive::<S>().await.with_context(|| format!("receiving {}", S::NAME))
        }
    }

    /// Sends a request to the device.
    fn send(&mut self, request: &Api<'_, Request>) -> impl Future<Output = anyhow::Result<()>> {
        async {
            let request = request.encode().context("encoding request")?;
            self.write(&request).await
        }
    }

    /// Receives a response from the device.
    fn receive<S: Service>(
        &mut self,
    ) -> impl Future<Output = anyhow::Result<Yoke<S::Response<'static>>>> {
        async {
            let response = self.read().await?;
            let response = ApiResult::<S>::decode_yoke(response).context("decoding response")?;
            response.try_map(|x| match x {
                ApiResult::Ok(x) => Ok(x),
                ApiResult::Err(error) => Err(anyhow::Error::new(error)),
            })
        }
    }
}

impl<T: Connection + ?Sized> ConnectionExt for T {}
