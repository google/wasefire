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

#![no_main]

use std::ops::Bound;

use libfuzzer_sys::{Corpus, fuzz_target};
use wasefire_slice_cell::SliceCell;

fuzz_target!(|data: &[u8]| -> Corpus {
    let mut entropy = Entropy(data);
    match entropy.next() {
        0 => test::<u8>(entropy),
        1 => test::<u16>(entropy),
        _ => return Corpus::Reject,
    }
    Corpus::Keep
});

fn test<T: Type>(mut entropy: Entropy) {
    let len = entropy.next() as usize;
    let mut data = vec![T::pack(0); len];
    let mut view = SliceCell::new(&mut data);
    let mut shared = Vec::new();
    let mut exclusive = Vec::new();
    // Collect arbitrary shared and exclusive accesses until entropy runs out or an access fails.
    while !entropy.exhausted() {
        let mut bias = BiasedEntropy::new(entropy.next());
        let result = match bias.next(2) {
            0 => {
                let end = bound(&mut bias, &mut entropy);
                let start = bound(&mut bias, &mut entropy);
                let range = (start, end);
                match bias.next(2) {
                    0 => view.get_range(range).map(|x| shared.push(x)),
                    1 => view.get_range_mut(range).map(|x| exclusive.push(x)),
                    _ => unreachable!(),
                }
            }
            1 => {
                let index = entropy.next() as usize;
                match bias.next(2) {
                    0 => view.get(index).map(|x| shared.push(std::slice::from_ref(x))),
                    1 => view.get_mut(index).map(|x| exclusive.push(std::slice::from_mut(x))),
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        };
        let Ok(()) = result else { break };
    }
    // Make sure we start from an expected state (probably not necessary).
    for slice in shared.iter().cloned().chain(exclusive.iter().map(|x| x as &[_])) {
        read(slice, T::pack(0));
    }
    // Make sure we can write to all exclusive accesses.
    for (id, slice) in exclusive.iter_mut().enumerate() {
        write(slice, T::pack(1 + id));
    }
    // Make sure the previous operations did not invalidate any exclusive access.
    for (id, slice) in exclusive.iter_mut().enumerate() {
        read(slice, T::pack(1 + id));
        write(slice, T::pack(!0));
    }
    // Make sure the shared accesses are disjoint from the exclusive ones.
    for slice in &shared {
        read(slice, T::pack(0));
    }
    // Make sure we can get back exclusive access to the full slice.
    view.reset();
    write(view.get_range_mut(0 .. len).unwrap(), T::pack(0));
}

fn bound(bias: &mut BiasedEntropy, entropy: &mut Entropy) -> Bound<usize> {
    match bias.next(3) {
        0 => Bound::Unbounded,
        1 => Bound::Included(entropy.next() as usize),
        2 => Bound::Excluded(entropy.next() as usize),
        _ => unreachable!(),
    }
}

fn read<T: Type>(xs: &[T], v: T) {
    for x in xs {
        assert_eq!(unsafe { std::ptr::read_volatile(x) }, v);
    }
}

fn write<T: Type>(xs: &mut [T], v: T) {
    for x in xs {
        unsafe { std::ptr::write_volatile(x, v) };
    }
}

struct BiasedEntropy {
    data: u8,
    used: u8,
}

impl BiasedEntropy {
    fn new(data: u8) -> Self {
        BiasedEntropy { data, used: 0 }
    }

    fn next(&mut self, count: u8) -> u8 {
        // Make sure we don't ask for too much.
        self.used = ((self.used + 1) as u16 * count as u16 - 1).try_into().unwrap();
        let result = self.data % count;
        self.data /= count;
        result
    }
}

struct Entropy<'a>(&'a [u8]);

impl Entropy<'_> {
    fn exhausted(&self) -> bool {
        self.0.is_empty()
    }

    fn next(&mut self) -> u8 {
        let Some((byte, data)) = self.0.split_first() else { return 0 };
        self.0 = data;
        *byte
    }
}

trait Type: Copy + std::fmt::Debug + Eq {
    fn pack(id: usize) -> Self;
}

impl Type for u8 {
    fn pack(id: usize) -> Self {
        id as u8
    }
}

impl Type for u16 {
    fn pack(id: usize) -> Self {
        id as u16
    }
}
