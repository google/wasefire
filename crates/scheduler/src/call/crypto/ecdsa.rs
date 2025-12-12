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

#[cfg(feature = "board-api-crypto-p384-ecdsa")]
use generic_array as _;
#[cfg(feature = "internal-board-api-crypto-ecdsa")]
use wasefire_applet_api::crypto::ecdsa::Kind;
use wasefire_applet_api::crypto::ecdsa::{self as api, Api, Curve};
use wasefire_board_api::Api as Board;
#[cfg(feature = "internal-board-api-crypto-ecdsa")]
use wasefire_board_api::applet::{Memory as _, MemoryExt as _};
#[cfg(feature = "internal-board-api-crypto-ecdsa")]
use wasefire_board_api::crypto::ecdsa::Api as _;
#[cfg(feature = "internal-board-api-crypto-ecdsa")]
use wasefire_board_api::{self as board, Support};
use wasefire_error::{Code, Error};

#[cfg(feature = "internal-board-api-crypto-ecdsa")]
use crate::applet::store::Memory;
use crate::{DispatchSchedulerCall, Failure, SchedulerCall};

pub fn process<B: Board>(call: Api<DispatchSchedulerCall<B>>) {
    match call {
        Api::IsSupported(call) => is_supported(call),
        Api::GetLayout(call) => or_fail!("internal-board-api-crypto-ecdsa", get_layout(call)),
        Api::WrappedLength(call) => {
            or_fail!("internal-board-api-crypto-ecdsa", wrapped_length(call))
        }
        Api::Generate(call) => or_fail!("internal-board-api-crypto-ecdsa", generate(call)),
        Api::Public(call) => or_fail!("internal-board-api-crypto-ecdsa", public(call)),
        Api::Sign(call) => or_fail!("internal-board-api-crypto-ecdsa", sign(call)),
        Api::Verify(call) => or_fail!("internal-board-api-crypto-ecdsa", verify(call)),
        Api::Drop(call) => or_fail!("internal-board-api-crypto-ecdsa", drop(call)),
        Api::Wrap(call) => or_fail!("internal-board-api-crypto-ecdsa", wrap(call)),
        Api::Unwrap(call) => or_fail!("internal-board-api-crypto-ecdsa", unwrap(call)),
        Api::Export(call) => or_fail!("internal-board-api-crypto-ecdsa", export(call)),
        Api::Import(call) => or_fail!("internal-board-api-crypto-ecdsa", import(call)),
    }
}

fn is_supported<B: Board>(call: SchedulerCall<B, api::is_supported::Sig>) {
    let api::is_supported::Params { curve } = call.read();
    call.reply(try { convert_curve::<B>(*curve).is_ok() })
}

#[cfg(feature = "internal-board-api-crypto-ecdsa")]
fn get_layout<B: Board>(mut call: SchedulerCall<B, api::get_layout::Sig>) {
    let api::get_layout::Params { curve, kind, size, align } = call.read();
    let memory = call.memory();
    let result = try bikeshed _ {
        let layout = layout::<B>(convert_curve::<B>(*curve)?, convert_kind(*kind)?);
        memory.get_mut(*size, 4)?.copy_from_slice(&(layout.size() as u32).to_le_bytes());
        memory.get_mut(*align, 4)?.copy_from_slice(&(layout.align() as u32).to_le_bytes());
    };
    call.reply(result)
}

#[cfg(feature = "internal-board-api-crypto-ecdsa")]
fn wrapped_length<B: Board>(call: SchedulerCall<B, api::wrapped_length::Sig>) {
    let api::wrapped_length::Params { curve } = call.read();
    let result = try bikeshed _ {
        let length = match convert_curve::<B>(*curve)? {
            #[cfg(feature = "board-api-crypto-p256-ecdsa")]
            Curve::P256 => board::crypto::P256Ecdsa::<B>::WRAPPED,
            #[cfg(feature = "board-api-crypto-p384-ecdsa")]
            Curve::P384 => board::crypto::P384Ecdsa::<B>::WRAPPED,
            #[allow(unreachable_patterns)]
            _ => trap_use!(),
        };
        length as u32
    };
    call.reply(result)
}

