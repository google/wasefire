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

use std::collections::BTreeMap;

use fuzz::correct;

fn main() {
    let mut stats = BTreeMap::<u8, Len>::new();
    let mut no_tag = 0;
    for entry in std::fs::read_dir("corpus/correct").unwrap() {
        let data = std::fs::read(entry.unwrap().path()).unwrap();
        let mut tag = None;
        let len = correct(&data, &mut tag).ok();
        match tag {
            None => no_tag += 1,
            Some(tag) => stats.entry(tag).or_default().update(len),
        }
    }
    println!("  -: {no_tag}");
    for (tag, len) in stats {
        let Len { err, ok, min, sum, max } = len;
        let avg = sum.checked_div(ok).unwrap_or(0);
        println!("{tag:3}: {ok} [{min} {avg} {max}] {err}");
    }
}

#[derive(Default)]
struct Len {
    err: usize,
    ok: usize,
    min: usize,
    sum: usize,
    max: usize,
}

impl Len {
    fn update(&mut self, len: Option<usize>) {
        match len {
            None => self.err += 1,
            Some(len) => {
                self.ok += 1;
                self.min = std::cmp::min(self.min, len);
                self.sum += len;
                self.max = std::cmp::max(self.max, len);
            }
        }
    }
}
