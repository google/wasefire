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
use wasefire_protocol::{Api, Request, Response};
use wasefire_wire::schema::{View, ViewEnum, ViewFields};
use wasefire_wire::Wire;
use yoke::{Yoke, Yokeable};

fn main() -> Result<()> {
    let new = Schema::new();
    new.write().context("writing bin")?;
    new.print().context("printing txt")?;
    let old = Schema::old()?;
    check(&old.get().request, &new.request).context("checking request")?;
    check(&old.get().response, &new.response).context("checking response")?;
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
    #[cfg(feature = "full")]
    ensure!(old_map.is_empty(), "deleted tag {}", old_map.keys().next().unwrap());
    Ok(())
}

#[derive(Debug, Wire, Yokeable)]
struct Schema<'a> {
    request: View<'a>,
    response: View<'a>,
}

type OwnedSchema = Yoke<Schema<'static>, Box<[u8]>>;

impl Schema<'static> {
    fn new() -> Self {
        Schema { request: View::new::<Api<Request>>(), response: View::new::<Api<Response>>() }
    }

    fn old() -> Result<OwnedSchema> {
        let mut git = Command::new("git");
        git.args(["show", &format!("origin/main:./{SIDE}.bin")]);
        let data = cmd::output(&mut git)?.stdout.into_boxed_slice();
        Yoke::try_attach_to_cart(data, |data| Ok(wasefire_wire::decode(data)?))
    }

    fn write(&self) -> Result<()> {
        fs::write(format!("{SIDE}.bin"), wasefire_wire::encode(self)?)
    }

    fn print(&self) -> Result<()> {
        let mut output = String::new();
        let request = get_enum(&self.request).context("in request")?;
        let response = get_enum(&self.response).context("in response")?;
        ensure!(request.len() == response.len(), "API length mismatch");
        for (request, response) in request.iter().zip(response.iter()) {
            ensure!(request.0 == response.0, "API name mismatch");
            ensure!(request.1 == response.1, "API tag mismatch");
            let name = request.0;
            let tag = request.1;
            let request = ViewFields(&request.2);
            let response = ViewFields(&response.2);
            writeln!(&mut output, "{name}={tag}: {request} -> {response}")?;
        }
        fs::write(format!("{SIDE}.txt"), &output)
    }
}

#[cfg(feature = "full")]
const SIDE: &str = "host";
#[cfg(not(feature = "full"))]
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
        #[cfg(not(feature = "full"))]
        assert!(check(&View::new::<Old>(), &View::new::<New>()).is_ok());
        // But rejected on the host.
        #[cfg(feature = "full")]
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
