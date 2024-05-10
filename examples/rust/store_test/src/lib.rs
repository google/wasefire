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

//! Tests that the store is working properly.

#![no_std]
wasefire::applet!();

use alloc::vec::Vec;

fn main() {
    store::clear().unwrap();
    debug::assert(store::keys().unwrap().is_empty());
    test_insert();
    test_remove();
    test_find();
    test_keys();
    test_fragment();
    debug::exit(true);
}

fn test_insert() {
    debug!("test_insert(): Inserts some entries into the store.");
    fn insert(key: usize) {
        let value = value(key);
        let length = value.len();
        debug!("- Insert {key:4} with {length:4} bytes");
        store::insert(key, &value).unwrap();
    }
    for &key in INSERTED {
        insert(key);
        insert(reverse(key));
    }
}

fn test_remove() {
    debug!("test_remove(): Removes some entries.");
    fn remove(key: usize) {
        debug!("- Remove {key:4}");
        store::remove(key).unwrap();
    }
    for &key in REMOVED {
        remove(key);
        remove(reverse(key));
    }
}

fn test_find() {
    debug!("test_find(): Checks whether entries where inserted/removed.");
    fn find(key: usize, removed: bool) {
        let expected = (!removed).then(|| value(key));
        debug!("- Check {key:4} is {}", if removed { "-removed" } else { "present" });
        let actual = store::find(key).unwrap();
        debug::assert_eq(&actual.as_deref(), &expected.as_deref());
    }
    for &key in INSERTED {
        let removed = REMOVED.contains(&key);
        find(key, removed);
        find(reverse(key), removed);
    }
}

fn test_keys() {
    debug!("test_keys(): Checks that keys match entries.");
    let mut expected = Vec::new();
    for &key in INSERTED {
        if REMOVED.contains(&key) {
            continue;
        }
        expected.push(key as u16);
        expected.push(reverse(key) as u16);
    }
    expected.sort();
    debug!("- {expected:?}");
    let mut actual = store::keys().unwrap();
    actual.sort();
    debug!("- {actual:?}");
    debug::assert_eq(&actual, &expected);
}

fn test_fragment() {
    debug!("test_fragment(): Test fragmented entries.");
    debug!("- insert then find");
    store::fragment::insert(0 .. 2, &[0xca; 1500]).unwrap();
    debug::assert_eq(&store::fragment::find(0 .. 2).unwrap().unwrap()[..], &[0xca; 1500][..]);
    debug!("- remove then find");
    store::fragment::remove(0 .. 2).unwrap();
    debug::assert(store::fragment::find(0 .. 2).unwrap().is_none());
}

const INSERTED: &[usize] = &[0, 1, 2, 3, 100, 500, 1000, 2000];
const REMOVED: &[usize] = &[2, 500];

fn reverse(key: usize) -> usize {
    4095 - key
}

fn value(mut key: usize) -> Vec<u8> {
    let reversed = key & 0x800 != 0;
    if reversed {
        key = 4095 - key;
    }
    let val = key as u8;
    let mut len = val as usize;
    if reversed {
        len = 1023 - len;
    }
    alloc::vec![val; len]
}
