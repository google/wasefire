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

#![feature(box_patterns)]

use std::collections::HashMap;
use std::fmt::Write;
use std::process::Command;

use anyhow::{bail, ensure, Context, Result};
use wasefire_cli_tools::{cmd, fs};
use wasefire_error::Error;
use wasefire_protocol::{Api, Descriptor, Request, Response, DESCRIPTORS, VERSION};
use wasefire_wire::schema::{View, ViewEnum, ViewFields};
use wasefire_wire::{Wire, Yoke};

fn main() -> Result<()> {
    let base = std::env::var("BASE_REF").unwrap_or_else(|_| {
        format!("origin/{}", std::env::var("GITHUB_BASE_REF").as_deref().unwrap_or("main"))
    });
    let new = Schema::new();
    new.write().context("writing bin")?;
    new.print().context("printing txt")?;
    let hash = cmd::output_line(Command::new("git").args(["rev-parse", &base]))?;
    if hash == "13a0d6eb5ed261c2c0e89744bb339b99e24d2e2a" {
        return Ok(());
    }
    let old = Schema::old(&base)?;
    check(&old.get().result, &new.result).context("checking result")?;
    check(&old.get().request, &new.request).context("checking request")?;
    check(&old.get().response, &new.response).context("checking response")?;
    check_versions(old.get(), &new).context("checking versions")?;
    Ok(())
}

fn check(old: &View, new: &View) -> Result<()> {
    let old = old.simplify();
    let new = new.simplify();
    let mut old_map = HashMap::new();
    for (_, tag, api) in get_enum(&old).context("in old")? {
        ensure!(old_map.insert(tag, api).is_none(), "duplicate tag {tag}");
    }
    for (_, tag, new_api) in get_enum(&new).context("in new")? {
        if let Some(old_api) = old_map.remove(tag) {
            let old_api = ViewFields(old_api);
            let new_api = ViewFields(new_api);
            ensure!(old_api == new_api, "incompatible API for {tag}: {old_api} != {new_api}");
        }
    }
    #[cfg(feature = "host")]
    ensure!(old_map.is_empty(), "deleted tag {}", old_map.keys().next().unwrap());
    Ok(())
}

fn check_versions(old: &Schema, new: &Schema) -> Result<()> {
    let mut old_map = HashMap::new();
    for Descriptor { tag, versions } in &old.descriptors {
        ensure!(old_map.insert(tag, *versions).is_none(), "duplicate tag {tag}");
    }
    let mut changed = false;
    let old_version = old.version;
    let new_version = new.version;
    for Descriptor { tag, versions: new } in &new.descriptors {
        #[cfg(feature = "device")]
        ensure!(new.max.is_none(), "max version is defined for {tag}");
        match old_map.remove(&tag) {
            Some(old) => {
                ensure!(new.min == old.min, "min version changed for {tag}");
                #[cfg(feature = "host")]
                match (old.max, new.max) {
                    (None, Some(new)) if new == old_version => (),
                    (None, Some(_)) => bail!("max version for {tag} must be {old_version}"),
                    (old, new) if old == new => (),
                    _ => bail!("max version changed for {tag}"),
                }
            }
            None => {
                changed = true;
                ensure!(new.min == new_version, "min version for {tag} must be {new_version}");
            }
        }
    }
    changed |= !old_map.is_empty();
    #[cfg(feature = "host")]
    ensure!(old_map.is_empty(), "deleted tag {}", old_map.keys().next().unwrap());
    #[cfg(feature = "host")]
    let _ = changed;
    #[cfg(feature = "device")]
    ensure!(new_version == old_version + changed as u32, "invalid version (must bump: {changed})");
    Ok(())
}

#[derive(Debug, Wire)]
struct Schema<'a> {
    result: View<'a>,   // Result<(), Error>
    request: View<'a>,  // Api<Request>
    response: View<'a>, // Api<Response>
    version: u32,
    descriptors: Vec<Descriptor>,
}

