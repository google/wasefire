use alloc::vec::Vec;

use opensk_lib::api::crypto::Crypto;
use opensk_lib::api::key_store::{CredentialSource, KeyStore};
use opensk_lib::ctap::secret::Secret;
use opensk_lib::env::Env;

use crate::WasefireEnv;

impl KeyStore for WasefireEnv {
    fn init(&mut self) -> Result<(), opensk_lib::api::key_store::Error> {
        todo!()
    }

    fn wrap_key<E>(
        &mut self,
    ) -> Result<<<E as Env>::Crypto as Crypto>::Aes256, opensk_lib::api::key_store::Error>
    where E: Env {
        todo!()
    }

    fn wrap_credential(
        &mut self, _: CredentialSource,
    ) -> Result<Vec<u8>, opensk_lib::api::key_store::Error> {
        todo!()
    }

    fn unwrap_credential(
        &mut self, _: &[u8], _: &[u8],
    ) -> Result<Option<CredentialSource>, opensk_lib::api::key_store::Error> {
        todo!()
    }

    fn cred_random(
        &mut self, _: bool,
    ) -> Result<Secret<[u8; 32]>, opensk_lib::api::key_store::Error> {
        todo!()
    }

    fn encrypt_pin_hash(
        &mut self, _: &[u8; 16],
    ) -> Result<[u8; 16], opensk_lib::api::key_store::Error> {
        todo!()
    }

    fn decrypt_pin_hash(
        &mut self, _: &[u8; 16],
    ) -> Result<Secret<[u8; 16]>, opensk_lib::api::key_store::Error> {
        todo!()
    }

    fn reset(&mut self) -> Result<(), opensk_lib::api::key_store::Error> {
        todo!()
    }
}
