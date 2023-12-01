use core::num::NonZeroU32;

use opensk_lib::api::rng::Rng;
use rand_core::{impls, CryptoRng, Error, RngCore};
use wasefire::*;

use crate::WasefireEnv;

impl CryptoRng for WasefireEnv {}

impl RngCore for WasefireEnv {
    fn next_u32(&mut self) -> u32 {
        impls::next_u32_via_fill(self)
    }

    fn next_u64(&mut self) -> u64 {
        impls::next_u64_via_fill(self)
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.try_fill_bytes(dest).unwrap()
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Error> {
        match rng::fill_bytes(dest) {
            Ok(()) => Ok(()),
            Err(_) => Err(NonZeroU32::new(rand_core::Error::CUSTOM_START).unwrap().into()),
        }
    }
}

impl Rng for WasefireEnv {}
