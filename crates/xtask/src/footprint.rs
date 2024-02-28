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

use std::collections::hash_map::Entry;
use std::collections::{BTreeSet, HashMap};
use std::fs::{File, OpenOptions};
use std::io::Write;

use anyhow::{anyhow, ensure, Context, Result};
use serde::{Deserialize, Serialize};
use wasefire_cli_tools::{cmd, fs};

use crate::wrap_command;

#[derive(Default, Serialize, Deserialize)]
struct Footprint {
    row: Vec<Row>,
}

#[derive(Serialize, Deserialize)]
struct Row {
    config: String,
    #[serde(flatten)]
    value: HashMap<String, usize>,
}

pub fn update_applet(config: &str, value: usize) -> Result<()> {
    update("applet", config, value)
}

pub fn update_runner(config: &str, value: usize) -> Result<()> {
    update("runner", config, value)
}

fn update(key: &str, config: &str, value: usize) -> Result<()> {
    const PATH: &str = "footprint.toml";
    let mut footprint = match fs::exists(PATH) {
        true => fs::read_toml(PATH)?,
        false => Footprint::default(),
    };
    let idx = match footprint.row.binary_search_by_key(&config, |x| &x.config) {
        Ok(x) => x,
        Err(x) => {
            let row = Row { config: config.to_string(), value: HashMap::new() };
            footprint.row.insert(x, row);
            x
        }
    };
    ensure!(
        footprint.row[idx].value.insert(key.to_string(), value).is_none(),
        "{key} is already defined for {config:?}"
    );
    fs::write_toml(PATH, &footprint)?;
    Ok(())
}

pub fn compare(output: &str) -> Result<()> {
    let mut output = OpenOptions::new().create(true).append(true).open(output)?;
    let base = Footprint::read("footprint-push.toml");
    let head = Footprint::read("footprint-pull_request.toml");
    let configs: BTreeSet<&String> = base.keys().chain(head.keys()).collect();
    writeln!(output, "### Footprint impact\n")?;
    writeln!(output, "| Config | Key | Base | Head | Diff | Ratio |")?;
    writeln!(output, "| --- | --- | ---: | ---: | ---: | ---: |")?;
    let write = |output: &mut File, x| match x {
        Some(x) => write!(output, " {x} |"),
        None => write!(output, " |"),
    };
    for config in configs {
        for key in ["total", "applet", "runner"] {
            let base = base.get(config).map(|x| x[key]);
            let head = head.get(config).map(|x| x[key]);
            let diff = base.and_then(|base| head.map(|head| head - base));
            write!(output, "| {config} | {key} |")?;
            write(&mut output, base)?;
            write(&mut output, head)?;
            write(&mut output, diff)?;
            match diff {
                Some(diff) => {
                    let base = base.unwrap();
                    let ratio = (diff as f64 / base as f64 * 100.) as i64;
                    let emoji = match ratio {
                        i64::MIN ..= -10 => ":+1: ",
                        -9 ..= 29 => "",
                        30 ..= i64::MAX => ":-1: ",
                    };
                    if !emoji.is_empty() {
                        println!("::notice::{config:?} {key} {emoji}{ratio}%");
                    }
                    write!(output, " {emoji}{ratio}% |")?;
                }
                None => write!(output, " |")?,
            }
            writeln!(output)?;
        }
    }
    Ok(())
}

/// Returns the sum of the .text and .data size.
pub fn rust_size(elf: &str) -> Result<usize> {
    let mut size = wrap_command()?;
    size.args(["rust-size", elf]);
    let output = String::from_utf8(cmd::output(&mut size)?.stdout)?;
    let line = output.lines().nth(1).context("parsing rust-size output")?;
    let words =
        line.split_whitespace().take(2).map(|x| Ok(x.parse()?)).collect::<Result<Vec<_>>>()?;
    Ok(words.iter().sum())
}

impl Footprint {
    fn read(path: &str) -> HashMap<String, HashMap<&'static str, i64>> {
        let mut result = HashMap::new();
        let footprint = fs::read_toml(path).unwrap_or_else(|err| {
            println!("::warning::{err}");
            Footprint::default()
        });
        for Row { config, mut value } in footprint.row {
            let value: Result<_> = try {
                let missing = |key| format!("Missing {key} for {config:?} in {path}");
                let applet = value.remove("applet").with_context(|| missing("applet"))?;
                let runner = value.remove("runner").with_context(|| missing("runner"))?;
                if !value.is_empty() {
                    Err(anyhow!("Extra keys for {config:?} in {path}"))?;
                }
                let mut value = HashMap::new();
                value.insert("applet", applet as i64);
                value.insert("total", runner as i64);
                value.insert("runner", runner as i64 - applet as i64);
                value
            };
            let value = match value {
                Ok(x) => x,
                Err(err) => {
                    println!("::warning::{err}");
                    continue;
                }
            };
            match result.entry(config) {
                Entry::Occupied(x) => println!("::warning::Duplicate {:?} in {path}", x.key()),
                Entry::Vacant(x) => drop(x.insert(value)),
            }
        }
        result
    }
}
