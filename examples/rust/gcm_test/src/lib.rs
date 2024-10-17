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

//! Tests that AES-256-GCM is working properly.

#![no_std]
wasefire::applet!();

use alloc::vec;

#[cfg(feature = "rust-crypto")]
use aead::{Aead, AeadInPlace, KeyInit, Payload};
use wasefire::crypto::gcm::tag_length;
#[cfg(not(feature = "rust-crypto"))]
use wasefire::crypto::gcm::{decrypt, decrypt_in_place, encrypt, encrypt_in_place, Cipher};
#[cfg(feature = "rust-crypto")]
use wasefire::crypto::gcm::{Aes256Gcm, Aes256GcmInPlace};

fn main() {
    debug!("Use RustCrypto API: {}", cfg!(feature = "rust-crypto"));
    if crypto::gcm::is_supported() {
        test_encrypt();
        test_encrypt_in_place();
        test_decrypt();
        test_decrypt_in_place();
    }
    scheduling::exit();
}

fn test_encrypt() {
    debug!("test_encrypt(): Encrypts the test vectors.");
    let tag_len = tag_length();
    for &Vector { key, iv, aad, clear, cipher, tag } in TEST_VECTORS {
        debug!("- {} bytes", clear.len());
        #[cfg(feature = "rust-crypto")]
        let (cipher_, tag_) = {
            let key = Aes256Gcm::new(key.into());
            let mut cipher_ = key.encrypt(iv.into(), Payload { msg: clear, aad }).unwrap();
            let tag_ = cipher_[clear.len() ..][.. tag_len].to_vec();
            cipher_.truncate(clear.len());
            (cipher_, tag_)
        };
        #[cfg(not(feature = "rust-crypto"))]
        let Cipher { text: cipher_, tag: tag_ } = encrypt(key, iv, aad, clear).unwrap();
        assert_eq!(cipher_[..], cipher[..]);
        assert_eq!(tag_[..], tag[.. tag_len]);
    }
}

fn test_encrypt_in_place() {
    debug!("test_encrypt_in_place(): Encrypts the test vectors in place.");
    let tag_len = tag_length();
    for &Vector { key, iv, aad, clear, cipher, tag } in TEST_VECTORS {
        debug!("- {} bytes", clear.len());
        let mut cipher_ = clear.to_vec();
        let mut tag_ = vec![0; tag_len];
        #[cfg(feature = "rust-crypto")]
        {
            let key = Aes256GcmInPlace::new(key.into());
            let tag = key.encrypt_in_place_detached(iv.into(), aad, &mut cipher_).unwrap();
            tag_.copy_from_slice(&tag[.. tag_len]);
        }
        #[cfg(not(feature = "rust-crypto"))]
        encrypt_in_place(key, iv, aad, &mut cipher_, &mut tag_).unwrap();
        assert_eq!(cipher_[..], cipher[..]);
        assert_eq!(tag_[..], tag[.. tag_len]);
    }
}

fn test_decrypt() {
    debug!("test_decrypt(): Decrypts the test vectors.");
    #[cfg(not(feature = "rust-crypto"))]
    let tag_len = tag_length();
    for &Vector { key, iv, aad, clear, cipher, tag } in TEST_VECTORS {
        debug!("- {} bytes", clear.len());
        #[cfg(feature = "rust-crypto")]
        let clear_ = {
            let key = Aes256Gcm::new(key.into());
            let mut msg = cipher.to_vec();
            msg.extend_from_slice(tag);
            key.decrypt(iv.into(), Payload { msg: &msg, aad }).unwrap()
        };
        #[cfg(not(feature = "rust-crypto"))]
        let clear_ = {
            let mut tag_ = vec![0; tag_len];
            tag_.copy_from_slice(&tag[.. tag_len]);
            let cipher = Cipher { text: cipher.to_vec(), tag: tag_ };
            decrypt(key, iv, aad, &cipher).unwrap()
        };
        assert_eq!(clear_[..], clear[..]);
    }
}

fn test_decrypt_in_place() {
    debug!("test_decrypt_in_place(): Decrypts the test vectors in place.");
    #[cfg(not(feature = "rust-crypto"))]
    let tag_len = tag_length();
    for &Vector { key, iv, aad, clear, cipher, tag } in TEST_VECTORS {
        debug!("- {} bytes", clear.len());
        let mut clear_ = cipher.to_vec();
        #[cfg(feature = "rust-crypto")]
        Aes256GcmInPlace::new(key.into())
            .decrypt_in_place_detached(iv.into(), aad, &mut clear_, tag.into())
            .unwrap();
        #[cfg(not(feature = "rust-crypto"))]
        decrypt_in_place(key, iv, aad, &tag[.. tag_len], &mut clear_).unwrap();
        assert_eq!(clear_[..], clear[..]);
    }
}