#[cfg(feature = "internal-board-api-crypto-ecdsa")]
fn generate<B: Board>(mut call: SchedulerCall<B, api::generate::Sig>) {
    let api::generate::Params { curve, private } = call.read();
    let memory = call.memory();
    let result = try bikeshed _ {
        let curve = convert_curve::<B>(*curve)?;
        let private = get_kind_mut::<B>(&memory, *private, curve, Kind::Private)?;
        match curve {
            #[cfg(feature = "board-api-crypto-p256-ecdsa")]
            Curve::P256 => board::crypto::P256Ecdsa::<B>::generate(private),
            #[cfg(feature = "board-api-crypto-p384-ecdsa")]
            Curve::P384 => board::crypto::P384Ecdsa::<B>::generate(private),
            #[allow(unreachable_patterns)]
            _ => trap_use!(),
        }?
    };
    call.reply(result)
}

#[cfg(feature = "internal-board-api-crypto-ecdsa")]
fn public<B: Board>(mut call: SchedulerCall<B, api::public::Sig>) {
    let api::public::Params { curve, private, public } = call.read();
    let memory = call.memory();
    let result = try bikeshed _ {
        let curve = convert_curve::<B>(*curve)?;
        let private = get_kind::<B>(&memory, *private, curve, Kind::Private)?;
        let public = get_kind_mut::<B>(&memory, *public, curve, Kind::Public)?;
        match curve {
            #[cfg(feature = "board-api-crypto-p256-ecdsa")]
            Curve::P256 => board::crypto::P256Ecdsa::<B>::public(private, public),
            #[cfg(feature = "board-api-crypto-p384-ecdsa")]
            Curve::P384 => board::crypto::P384Ecdsa::<B>::public(private, public),
            #[allow(unreachable_patterns)]
            _ => trap_use!(),
        }?
    };
    call.reply(result);
}

#[cfg(feature = "internal-board-api-crypto-ecdsa")]
fn sign<B: Board>(mut call: SchedulerCall<B, api::sign::Sig>) {
    let api::sign::Params { curve, private, digest, r, s } = call.read();
    let memory = call.memory();
    let result = try bikeshed _ {
        let curve = convert_curve::<B>(*curve)?;
        let private = get_kind::<B>(&memory, *private, curve, Kind::Private)?;
        match curve {
            #[cfg(feature = "board-api-crypto-p256-ecdsa")]
            Curve::P256 => {
                let digest = memory.get_array::<32>(*digest)?;
                let r = memory.get_array_mut::<32>(*r)?;
                let s = memory.get_array_mut::<32>(*s)?;
                board::crypto::P256Ecdsa::<B>::sign(private, digest, r, s)
            }
            #[cfg(feature = "board-api-crypto-p384-ecdsa")]
            Curve::P384 => {
                let digest = memory.get_array::<48>(*digest)?;
                let r = memory.get_array_mut::<48>(*r)?;
                let s = memory.get_array_mut::<48>(*s)?;
                board::crypto::P384Ecdsa::<B>::sign(private, digest, r, s)
            }
            #[allow(unreachable_patterns)]
            _ => trap_use!(),
        }?
    };
    call.reply(result);
}

#[cfg(feature = "internal-board-api-crypto-ecdsa")]
fn verify<B: Board>(mut call: SchedulerCall<B, api::verify::Sig>) {
    let api::verify::Params { curve, public, digest, r, s } = call.read();
    let memory = call.memory();
    let result = try bikeshed _ {
        let curve = convert_curve::<B>(*curve)?;
        let public = get_kind::<B>(&memory, *public, curve, Kind::Public)?;
        match curve {
            #[cfg(feature = "board-api-crypto-p256-ecdsa")]
            Curve::P256 => {
                let digest = memory.get_array::<32>(*digest)?;
                let r = memory.get_array::<32>(*r)?;
                let s = memory.get_array::<32>(*s)?;
                board::crypto::P256Ecdsa::<B>::verify(public, digest, r, s)
            }
            #[cfg(feature = "board-api-crypto-p384-ecdsa")]
            Curve::P384 => {
                let digest = memory.get_array::<48>(*digest)?;
                let r = memory.get_array::<48>(*r)?;
                let s = memory.get_array::<48>(*s)?;
                board::crypto::P384Ecdsa::<B>::verify(public, digest, r, s)
            }
            #[allow(unreachable_patterns)]
            _ => trap_use!(),
        }?
    };
    call.reply(result);
}

