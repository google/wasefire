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

#[cfg(feature = "board-api-crypto-p384-ecdh")]
use generic_array as _;
#[cfg(feature = "internal-board-api-crypto-ecdh")]
use wasefire_applet_api::crypto::ecdh::Kind;
use wasefire_applet_api::crypto::ecdh::{self as api, Api, Curve};
use wasefire_board_api::Api as Board;
#[cfg(feature = "internal-board-api-crypto-ecdh")]
use wasefire_board_api::crypto::ecdh::Api as _;
#[cfg(feature = "internal-board-api-crypto-ecdh")]
use wasefire_board_api::{self as board, Support};
use wasefire_error::{Code, Error};

#[cfg(feature = "internal-board-api-crypto-ecdh")]
use crate::applet::store::{Memory, MemoryApi};
use crate::{DispatchSchedulerCall, Failure, SchedulerCall};

pub fn process<B: Board>(call: Api<DispatchSchedulerCall<B>>) {
    match call {
        Api::IsSupported(call) => is_supported(call),
        Api::GetLayout(call) => or_fail!("internal-board-api-crypto-ecdh", get_layout(call)),
        Api::Generate(call) => or_fail!("internal-board-api-crypto-ecdh", generate(call)),
        Api::Public(call) => or_fail!("internal-board-api-crypto-ecdh", public(call)),
        Api::Shared(call) => or_fail!("internal-board-api-crypto-ecdh", shared(call)),
        Api::Drop(call) => or_fail!("internal-board-api-crypto-ecdh", drop(call)),
        Api::Export(call) => or_fail!("internal-board-api-crypto-ecdh", export(call)),
        Api::Import(call) => or_fail!("internal-board-api-crypto-ecdh", import(call)),
        Api::Access(call) => or_fail!("internal-board-api-crypto-ecdh", access(call)),
    }
}

fn is_supported<B: Board>(call: SchedulerCall<B, api::is_supported::Sig>) {
    let api::is_supported::Params { curve } = call.read();
    call.reply(try { convert_curve::<B>(*curve).is_ok() })
}

#[cfg(feature = "internal-board-api-crypto-ecdh")]
fn get_layout<B: Board>(mut call: SchedulerCall<B, api::get_layout::Sig>) {
    let api::get_layout::Params { curve, kind, size, align } = call.read();
    let memory = call.memory();
    let result = try {
        let layout = layout::<B>(convert_curve::<B>(*curve)?, convert_kind(*kind)?);
        memory.get_mut(*size, 4)?.copy_from_slice(&(layout.size() as u32).to_le_bytes());
        memory.get_mut(*align, 4)?.copy_from_slice(&(layout.align() as u32).to_le_bytes());
    };
    call.reply(result)
}

#[cfg(feature = "internal-board-api-crypto-ecdh")]
fn generate<B: Board>(mut call: SchedulerCall<B, api::generate::Sig>) {
    let api::generate::Params { curve, private } = call.read();
    let memory = call.memory();
    let result = try {
        let curve = convert_curve::<B>(*curve)?;
        let private = get_kind_mut::<B>(&memory, *private, curve, Kind::Private)?;
        match curve {
            #[cfg(feature = "board-api-crypto-p256-ecdh")]
            Curve::P256 => board::crypto::P256Ecdh::<B>::generate(private),
            #[cfg(feature = "board-api-crypto-p384-ecdh")]
            Curve::P384 => board::crypto::P384Ecdh::<B>::generate(private),
            #[allow(unreachable_patterns)]
            _ => trap_use!(),
        }?
    };
    call.reply(result)
}

#[cfg(feature = "internal-board-api-crypto-ecdh")]
fn public<B: Board>(mut call: SchedulerCall<B, api::public::Sig>) {
    let api::public::Params { curve, private, public } = call.read();
    let memory = call.memory();
    let result = try {
        let curve = convert_curve::<B>(*curve)?;
        let private = get_kind::<B>(&memory, *private, curve, Kind::Private)?;
        let public = get_kind_mut::<B>(&memory, *public, curve, Kind::Public)?;
        match curve {
            #[cfg(feature = "board-api-crypto-p256-ecdh")]
            Curve::P256 => board::crypto::P256Ecdh::<B>::public(private, public),
            #[cfg(feature = "board-api-crypto-p384-ecdh")]
            Curve::P384 => board::crypto::P384Ecdh::<B>::public(private, public),
            #[allow(unreachable_patterns)]
            _ => trap_use!(),
        }?
    };
    call.reply(result);
}