struct Vector {
    key: &'static [u8; 32],
    iv: &'static [u8; 12],
    aad: &'static [u8],
    clear: &'static [u8],
    cipher: &'static [u8],
    tag: &'static [u8; 16],
}

// Those test vectors are taken from: https://github.com/google/boringssl/blob/master/crypto/
// cipher_extra/test/aes_256_gcm_tests.txt
const TEST_VECTORS: &[Vector] = &[
    Vector {
        key: &[
            0xe5, 0xac, 0x4a, 0x32, 0xc6, 0x7e, 0x42, 0x5a, 0xc4, 0xb1, 0x43, 0xc8, 0x3c, 0x6f,
            0x16, 0x13, 0x12, 0xa9, 0x7d, 0x88, 0xd6, 0x34, 0xaf, 0xdf, 0x9f, 0x4d, 0xa5, 0xbd,
            0x35, 0x22, 0x3f, 0x01,
        ],
        iv: &[0x5b, 0xf1, 0x1a, 0x09, 0x51, 0xf0, 0xbf, 0xc7, 0xea, 0x5c, 0x9e, 0x58],
        aad: &[],
        clear: &[],
        cipher: &[],
        tag: &[
            0xd7, 0xcb, 0xa2, 0x89, 0xd6, 0xd1, 0x9a, 0x5a, 0xf4, 0x5d, 0xc1, 0x38, 0x57, 0x01,
            0x6b, 0xac,
        ],
    },
    Vector {
        key: &[
            0x73, 0xad, 0x7b, 0xbb, 0xbc, 0x64, 0x0c, 0x84, 0x5a, 0x15, 0x0f, 0x67, 0xd0, 0x58,
            0xb2, 0x79, 0x84, 0x93, 0x70, 0xcd, 0x2c, 0x1f, 0x3c, 0x67, 0xc4, 0xdd, 0x6c, 0x86,
            0x92, 0x13, 0xe1, 0x3a,
        ],
        iv: &[0xa3, 0x30, 0xa1, 0x84, 0xfc, 0x24, 0x58, 0x12, 0xf4, 0x82, 0x0c, 0xaa],
        aad: &[0xe9, 0x14, 0x28, 0xbe, 0x04],
        clear: &[0xf0, 0x53, 0x5f, 0xe2, 0x11],
        cipher: &[0xe9, 0xb8, 0xa8, 0x96, 0xda],
        tag: &[
            0x91, 0x15, 0xed, 0x79, 0xf2, 0x6a, 0x03, 0x0c, 0x14, 0x94, 0x7b, 0x3e, 0x45, 0x4d,
            0xb9, 0xe7,
        ],
    },
    Vector {
        key: &[
            0x80, 0xe2, 0xe5, 0x61, 0x88, 0x6e, 0xb2, 0xa9, 0x53, 0xcf, 0x92, 0x3a, 0xaa, 0xc1,
            0x65, 0x3e, 0xd2, 0xdb, 0x01, 0x11, 0xee, 0x62, 0xe0, 0x9c, 0xb2, 0x0d, 0x9e, 0x26,
            0x52, 0xbd, 0x34, 0x76,
        ],
        iv: &[0x5d, 0xaf, 0x20, 0x15, 0x89, 0x65, 0x4d, 0xa8, 0x88, 0x4c, 0x3c, 0x68],
        aad: &[0xe5, 0x1e, 0x5b, 0xce, 0x7c, 0xbc, 0xeb, 0x66, 0x03, 0x99],
        clear: &[0x96, 0x66, 0x9d, 0x2d, 0x35, 0x42, 0xa4, 0xd4, 0x9c, 0x7c],
        cipher: &[0x45, 0x21, 0x95, 0x3e, 0x7d, 0x39, 0x49, 0x7e, 0x45, 0x63],
        tag: &[
            0x20, 0x83, 0xe3, 0xc0, 0xd8, 0x4d, 0x66, 0x30, 0x66, 0xbb, 0xe2, 0x96, 0x1b, 0x08,
            0xdc, 0xf7,
        ],
    },
    Vector {
        key: &[
            0x88, 0x1c, 0xca, 0x01, 0x2e, 0xf9, 0xd6, 0xf1, 0x24, 0x1b, 0x88, 0xe4, 0x36, 0x40,
            0x84, 0xd8, 0xc9, 0x54, 0x70, 0xc6, 0x02, 0x2e, 0x59, 0xb6, 0x27, 0x32, 0xa1, 0xaf,
            0xcc, 0x02, 0xe6, 0x57,
        ],
        iv: &[0x17, 0x2e, 0xc6, 0x39, 0xbe, 0x73, 0x60, 0x62, 0xbb, 0xa5, 0xc3, 0x2f],
        aad: &[
            0x98, 0xc1, 0x15, 0xf2, 0xc3, 0xbb, 0xe2, 0x2e, 0x3a, 0x0c, 0x56, 0x2e, 0x8e, 0x67,
            0xff,
        ],
        clear: &[
            0x8e, 0xd8, 0xef, 0x4c, 0x09, 0x36, 0x0e, 0xf7, 0x0b, 0xb2, 0x2c, 0x71, 0x65, 0x54,
            0xef,
        ],
        cipher: &[
            0x06, 0xa7, 0x61, 0x98, 0x7a, 0x7e, 0xb0, 0xe5, 0x7a, 0x31, 0x97, 0x90, 0x43, 0x74,
            0x7d,
        ],
        tag: &[
            0xcf, 0x07, 0x23, 0x9b, 0x9d, 0x40, 0xa7, 0x59, 0xe0, 0xf4, 0xf8, 0xef, 0x08, 0x8f,
            0x01, 0x6a,
        ],
    },
    Vector {
        key: &[
            0xa6, 0xef, 0xd2, 0xe2, 0xb0, 0x05, 0x6d, 0x0f, 0x95, 0x5e, 0x00, 0x8c, 0xa8, 0x8c,
            0xa5, 0x9f, 0xb2, 0x1a, 0x8f, 0x5f, 0xc0, 0xe9, 0xaa, 0x6d, 0x73, 0x0f, 0xbf, 0xc5,
            0xa2, 0x8b, 0x5f, 0x90,
        ],
        iv: &[0xf6, 0x77, 0x5d, 0xca, 0x7c, 0xd8, 0x67, 0x4c, 0x16, 0xfd, 0xb4, 0xee],
        aad: &[
            0x86, 0xa5, 0x97, 0xf5, 0xe2, 0xc3, 0x98, 0xff, 0xf9, 0x63, 0xfc, 0xfe, 0x12, 0x6e,
            0xae, 0x1b, 0xc1, 0x3f, 0x09, 0x7f,
        ],
        clear: &[
            0x5d, 0xc4, 0x95, 0xd9, 0x49, 0xf4, 0xb2, 0xc8, 0xa7, 0x09, 0x09, 0x2b, 0x12, 0x0a,
            0xc8, 0x07, 0x8c, 0xdf, 0xd1, 0x04,
        ],
        cipher: &[
            0x04, 0x41, 0x6e, 0x23, 0x58, 0x6e, 0xe3, 0x64, 0xb1, 0xcf, 0x3f, 0xb7, 0x54, 0x05,
            0xf8, 0xef, 0x28, 0xfd, 0xdb, 0xde,
        ],
        tag: &[
            0xe7, 0xb9, 0xd5, 0xec, 0xb2, 0xcf, 0x30, 0x16, 0x2a, 0x28, 0xc8, 0xf6, 0x45, 0xf6,
            0x2f, 0x87,
        ],
    },
    Vector {
        key: &[
            0x8d, 0x6e, 0xd9, 0xa6, 0xd4, 0x10, 0x98, 0x9e, 0x3b, 0xd3, 0x78, 0x74, 0xed, 0xb5,
            0xa8, 0x9f, 0x9a, 0xb3, 0x55, 0xfa, 0x39, 0x59, 0x67, 0xdc, 0xbb, 0xfa, 0x21, 0x6e,
            0xc9, 0xce, 0x3f, 0x45,
        ],
        iv: &[0x55, 0xde, 0xbb, 0xb2, 0x89, 0xb9, 0x43, 0x9e, 0xb4, 0x78, 0x34, 0xab],
        aad: &[
            0x77, 0x90, 0xaf, 0x91, 0x3d, 0x84, 0xa0, 0x4c, 0x1b, 0x72, 0xd4, 0x48, 0x4e, 0xa2,
            0xe0, 0x9f, 0xda, 0xa8, 0x02, 0xd8, 0xb1, 0x73, 0x3b, 0x84, 0x70,
        ],
        clear: &[
            0x52, 0x93, 0x9c, 0x74, 0x16, 0x22, 0x08, 0x22, 0xa7, 0x74, 0x35, 0xa4, 0x66, 0x87,
            0xf1, 0x34, 0xce, 0xbc, 0x70, 0xa2, 0xf1, 0xa4, 0xc3, 0x3d, 0x37,
        ],
        cipher: &[
            0xd7, 0xbd, 0xda, 0xe8, 0x92, 0x9e, 0xd6, 0xbb, 0xc9, 0xac, 0x07, 0x7e, 0x24, 0x15,
            0xd9, 0xfb, 0xaf, 0xae, 0x4a, 0x04, 0x32, 0xf8, 0xf7, 0xeb, 0x6b,
        ],
        tag: &[
            0xe6, 0x38, 0x3b, 0x16, 0xed, 0x9c, 0x32, 0x52, 0x1d, 0xca, 0xee, 0xf3, 0xa7, 0xb9,
            0xb6, 0x7f,
        ],
    },
];
