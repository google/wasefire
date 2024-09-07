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

//! Message format between device and host.

use std::io::ErrorKind;

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use wasefire_logger as log;

pub(crate) async fn read<S: AsyncRead + Unpin>(stream: &mut S) -> Result<Box<[u8]>, ()> {
    let mut length = [0; 4];
    read_exact(stream, &mut length).await?;
    let length = u32::from_be_bytes(length);
    let mut message = vec![0; length as usize];
    read_exact(stream, &mut message).await?;
    Ok(message.into_boxed_slice())
}

pub(crate) async fn write<S: AsyncWrite + Unpin>(stream: &mut S, message: &[u8]) -> Result<(), ()> {
    let length = message.len() as u32;
    write_all(stream, &length.to_be_bytes()).await?;
    write_all(stream, message).await?;
    Ok(())
}

async fn read_exact<S: AsyncRead + Unpin>(stream: &mut S, buffer: &mut [u8]) -> Result<(), ()> {
    match stream.read_exact(buffer).await {
        Ok(_) => Ok(()),
        Err(x) => match x.kind() {
            ErrorKind::NotConnected | ErrorKind::BrokenPipe | ErrorKind::UnexpectedEof => {
                log::debug!("read_exact {:?}", x.kind());
                Err(())
            }
            _ => panic!("{x}"),
        },
    }
}

async fn write_all<S: AsyncWrite + Unpin>(stream: &mut S, buffer: &[u8]) -> Result<(), ()> {
    match stream.write_all(buffer).await {
        Ok(_) => Ok(()),
        Err(x) => match x.kind() {
            ErrorKind::NotConnected | ErrorKind::BrokenPipe => {
                log::debug!("write_all {:?}", x.kind());
                Err(())
            }
            _ => panic!("{x}"),
        },
    }
}