#[cfg(feature = "internal-board-api-crypto-ecdh")]
fn shared<B: Board>(mut call: SchedulerCall<B, api::shared::Sig>) {
    let api::shared::Params { curve, private, public, shared } = call.read();
    let memory = call.memory();
    let result = try {
        let curve = convert_curve::<B>(*curve)?;
        let private = get_kind::<B>(&memory, *private, curve, Kind::Private)?;
        let public = get_kind::<B>(&memory, *public, curve, Kind::Public)?;
        let shared = get_kind_mut::<B>(&memory, *shared, curve, Kind::Shared)?;
        match curve {
            #[cfg(feature = "board-api-crypto-p256-ecdh")]
            Curve::P256 => board::crypto::P256Ecdh::<B>::shared(private, public, shared),
            #[cfg(feature = "board-api-crypto-p384-ecdh")]
            Curve::P384 => board::crypto::P384Ecdh::<B>::shared(private, public, shared),
            #[allow(unreachable_patterns)]
            _ => trap_use!(),
        }?
    };
    call.reply(result);
}

#[cfg(feature = "internal-board-api-crypto-ecdh")]
fn drop<B: Board>(mut call: SchedulerCall<B, api::drop::Sig>) {
    let api::drop::Params { curve, kind, object } = call.read();
    let memory = call.memory();
    let result = try {
        let curve = convert_curve::<B>(*curve)?;
        let kind = convert_kind(*kind)?;
        Error::user(Code::InvalidArgument).check(matches!(kind, Kind::Private | Kind::Shared))?;
        let object = get_kind_mut::<B>(&memory, *object, curve, kind)?;
        match curve {
            #[cfg(feature = "board-api-crypto-p256-ecdh")]
            Curve::P256 => match kind {
                Kind::Private => board::crypto::P256Ecdh::<B>::drop_private(object),
                Kind::Public => unreachable!(),
                Kind::Shared => board::crypto::P256Ecdh::<B>::drop_shared(object),
            },
            #[cfg(feature = "board-api-crypto-p384-ecdh")]
            Curve::P384 => match kind {
                Kind::Private => board::crypto::P384Ecdh::<B>::drop_private(object),
                Kind::Public => unreachable!(),
                Kind::Shared => board::crypto::P384Ecdh::<B>::drop_shared(object),
            },
            #[allow(unreachable_patterns)]
            _ => trap_use!(),
        }?
    };
    call.reply(result);
}

#[cfg(feature = "internal-board-api-crypto-ecdh")]
fn export<B: Board>(mut call: SchedulerCall<B, api::export::Sig>) {
    let api::export::Params { curve, public, x, y } = call.read();
    let memory = call.memory();
    let result = try {
        let curve = convert_curve::<B>(*curve)?;
        let public = get_kind::<B>(&memory, *public, curve, Kind::Public)?;
        match curve {
            #[cfg(feature = "board-api-crypto-p256-ecdh")]
            Curve::P256 => {
                let x = memory.get_array_mut::<32>(*x)?;
                let y = memory.get_array_mut::<32>(*y)?;
                board::crypto::P256Ecdh::<B>::export_public(public, x, y)
            }
            #[cfg(feature = "board-api-crypto-p384-ecdh")]
            Curve::P384 => {
                let x = memory.get_array_mut::<48>(*x)?;
                let y = memory.get_array_mut::<48>(*y)?;
                board::crypto::P384Ecdh::<B>::export_public(public, x, y)
            }
            #[allow(unreachable_patterns)]
            _ => trap_use!(),
        }?
    };
    call.reply(result);
}

