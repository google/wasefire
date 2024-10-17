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

use std::io::Result;

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub(crate) async fn read<S: AsyncRead + Unpin>(stream: &mut S) -> Result<Box<[u8]>> {
    let mut length = [0; 4];
    stream.read_exact(&mut length).await?;
    let length = u32::from_be_bytes(length);
    let mut message = vec![0; length as usize];
    stream.read_exact(&mut message).await?;
    Ok(message.into_boxed_slice())
}

pub(crate) async fn write<S: AsyncWrite + Unpin>(stream: &mut S, message: &[u8]) -> Result<()> {
    let length = message.len() as u32;
    stream.write_all(&length.to_be_bytes()).await?;
    stream.write_all(message).await?;
    Ok(())
}