impl Schema<'static> {
    fn new() -> Self {
        Schema {
            result: View::new::<Result<(), Error>>(),
            request: View::new::<Api<Request>>(),
            response: View::new::<Api<Response>>(),
            version: VERSION,
            descriptors: DESCRIPTORS.to_vec(),
        }
    }

    fn old(base: &str) -> Result<Yoke<Schema<'static>>> {
        let mut git = Command::new("git");
        git.args(["show", &format!("{base}:./{SIDE}.bin")]);
        let data = cmd::output(&mut git)?.stdout.into_boxed_slice();
        Ok(wasefire_wire::decode_yoke(data)?)
    }

    fn write(&self) -> Result<()> {
        fs::write(format!("{SIDE}.bin"), wasefire_wire::encode(self)?)
    }

    fn print(&self) -> Result<()> {
        let mut output = String::new();
        writeln!(&mut output, "result: {}", self.result)?;
        writeln!(&mut output, "version: {}", self.version)?;
        let request = get_enum(&self.request).context("in request")?.iter();
        let mut response = get_enum(&self.response).context("in response")?.iter();
        let mut descriptor = self.descriptors.iter();
        for request in request {
            let name = request.0;
            let tag = request.1;
            let response = response.next().unwrap();
            let descriptor = descriptor.next().unwrap();
            assert_eq!(response.0, name);
            assert_eq!(response.1, tag);
            assert_eq!(descriptor.tag, tag);
            let request = ViewFields(&request.2);
            let response = ViewFields(&response.2);
            writeln!(&mut output, "{descriptor} {name}: {request} -> {response}")?;
        }
        assert!(response.next().is_none());
        assert!(descriptor.next().is_none());
        fs::write(format!("{SIDE}.txt"), &output)
    }
}

#[cfg(feature = "host")]
const SIDE: &str = "host";
#[cfg(feature = "device")]
const SIDE: &str = "device";

fn get_enum<'a>(x: &'a View<'a>) -> Result<&'a ViewEnum<'a>> {
    match x {
        View::Enum(xs) => Ok(xs),
        x => bail!("expected enum, got {x}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adding_variant() {
        #[derive(Wire)]
        enum Old {
            Foo,
        }
        #[derive(Wire)]
        enum New {
            Foo,
            Bar,
        }
        // Adding a variant is always accepted.
        assert!(check(&View::new::<Old>(), &View::new::<New>()).is_ok());
    }

    #[test]
    fn removing_variant() {
        #[derive(Wire)]
        enum Old {
            Foo,
            Bar,
        }
        #[derive(Wire)]
        enum New {
            Foo,
        }
        // Removing a variant is accepted on the device.
        #[cfg(feature = "device")]
        assert!(check(&View::new::<Old>(), &View::new::<New>()).is_ok());
        // But rejected on the host.
        #[cfg(feature = "host")]
        assert!(check(&View::new::<Old>(), &View::new::<New>()).is_err());
    }

    #[test]
    fn updating_variant_incompatible() {
        #[derive(Wire)]
        enum Old {
            Foo,
            Bar(u8),
        }
        #[derive(Wire)]
        enum New {
            Foo,
            Bar(i8),
        }
        // Updating a variant in an incompatible way is always rejected. Which updates are
        // considered compatible should always be sound but may be incomplete.
        assert!(check(&View::new::<Old>(), &View::new::<New>()).is_err());
    }

    #[test]
    fn updating_variant_compatible() {
        #[derive(Wire)]
        enum Old {
            Foo,
            Bar(u8),
        }
        #[derive(Wire)]
        enum New {
            Foo,
            Bar { bar: u8 },
        }
        // Updating a variant in a compatible way is always accepted.
        assert!(check(&View::new::<Old>(), &View::new::<New>()).is_ok());
    }

    #[test]
    fn renaming_variant() {
        #[derive(Wire)]
        enum Old {
            Foo,
            Bar,
        }
        #[derive(Wire)]
        enum New {
            Foo,
            Baz,
        }
        // Renaming a variant is always accepted.
        assert!(check(&View::new::<Old>(), &View::new::<New>()).is_ok());
    }
}
