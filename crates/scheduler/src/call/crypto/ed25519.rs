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

#[cfg(feature = "board-api-crypto-ed25519")]
use wasefire_applet_api::crypto::ed25519::Kind;
use wasefire_applet_api::crypto::ed25519::{self as api, Api};
use wasefire_board_api::Api as Board;
#[cfg(feature = "board-api-crypto-ed25519")]
use wasefire_board_api::applet::{Memory as _, MemoryExt as _};
#[cfg(feature = "board-api-crypto-ed25519")]
use wasefire_board_api::crypto::ed25519::Api as _;
#[cfg(feature = "board-api-crypto-ed25519")]
use wasefire_board_api::{self as board, Support};
#[cfg(feature = "board-api-crypto-ed25519")]
use wasefire_error::{Code, Error};

#[cfg(feature = "board-api-crypto-ed25519")]
use crate::Failure;
#[cfg(feature = "board-api-crypto-ed25519")]
use crate::applet::store::Memory;
use crate::{DispatchSchedulerCall, SchedulerCall};

pub fn process<B: Board>(call: Api<DispatchSchedulerCall<B>>) {
    match call {
        Api::IsSupported(call) => is_supported(call),
        Api::GetLayout(call) => or_fail!("board-api-crypto-ed25519", get_layout(call)),
        Api::WrappedLength(call) => {
            or_fail!("board-api-crypto-ed25519", wrapped_length(call))
        }
        Api::Generate(call) => or_fail!("board-api-crypto-ed25519", generate(call)),
        Api::Public(call) => or_fail!("board-api-crypto-ed25519", public(call)),
        Api::Sign(call) => or_fail!("board-api-crypto-ed25519", sign(call)),
        Api::Verify(call) => or_fail!("board-api-crypto-ed25519", verify(call)),
        Api::Drop(call) => or_fail!("board-api-crypto-ed25519", drop(call)),
        Api::Wrap(call) => or_fail!("board-api-crypto-ed25519", wrap(call)),
        Api::Unwrap(call) => or_fail!("board-api-crypto-ed25519", unwrap(call)),
        Api::Export(call) => or_fail!("board-api-crypto-ed25519", export(call)),
        Api::Import(call) => or_fail!("board-api-crypto-ed25519", import(call)),
    }
}

fn is_supported<B: Board>(call: SchedulerCall<B, api::is_supported::Sig>) {
    let api::is_supported::Params {} = call.read();
    let result = or_false!("board-api-crypto-ed25519", board::crypto::Ed25519::<B>::SUPPORT);
    call.reply(Ok(result))
}

#[cfg(feature = "board-api-crypto-ed25519")]
fn get_layout<B: Board>(mut call: SchedulerCall<B, api::get_layout::Sig>) {
    let api::get_layout::Params { kind, size, align } = call.read();
    let memory = call.memory();
    let result = try {
        let layout = layout::<B>(convert_kind(*kind)?);
        memory.get_mut(*size, 4)?.copy_from_slice(&(layout.size() as u32).to_le_bytes());
        memory.get_mut(*align, 4)?.copy_from_slice(&(layout.align() as u32).to_le_bytes());
    };
    call.reply(result)
}

#[cfg(feature = "board-api-crypto-ed25519")]
fn wrapped_length<B: Board>(call: SchedulerCall<B, api::wrapped_length::Sig>) {
    let api::wrapped_length::Params {} = call.read();
    call.reply(Ok(board::crypto::Ed25519::<B>::WRAPPED as u32))
}

#[cfg(feature = "board-api-crypto-ed25519")]
fn generate<B: Board>(mut call: SchedulerCall<B, api::generate::Sig>) {
    let api::generate::Params { private } = call.read();
    let memory = call.memory();
    let result = try {
        let private = get_kind_mut::<B>(&memory, *private, Kind::Private)?;
        board::crypto::Ed25519::<B>::generate(private)?;
    };
    call.reply(result)
}

#[cfg(feature = "board-api-crypto-ed25519")]
fn public<B: Board>(mut call: SchedulerCall<B, api::public::Sig>) {
    let api::public::Params { private, public } = call.read();
    let memory = call.memory();
    let result = try {
        let private = get_kind::<B>(&memory, *private, Kind::Private)?;
        let public = get_kind_mut::<B>(&memory, *public, Kind::Public)?;
        board::crypto::Ed25519::<B>::public(private, public)?;
    };
    call.reply(result);
}

#[cfg(feature = "board-api-crypto-ed25519")]
fn sign<B: Board>(mut call: SchedulerCall<B, api::sign::Sig>) {
    let api::sign::Params { private, message, message_len, r, s } = call.read();
    let memory = call.memory();
    let result = try {
        let private = get_kind::<B>(&memory, *private, Kind::Private)?;
        let message = memory.get(*message, *message_len)?;
        let r = memory.get_array_mut::<32>(*r)?;
        let s = memory.get_array_mut::<32>(*s)?;
        board::crypto::Ed25519::<B>::sign(private, message, r, s)?;
    };
    call.reply(result);
}

