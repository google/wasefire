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

use std::net::SocketAddr;
use std::path::Path;

use anyhow::Result;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::{TcpStream, UnixStream};
use wasefire_protocol::DynFuture;

use crate::common::{read, write};

/// Holds a connection to a device.
pub struct Connection<S: AsyncRead + AsyncWrite + Send + Unpin> {
    stream: S,
}

impl Connection<UnixStream> {
    pub async fn new_unix(path: &Path) -> Result<Self> {
        let stream = UnixStream::connect(path).await?;
        Ok(Connection { stream })
    }
}

impl Connection<TcpStream> {
    pub async fn new_tcp(addr: SocketAddr) -> Result<Self> {
        let stream = TcpStream::connect(addr).await?;
        Ok(Connection { stream })
    }
}

impl<T: AsyncRead + AsyncWrite + Send + Unpin + 'static> wasefire_protocol::Connection
    for Connection<T>
{
    fn read(&mut self) -> DynFuture<Box<[u8]>> {
        Box::pin(async move { Ok(read(&mut self.stream).await?) })
    }

    fn write<'a>(&'a mut self, request: &'a [u8]) -> DynFuture<'a, ()> {
        Box::pin(async move { Ok(write(&mut self.stream, request).await?) })
    }
}
