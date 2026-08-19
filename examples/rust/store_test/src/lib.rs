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

use alloc::vec;
use alloc::vec::Vec;

fn main() {
    store::clear().unwrap();
    assert!(store::keys().unwrap().is_empty());
    let mut inserted = vec![0, 1, 2, 3, 100, 500, 1000, 2000];
    let mut removed = vec![2, 500];
    inserted.retain(|x| 2 * x <= store::max_key());
    removed.retain(|x| 2 * x <= store::max_key());
    test_insert(&inserted);
    test_remove(&removed);
    test_find(&inserted, &removed);
    test_keys(&inserted, &removed);
    test_fragment();
    scheduling::exit();
}

fn test_insert(inserted: &[usize]) {
    debug!("test_insert(): Inserts some entries into the store.");
    fn insert(key: usize) {
        let value = value(key);
        let length = value.len();
        debug!("- Insert {key:4} with {length:4} bytes");
        store::insert(key, &value).unwrap();
    }
    for &key in inserted {
        insert(key);
        insert(reverse(key));
    }
}

fn test_remove(removed: &[usize]) {
    debug!("test_remove(): Removes some entries.");
    fn remove(key: usize) {
        debug!("- Remove {key:4}");
        store::remove(key).unwrap();
    }
    for &key in removed {
        remove(key);
        remove(reverse(key));
    }
}

fn test_find(inserted: &[usize], removed: &[usize]) {
    debug!("test_find(): Checks whether entries where inserted/removed.");
    fn find(key: usize, removed: bool) {
        let expected = (!removed).then(|| value(key));
        debug!("- Check {key:4} is {}", if removed { "-removed" } else { "present" });
        let actual = store::find(key).unwrap();
        assert_eq!(actual.as_deref(), expected.as_deref());
    }
    for &key in inserted {
        let removed = removed.contains(&key);
        find(key, removed);
        find(reverse(key), removed);
    }
}

fn test_keys(inserted: &[usize], removed: &[usize]) {
    debug!("test_keys(): Checks that keys match entries.");
    let mut expected = Vec::new();
    for &key in inserted {
        if removed.contains(&key) {
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
    assert_eq!(actual, expected);
}

fn test_fragment() {
    debug!("test_fragment(): Test fragmented entries.");
    let value = vec![0xca; 3 * store::max_len() / 2];
    debug!("- insert then find");
    store::fragment::insert(0 .. 2, &value).unwrap();
    assert_eq!(store::fragment::find(0 .. 2).unwrap().unwrap()[..], value);
    debug!("- remove then find");
    store::fragment::remove(0 .. 2).unwrap();
    assert!(store::fragment::find(0 .. 2).unwrap().is_none());
}

fn reverse(key: usize) -> usize {
    store::max_key() - key
}

fn value(mut key: usize) -> Vec<u8> {
    let reversed = 2 * key <= store::max_key();
    if reversed {
        key = store::max_key() - key;
    }
    let val = key as u8;
    let mut len = val as usize;
    if reversed {
        len = store::max_len() - len;
    }
    alloc::vec![val; len]
}
