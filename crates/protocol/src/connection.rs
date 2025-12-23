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
use core::ops::Deref;
use core::pin::Pin;

use anyhow::{Context, ensure};
use wasefire_wire::Yoke;

use crate::{Api, ApiResult, Request, Service, VERSION};

/// Connection with a cached API version.
#[derive(PartialEq, Eq)]
pub struct Device<T: Connection> {
    version: u32,
    connection: T,
}

pub type DynDevice = Device<Box<dyn Connection>>;

impl<T: Connection> Device<T> {
    /// Checks whether the device is supported and caches the API version.
    pub async fn new(connection: T) -> anyhow::Result<Self> {
        let version = *connection.call::<crate::ApiVersion>(()).await?.get();
        ensure!(version <= VERSION, "the device is more recent than the host");
        Ok(Device { version, connection })
    }

    /// Returns the cached API version.
    pub fn version(&self) -> u32 {
        self.version
    }

    /// Extracts the connection.
    pub fn connection(self) -> T {
        self.connection
    }

    /// Returns whether the device supports a service.
    pub fn supports<S: Service>(&self) -> bool {
        // The device is not newer than the host by invariant.
        S::VERSIONS.contains(self.version).unwrap()
    }
}

impl<T: Connection> Deref for Device<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.connection
    }
}

pub type DynFuture<'a, T> = Pin<Box<dyn Future<Output = anyhow::Result<T>> + 'a>>;

pub trait Connection: Any {
    /// Sends a raw request (possibly tunneled) to the device.
    fn write<'a>(&'a self, request: &'a [u8]) -> DynFuture<'a, ()>;

    /// Receives a raw response (possibly tunneled) from the device.
    fn read(&self) -> DynFuture<'_, Box<[u8]>>;
}

impl Connection for Box<dyn Connection> {
    fn write<'a>(&'a self, request: &'a [u8]) -> DynFuture<'a, ()> {
        (**self).write(request)
    }

    fn read(&self) -> DynFuture<'_, Box<[u8]>> {
        (**self).read()
    }
}

pub trait ConnectionExt: Connection {
    /// Calls a service on the device.
    fn call<S: Service>(
        &self, request: S::Request<'_>,
    ) -> impl Future<Output = anyhow::Result<Yoke<S::Response<'static>>>> {
        async { self.call_ref::<S>(&S::request(request)).await }
    }

    fn call_ref<S: Service>(
        &self, request: &Api<Request>,
    ) -> impl Future<Output = anyhow::Result<Yoke<S::Response<'static>>>> {
        async {
            self.send(request).await.with_context(|| format!("sending {}", S::NAME))?;
            self.receive::<S>().await.with_context(|| format!("receiving {}", S::NAME))
        }
    }

    /// Sends a request to the device.
    fn send(&self, request: &Api<'_, Request>) -> impl Future<Output = anyhow::Result<()>> {
        async {
            let request = request.encode().context("encoding request")?;
            self.write(&request).await
        }
    }

    /// Receives a response from the device.
    fn receive<S: Service>(
        &self,
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
