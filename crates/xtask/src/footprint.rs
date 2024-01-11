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

use std::collections::{HashMap, HashSet};

use anyhow::{ensure, Context, Result};
use serde::{Deserialize, Serialize};

use crate::{fs, output_command, wrap_command};

#[derive(Serialize, Deserialize)]
struct Footprint {
    version: usize,
    measurements: Vec<Measurement>,
}

#[derive(Serialize, Deserialize)]
struct Measurement {
    key: Vec<String>,
    value: usize,
}

pub fn update(key: &str, value: usize) -> Result<()> {
    const PATH: &str = "footprint.toml";
    let mut footprint = Footprint::read(PATH)?;
    let key = key.split_whitespace().map(|x| x.to_string()).collect();
    let measurement = Measurement { key, value };
    match footprint.measurements.iter_mut().find(|x| x.key == measurement.key) {
        Some(x) => x.value = value,
        None => footprint.measurements.push(measurement),
    }
    fs::write_toml(PATH, &footprint)?;
    Ok(())
}

pub fn compare() -> Result<()> {
    let base = Footprint::read_measurements("footprint-push.toml")?;
    let head = Footprint::read_measurements("footprint-pull_request.toml")?;
    let keys: HashSet<&Vec<String>> = base.keys().chain(head.keys()).collect();
    let mut keys: Vec<&Vec<String>> = keys.into_iter().collect();
    keys.sort();
    let print = |k, x| match x {
        Some(x) => print!(" {x}"),
        None => print!(" no-{k}"),
    };
    for key in keys {
        print!("{key:?}");
        let base = base.get(key).cloned();
        let head = head.get(key).cloned();
        let diff = base.and_then(|base| head.map(|head| head - base));
        print("base", base);
        print("head", head);
        print("diff", diff);
        println!();
    }
    Ok(())
}

/// Returns the sum of the .text and .data size.
pub fn rust_size(elf: &str) -> Result<usize> {
    let mut size = wrap_command()?;
    size.args(["rust-size", elf]);
    let output = String::from_utf8(output_command(&mut size)?.stdout)?;
    let line = output.lines().nth(1).context("parsing rust-size output")?;
    let words =
        line.split_whitespace().take(2).map(|x| Ok(x.parse()?)).collect::<Result<Vec<_>>>()?;
    Ok(words.iter().sum())
}

impl Footprint {
    fn read(path: &str) -> Result<Self> {
        let footprint = if fs::exists(path) {
            fs::read_toml(path)?
        } else {
            Footprint { version: VERSION, measurements: Vec::new() }
        };
        ensure!(footprint.version == VERSION);
        Ok(footprint)
    }

    fn read_measurements(path: &str) -> Result<HashMap<Vec<String>, usize>> {
        let mut result = HashMap::new();
        let footprint = Self::read(path)?;
        for Measurement { key, value } in footprint.measurements {
            ensure!(result.insert(key, value).is_none(), "duplicate key in {path}");
        }
        Ok(result)
    }
}

const VERSION: usize = 1;