#[cfg(feature = "board-api-crypto-ed25519")]
fn verify<B: Board>(mut call: SchedulerCall<B, api::verify::Sig>) {
    let api::verify::Params { public, message, message_len, r, s } = call.read();
    let memory = call.memory();
    let result = try {
        let public = get_kind::<B>(&memory, *public, Kind::Public)?;
        let message = memory.get(*message, *message_len)?;
        let r = memory.get_array::<32>(*r)?;
        let s = memory.get_array::<32>(*s)?;
        board::crypto::Ed25519::<B>::verify(public, message, r, s)?
    };
    call.reply(result);
}

#[cfg(feature = "board-api-crypto-ed25519")]
fn drop<B: Board>(mut call: SchedulerCall<B, api::drop::Sig>) {
    let api::drop::Params { private } = call.read();
    let memory = call.memory();
    let result = try {
        let private = get_kind_mut::<B>(&memory, *private, Kind::Private)?;
        board::crypto::Ed25519::<B>::drop_private(private)?;
    };
    call.reply(result);
}

#[cfg(feature = "board-api-crypto-ed25519")]
fn wrap<B: Board>(mut call: SchedulerCall<B, api::wrap::Sig>) {
    let api::wrap::Params { private, wrapped } = call.read();
    let memory = call.memory();
    let result = try {
        let private = get_kind::<B>(&memory, *private, Kind::Private)?;
        let wrapped = memory.get_mut(*wrapped, board::crypto::Ed25519::<B>::WRAPPED as u32)?;
        board::crypto::Ed25519::<B>::export_private(private, wrapped)?;
    };
    call.reply(result);
}

#[cfg(feature = "board-api-crypto-ed25519")]
fn unwrap<B: Board>(mut call: SchedulerCall<B, api::unwrap::Sig>) {
    let api::unwrap::Params { wrapped, private } = call.read();
    let memory = call.memory();
    let result = try {
        let private = get_kind_mut::<B>(&memory, *private, Kind::Private)?;
        let wrapped = memory.get(*wrapped, board::crypto::Ed25519::<B>::WRAPPED as u32)?;
        board::crypto::Ed25519::<B>::import_private(wrapped, private)?;
    };
    call.reply(result);
}

#[cfg(feature = "board-api-crypto-ed25519")]
fn export<B: Board>(mut call: SchedulerCall<B, api::export::Sig>) {
    let api::export::Params { public, a } = call.read();
    let memory = call.memory();
    let result = try {
        let public = get_kind::<B>(&memory, *public, Kind::Public)?;
        let a = memory.get_array_mut::<32>(*a)?;
        board::crypto::Ed25519::<B>::export_public(public, a)?;
    };
    call.reply(result);
}

#[cfg(feature = "board-api-crypto-ed25519")]
fn import<B: Board>(mut call: SchedulerCall<B, api::import::Sig>) {
    let api::import::Params { a, public } = call.read();
    let memory = call.memory();
    let result = try {
        let public = get_kind_mut::<B>(&memory, *public, Kind::Public)?;
        let a = memory.get_array::<32>(*a)?;
        board::crypto::Ed25519::<B>::import_public(a, public)?;
    };
    call.reply(result);
}

#[cfg(feature = "board-api-crypto-ed25519")]
fn convert_kind(kind: u32) -> Result<Kind, Failure> {
    Kind::try_from(kind).map_err(|_| Error::user(Code::InvalidArgument).into())
}

#[cfg(feature = "board-api-crypto-ed25519")]
fn layout<B: Board>(kind: Kind) -> core::alloc::Layout {
    match kind {
        Kind::Private => board::crypto::Ed25519::<B>::PRIVATE,
        Kind::Public => board::crypto::Ed25519::<B>::PUBLIC,
    }
}

#[cfg(feature = "board-api-crypto-ed25519")]
fn get_kind<'a, B: Board>(
    memory: &'a Memory<'a>, ptr: u32, kind: Kind,
) -> Result<&'a [u8], Failure> {
    let layout = layout::<B>(kind);
    Ok(memory.get(ptr, layout.size() as u32)?)
}

#[cfg(feature = "board-api-crypto-ed25519")]
fn get_kind_mut<'a, B: Board>(
    memory: &'a Memory<'a>, ptr: u32, kind: Kind,
) -> Result<&'a mut [u8], Failure> {
    let layout = layout::<B>(kind);
    Ok(memory.get_mut(ptr, layout.size() as u32)?)
}