#[cfg(feature = "internal-board-api-crypto-ecdsa")]
fn drop<B: Board>(mut call: SchedulerCall<B, api::drop::Sig>) {
    let api::drop::Params { curve, private } = call.read();
    let memory = call.memory();
    let result = try bikeshed _ {
        let curve = convert_curve::<B>(*curve)?;
        let private = get_kind_mut::<B>(&memory, *private, curve, Kind::Private)?;
        match curve {
            #[cfg(feature = "board-api-crypto-p256-ecdsa")]
            Curve::P256 => board::crypto::P256Ecdsa::<B>::drop_private(private),
            #[cfg(feature = "board-api-crypto-p384-ecdsa")]
            Curve::P384 => board::crypto::P384Ecdsa::<B>::drop_private(private),
            #[allow(unreachable_patterns)]
            _ => trap_use!(),
        }?
    };
    call.reply(result);
}

#[cfg(feature = "internal-board-api-crypto-ecdsa")]
fn wrap<B: Board>(mut call: SchedulerCall<B, api::wrap::Sig>) {
    let api::wrap::Params { curve, private, wrapped } = call.read();
    let memory = call.memory();
    let result = try bikeshed _ {
        let curve = convert_curve::<B>(*curve)?;
        let private = get_kind::<B>(&memory, *private, curve, Kind::Private)?;
        match curve {
            #[cfg(feature = "board-api-crypto-p256-ecdsa")]
            Curve::P256 => {
                let wrapped =
                    memory.get_mut(*wrapped, board::crypto::P256Ecdsa::<B>::WRAPPED as u32)?;
                board::crypto::P256Ecdsa::<B>::export_private(private, wrapped)
            }
            #[cfg(feature = "board-api-crypto-p384-ecdsa")]
            Curve::P384 => {
                let wrapped =
                    memory.get_mut(*wrapped, board::crypto::P384Ecdsa::<B>::WRAPPED as u32)?;
                board::crypto::P384Ecdsa::<B>::export_private(private, wrapped)
            }
            #[allow(unreachable_patterns)]
            _ => trap_use!(),
        }?
    };
    call.reply(result);
}

#[cfg(feature = "internal-board-api-crypto-ecdsa")]
fn unwrap<B: Board>(mut call: SchedulerCall<B, api::unwrap::Sig>) {
    let api::unwrap::Params { curve, wrapped, private } = call.read();
    let memory = call.memory();
    let result = try bikeshed _ {
        let curve = convert_curve::<B>(*curve)?;
        let private = get_kind_mut::<B>(&memory, *private, curve, Kind::Private)?;
        match curve {
            #[cfg(feature = "board-api-crypto-p256-ecdsa")]
            Curve::P256 => {
                let wrapped =
                    memory.get(*wrapped, board::crypto::P256Ecdsa::<B>::WRAPPED as u32)?;
                board::crypto::P256Ecdsa::<B>::import_private(wrapped, private)
            }
            #[cfg(feature = "board-api-crypto-p384-ecdsa")]
            Curve::P384 => {
                let wrapped =
                    memory.get(*wrapped, board::crypto::P384Ecdsa::<B>::WRAPPED as u32)?;
                board::crypto::P384Ecdsa::<B>::import_private(wrapped, private)
            }
            #[allow(unreachable_patterns)]
            _ => trap_use!(),
        }?
    };
    call.reply(result);
}

