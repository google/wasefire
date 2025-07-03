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

//! Cipher block chaining (CBC).

use crypto_common::generic_array::{ArrayLength, GenericArray};
#[cfg(feature = "internal-software-crypto-cbc")]
pub use software::*;

use crate::{Error, Support};

/// CBC interface.
pub trait Api<Key, Block>: Support<bool> + Send
where
    Key: ArrayLength<u8>,
    Block: ArrayLength<u8>,
{
    /// Encrypts a sequence of blocks given a key and IV.
    fn encrypt(key: &Array<Key>, iv: &Array<Block>, blocks: &mut [u8]) -> Result<(), Error>;

    /// Decrypts a sequence of blocks given a key and IV.
    fn decrypt(key: &Array<Key>, iv: &Array<Block>, blocks: &mut [u8]) -> Result<(), Error>;
}

/// Sequence of N bytes.
pub type Array<N> = GenericArray<u8, N>;

#[cfg(feature = "internal-software-crypto-cbc")]
mod software {
    use core::marker::PhantomData;

    use aes::cipher::{BlockCipher, BlockDecryptMut, BlockEncryptMut};
    use cbc::{Decryptor, Encryptor};
    use crypto_common::{KeyInit, KeyIvInit, KeySizeUser};
    use wasefire_error::Code;

    use super::*;

    /// Generic CBC software implementation.
    pub struct Software<C> {
        cipher: PhantomData<C>,
    }

    impl<C> Support<bool> for Software<C> {
        const SUPPORT: bool = true;
    }

    impl<Key, Block, C> Api<Key, Block> for Software<C>
    where
        C: Send + KeyInit + KeySizeUser<KeySize = Key>,
        C: BlockCipher<BlockSize = Block> + BlockDecryptMut + BlockEncryptMut,
        Key: ArrayLength<u8>,
        Block: ArrayLength<u8>,
    {
        fn encrypt(key: &Array<Key>, iv: &Array<Block>, blocks: &mut [u8]) -> Result<(), Error> {
            Ok(Encryptor::<C>::new(key, iv).encrypt_blocks_mut(convert_blocks(blocks)?))
        }

        fn decrypt(key: &Array<Key>, iv: &Array<Block>, blocks: &mut [u8]) -> Result<(), Error> {
            Ok(Decryptor::<C>::new(key, iv).decrypt_blocks_mut(convert_blocks(blocks)?))
        }
    }

    fn convert_blocks<N: ArrayLength<u8>>(blocks: &mut [u8]) -> Result<&mut [Array<N>], Error> {
        if !blocks.len().is_multiple_of(N::USIZE) {
            return Err(Error::user(Code::InvalidLength));
        }
        let ptr = blocks.as_mut_ptr() as *mut Array<N>;
        let len = blocks.len() / N::USIZE;
        // SAFETY: Array<N> has the same layout as [u8; N].
        let blocks = unsafe { core::slice::from_raw_parts_mut(ptr, len) };
        Ok(blocks)
    }
}
