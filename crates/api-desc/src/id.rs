// Copyright 2022 Google LLC
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

const FORMAT: data_encoding::Encoding = data_encoding_macro::new_encoding! {
    symbols: " ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_",
};

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Id(u32);

#[derive(Debug, Copy, Clone)]
pub struct Name([u8; 6]);

impl Id {
    /// Checks whether an id is valid.
    ///
    /// Valid ids have valid names.
    pub fn new(id: u32) -> Option<Self> {
        let name = id_to_name(id);
        Name::from_bytes(&name[.. name_len(name)])?;
        Some(Id(id))
    }
}

impl Name {
    /// Checks whether a name is valid.
    ///
    /// Valid names have between 1 and 5 alphanumeric or underscore characters and the first
    /// character is not a digit.
    pub fn new(name: &str) -> Option<Self> {
        Self::from_bytes(name.as_bytes())
    }

    fn from_bytes(name: &[u8]) -> Option<Self> {
        let len = name.len();
        if (len < 1 || 5 < len || name[0].is_ascii_digit())
            || !name.iter().all(|&x| x.is_ascii_alphanumeric() || x == b'_')
        {
            return None;
        }
        let mut result = [b' '; 6];
        result[.. len].copy_from_slice(name);
        Some(Name(result))
    }

    pub fn as_str(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(&self.0[.. name_len(self.0)]) }
    }
}

impl From<Id> for u32 {
    fn from(id: Id) -> Self {
        id.0
    }
}

impl From<Id> for Name {
    fn from(id: Id) -> Self {
        Name(id_to_name(id.0))
    }
}

impl From<Name> for Id {
    fn from(name: Name) -> Self {
        Id(name_to_id(name.0))
    }
}

fn id_to_name(id: u32) -> [u8; 6] {
    let mut name = [0; 6];
    FORMAT.encode_mut(&id.to_be_bytes(), &mut name);
    name
}

fn name_to_id(name: [u8; 6]) -> u32 {
    let mut id = [0; 4];
    let len = FORMAT.decode_mut(&name, &mut id).unwrap();
    debug_assert_eq!(len, 4);
    u32::from_be_bytes(id)
}

fn name_len(name: [u8; 6]) -> usize {
    let mut len = 6;
    while len > 0 && name[len - 1] == b' ' {
        len -= 1;
    }
    len
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_name_map() {
        #[track_caller]
        fn test(name: &[u8; 6], id: u32) {
            let actual = name_to_id(*name);
            assert_eq!(actual, id, "{actual:#x?}");
            let actual = id_to_name(id);
            assert_eq!(actual, *name, "{:?}", std::str::from_utf8(&actual));
        }
        test(b"      ", 0x00000000);
        test(b"     P", 0x00000001);
        test(b"     f", 0x00000002);
        test(b"     v", 0x00000003);
        test(b"    A ", 0x00000004);
        test(b"    B ", 0x00000008);
        test(b"    Bv", 0x0000000b);
        test(b"    _v", 0x000000ff);
        test(b"   A  ", 0x00000100);
        test(b"   A_v", 0x000001ff);
        test(b"   __v", 0x00003fff);
        test(b"  A   ", 0x00004000);
        test(b"  ___v", 0x000fffff);
        test(b" A    ", 0x00100000);
        test(b" ____v", 0x03ffffff);
        test(b"A     ", 0x04000000);
        test(b"AAAAA ", 0x04104104);
        test(b"ffffff", 0x82082082);
        test(b"_____v", 0xffffffff);
    }

    #[test]
    fn name_new() {
        #[track_caller]
        fn test(expected: Option<&[u8; 6]>, name: &str) {
            let expected = expected.map(|x| *x);
            let actual = Name::new(name).map(|x| x.0);
            assert_eq!(actual, expected);
        }
        test(None /* .... */, "");
        test(None /* .... */, " ");
        test(None /* .... */, "  ");
        test(None /* .... */, " A");
        test(None /* .... */, "A ");
        test(None /* .... */, "f o o");
        test(None /* .... */, "0123");
        test(None /* .... */, "foobar");
        test(Some(b"USB   "), "USB");
        test(Some(b"ASN_1 "), "ASN_1");
        test(Some(b"_0k   "), "_0k");
        test(Some(b"_____ "), "_____");
    }

    #[test]
    fn name_to_str() {
        #[track_caller]
        fn test(name: &[u8], expected: &str) {
            let name = Name::from_bytes(name).unwrap();
            assert_eq!(name.as_str(), expected);
        }
        test(b"A", "A");
        test(b"heLL0", "heLL0");
        test(b"_0k", "_0k");
    }
}