#[cfg(feature = "internal-board-api-crypto-ecdsa")]
fn export<B: Board>(mut call: SchedulerCall<B, api::export::Sig>) {
    let api::export::Params { curve, public, x, y } = call.read();
    let memory = call.memory();
    let result = try bikeshed _ {
        let curve = convert_curve::<B>(*curve)?;
        let public = get_kind::<B>(&memory, *public, curve, Kind::Public)?;
        match curve {
            #[cfg(feature = "board-api-crypto-p256-ecdsa")]
            Curve::P256 => {
                let x = memory.get_array_mut::<32>(*x)?;
                let y = memory.get_array_mut::<32>(*y)?;
                board::crypto::P256Ecdsa::<B>::export_public(public, x, y)
            }
            #[cfg(feature = "board-api-crypto-p384-ecdsa")]
            Curve::P384 => {
                let x = memory.get_array_mut::<48>(*x)?;
                let y = memory.get_array_mut::<48>(*y)?;
                board::crypto::P384Ecdsa::<B>::export_public(public, x, y)
            }
            #[allow(unreachable_patterns)]
            _ => trap_use!(),
        }?
    };
    call.reply(result);
}

#[cfg(feature = "internal-board-api-crypto-ecdsa")]
fn import<B: Board>(mut call: SchedulerCall<B, api::import::Sig>) {
    let api::import::Params { curve, x, y, public } = call.read();
    let memory = call.memory();
    let result = try bikeshed _ {
        let curve = convert_curve::<B>(*curve)?;
        let public = get_kind_mut::<B>(&memory, *public, curve, Kind::Public)?;
        match curve {
            #[cfg(feature = "board-api-crypto-p256-ecdsa")]
            Curve::P256 => {
                let x = memory.get_array::<32>(*x)?;
                let y = memory.get_array::<32>(*y)?;
                board::crypto::P256Ecdsa::<B>::import_public(x, y, public)
            }
            #[cfg(feature = "board-api-crypto-p384-ecdsa")]
            Curve::P384 => {
                let x = memory.get_array::<48>(*x)?;
                let y = memory.get_array::<48>(*y)?;
                board::crypto::P384Ecdsa::<B>::import_public(x, y, public)
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
            or_false!("board-api-crypto-p256-ecdsa", board::crypto::P256Ecdsa::<B>::SUPPORT)
        }
        Curve::P384 => {
            or_false!("board-api-crypto-p384-ecdsa", board::crypto::P384Ecdsa::<B>::SUPPORT)
        }
    };
    if support { Ok(curve) } else { Err(Error::world(Code::NotImplemented).into()) }
}

#[cfg(feature = "internal-board-api-crypto-ecdsa")]
fn convert_kind(kind: u32) -> Result<Kind, Failure> {
    Kind::try_from(kind).map_err(|_| Error::user(Code::InvalidArgument).into())
}

#[cfg(feature = "internal-board-api-crypto-ecdsa")]
fn layout<B: Board>(curve: Curve, kind: Kind) -> core::alloc::Layout {
    match curve {
        #[cfg(feature = "board-api-crypto-p256-ecdsa")]
        Curve::P256 => match kind {
            Kind::Private => board::crypto::P256Ecdsa::<B>::PRIVATE,
            Kind::Public => board::crypto::P256Ecdsa::<B>::PUBLIC,
        },
        #[cfg(feature = "board-api-crypto-p384-ecdsa")]
        Curve::P384 => match kind {
            Kind::Private => board::crypto::P384Ecdsa::<B>::PRIVATE,
            Kind::Public => board::crypto::P384Ecdsa::<B>::PUBLIC,
        },
        #[allow(unreachable_patterns)]
        _ => unreachable!(),
    }
}

#[cfg(feature = "internal-board-api-crypto-ecdsa")]
fn get_kind<'a, B: Board>(
    memory: &'a Memory<'a>, ptr: u32, curve: Curve, kind: Kind,
) -> Result<&'a [u8], Failure> {
    let layout = layout::<B>(curve, kind);
    Ok(memory.get(ptr, layout.size() as u32)?)
}

#[cfg(feature = "internal-board-api-crypto-ecdsa")]
fn get_kind_mut<'a, B: Board>(
    memory: &'a Memory<'a>, ptr: u32, curve: Curve, kind: Kind,
) -> Result<&'a mut [u8], Failure> {
    let layout = layout::<B>(curve, kind);
    Ok(memory.get_mut(ptr, layout.size() as u32)?)
}