#[cfg(feature = "internal-board-api-crypto-ecdh")]
fn import<B: Board>(mut call: SchedulerCall<B, api::import::Sig>) {
    let api::import::Params { curve, x, y, public } = call.read();
    let memory = call.memory();
    let result = try {
        let curve = convert_curve::<B>(*curve)?;
        let public = get_kind_mut::<B>(&memory, *public, curve, Kind::Public)?;
        match curve {
            #[cfg(feature = "board-api-crypto-p256-ecdh")]
            Curve::P256 => {
                let x = memory.get_array::<32>(*x)?;
                let y = memory.get_array::<32>(*y)?;
                board::crypto::P256Ecdh::<B>::import_public(x, y, public)
            }
            #[cfg(feature = "board-api-crypto-p384-ecdh")]
            Curve::P384 => {
                let x = memory.get_array::<48>(*x)?;
                let y = memory.get_array::<48>(*y)?;
                board::crypto::P384Ecdh::<B>::import_public(x, y, public)
            }
            #[allow(unreachable_patterns)]
            _ => trap_use!(),
        }?
    };
    call.reply(result);
}

#[cfg(feature = "internal-board-api-crypto-ecdh")]
fn access<B: Board>(mut call: SchedulerCall<B, api::access::Sig>) {
    let api::access::Params { curve, shared, x } = call.read();
    let memory = call.memory();
    let result = try {
        let curve = convert_curve::<B>(*curve)?;
        let shared = get_kind::<B>(&memory, *shared, curve, Kind::Shared)?;
        match curve {
            #[cfg(feature = "board-api-crypto-p256-ecdh")]
            Curve::P256 => {
                let x = memory.get_array_mut::<32>(*x)?;
                board::crypto::P256Ecdh::<B>::export_shared(shared, x)
            }
            #[cfg(feature = "board-api-crypto-p384-ecdh")]
            Curve::P384 => {
                let x = memory.get_array_mut::<48>(*x)?;
                board::crypto::P384Ecdh::<B>::export_shared(shared, x)
            }
            #[allow(unreachable_patterns)]
            _ => trap_use!(),
        }?
    };
    call.reply(result);
}

#[allow(clippy::extra_unused_type_parameters)]
fn convert_curve<B: Board>(curve: u32) -> Result<Curve, Failure> {
    let curve = Curve::try_from(curve).map_err(|_| Error::user(Code::InvalidArgument))?;
    let support = match curve {
        Curve::P256 => {
            or_false!("board-api-crypto-p256-ecdh", board::crypto::P256Ecdh::<B>::SUPPORT)
        }
        Curve::P384 => {
            or_false!("board-api-crypto-p384-ecdh", board::crypto::P384Ecdh::<B>::SUPPORT)
        }
    };
    if support { Ok(curve) } else { Err(Error::world(Code::NotImplemented).into()) }
}

#[cfg(feature = "internal-board-api-crypto-ecdh")]
fn convert_kind(kind: u32) -> Result<Kind, Failure> {
    Kind::try_from(kind).map_err(|_| Error::user(Code::InvalidArgument).into())
}

#[cfg(feature = "internal-board-api-crypto-ecdh")]
fn layout<B: Board>(curve: Curve, kind: Kind) -> core::alloc::Layout {
    match curve {
        #[cfg(feature = "board-api-crypto-p256-ecdh")]
        Curve::P256 => match kind {
            Kind::Private => board::crypto::P256Ecdh::<B>::PRIVATE,
            Kind::Public => board::crypto::P256Ecdh::<B>::PUBLIC,
            Kind::Shared => board::crypto::P256Ecdh::<B>::SHARED,
        },
        #[cfg(feature = "board-api-crypto-p384-ecdh")]
        Curve::P384 => match kind {
            Kind::Private => board::crypto::P384Ecdh::<B>::PRIVATE,
            Kind::Public => board::crypto::P384Ecdh::<B>::PUBLIC,
            Kind::Shared => board::crypto::P384Ecdh::<B>::SHARED,
        },
        #[allow(unreachable_patterns)]
        _ => unreachable!(),
    }
}

#[cfg(feature = "internal-board-api-crypto-ecdh")]
fn get_kind<'a, B: Board>(
    memory: &'a Memory<'a>, ptr: u32, curve: Curve, kind: Kind,
) -> Result<&'a [u8], Failure> {
    let layout = layout::<B>(curve, kind);
    Ok(memory.get(ptr, layout.size() as u32)?)
}

#[cfg(feature = "internal-board-api-crypto-ecdh")]
fn get_kind_mut<'a, B: Board>(
    memory: &'a Memory<'a>, ptr: u32, curve: Curve, kind: Kind,
) -> Result<&'a mut [u8], Failure> {
    let layout = layout::<B>(curve, kind);
    Ok(memory.get_mut(ptr, layout.size() as u32)?)
}
