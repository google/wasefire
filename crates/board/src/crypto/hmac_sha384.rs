// Copyright 2023 Google LLC
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

//! HMAC-SHA-384 interface.

use core::fmt::Debug;

use crate::{Error, Unimplemented, Unsupported};

/// Returns this [`Types`] given a [`crate::Types`].
pub type Get<B> = <super::Get<B> as super::Types>::HmacSha384;

/// Returns the [`Types::Context`] associated type given a [`crate::Types`].
pub type Context<B> = <Get<B> as Types>::Context;

/// Associated types of [`Api`].
pub trait Types {
    /// Hmac context.
    type Context: Debug;
}

/// HMAC-SHA-384 interface.
pub trait Api<T: Types> {
    /// Whether HMAC-SHA-384 is supported.
    fn is_supported(&mut self) -> bool;

    /// Creates a new hmac context.
    fn initialize(&mut self, key: &[u8]) -> Result<T::Context, Error>;

    /// Updates a hmac context.
    fn update(&mut self, context: &mut T::Context, data: &[u8]) -> Result<(), Error>;

    /// Finalizes an hmac.
    fn finalize(&mut self, context: T::Context, hmac: &mut [u8; 48]) -> Result<(), Error>;
}

impl Types for Unimplemented {
    type Context = Unimplemented;
}

impl Api<Unimplemented> for Unimplemented {
    fn is_supported(&mut self) -> bool {
        unreachable!()
    }

    fn initialize(&mut self, _: &[u8]) -> Result<Unimplemented, Error> {
        unreachable!()
    }

    fn update(&mut self, _: &mut Unimplemented, _: &[u8]) -> Result<(), Error> {
        unreachable!()
    }

    fn finalize(&mut self, _: Unimplemented, _: &mut [u8; 48]) -> Result<(), Error> {
        unreachable!()
    }
}

#[cfg(not(feature = "software-crypto-hmac-sha384"))]
mod unsupported {
    use super::*;

    impl Types for Unsupported {
        type Context = Unsupported;
    }

    impl Api<Unsupported> for Unsupported {
        fn is_supported(&mut self) -> bool {
            false
        }

        fn initialize(&mut self, _: &[u8]) -> Result<Unsupported, Error> {
            Err(Error::User)
        }

        fn update(&mut self, _: &mut Unsupported, _: &[u8]) -> Result<(), Error> {
            Err(Error::User)
        }

        fn finalize(&mut self, _: Unsupported, _: &mut [u8; 48]) -> Result<(), Error> {
            Err(Error::User)
        }
    }
}

#[cfg(feature = "software-crypto-hmac-sha384")]
mod unsupported {
    use hmac::digest::{FixedOutput, KeyInit, Update};
    use hmac::Hmac;
    use sha2::Sha384;

    use super::*;

    impl Types for Unsupported {
        type Context = Hmac<Sha384>;
    }

    impl Api<Unsupported> for Unsupported {
        fn is_supported(&mut self) -> bool {
            true
        }

        fn initialize(&mut self, key: &[u8]) -> Result<Hmac<Sha384>, Error> {
            Hmac::new_from_slice(key).map_err(|_| Error::World)
        }

        fn update(&mut self, context: &mut Hmac<Sha384>, data: &[u8]) -> Result<(), Error> {
            context.update(data);
            Ok(())
        }

        fn finalize(&mut self, context: Hmac<Sha384>, hmac: &mut [u8; 48]) -> Result<(), Error> {
            context.finalize_into(hmac.into());
            Ok(())
        }
    }
}
