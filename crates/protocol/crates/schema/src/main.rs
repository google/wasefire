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

use anyhow::{Context, Result, bail, ensure};
use tokio::process::Command;
use wasefire_cli_tools::{cmd, fs};
use wasefire_error::Error;
#[cfg(feature = "host")]
use wasefire_protocol::bundle::Bundle;
use wasefire_protocol::{Api, DESCRIPTORS, Descriptor, Request, Response, VERSION};
use wasefire_wire::schema::{View, ViewEnum, ViewFields};
use wasefire_wire::{Wire, Yoke};

#[tokio::main]
async fn main() -> Result<()> {
    let new = Schema::new();
    new.write().await.context("writing bin")?;
    new.print().await.context("printing txt")?;
    let old = Schema::old().await?;
    check(&old.get().result, &new.result).context("checking result")?;
    check(&old.get().request, &new.request).context("checking request")?;
    check(&old.get().response, &new.response).context("checking response")?;
    check_versions(&old.get().versions, &new.versions).context("checking versions")?;
    #[cfg(feature = "host")]
    check(&old.get().bundle, &new.bundle).context("checking bundle")?;
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

fn check_versions(old: &Versions, new: &Versions) -> Result<()> {
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
                match (old.max, new.max) {
                    (None, Some(new)) if new == old_version => changed = true,
                    (None, Some(_)) => bail!("max version for {tag} must be {old_version}"),
                    (old, new) if old == new => (),
                    _ => bail!("max version changed for {tag}"),
                }
            }
            None => {
                changed = true;
                ensure!(new.min == new_version, "min version for {tag} must be {new_version}");
                ensure!(new.max.is_none(), "max version is defined for {tag}");
            }
        }
    }
    changed |= !old_map.is_empty();
    #[cfg(feature = "host")]
    ensure!(old_map.is_empty(), "deleted tag {}", old_map.keys().next().unwrap());
    match (new_version.checked_sub(old_version), changed) {
        (Some(0), false) | (Some(1), true) => (),
        (Some(0), true) => bail!("version must increase by 1"),
        (Some(1), false) => bail!("version must stay the same"),
        (None, _) => bail!("version cannot decrease"),
        (Some(_), _) => bail!("version can only increase by 1"),
    }
    Ok(())
}

#[derive(Debug, Wire)]
struct Schema<'a> {
    result: View<'a>,   // Result<(), Error>
    request: View<'a>,  // Api<Request>
    response: View<'a>, // Api<Response>
    versions: Versions,
    #[cfg(feature = "host")]
    bundle: View<'a>, // Bundle
}

#[derive(Debug, Wire)]
struct Versions {
    version: u32,
    descriptors: Vec<Descriptor>,
}

impl Schema<'static> {
    fn new() -> Self {
        Schema {
            result: View::new::<Result<(), Error>>(),
            request: View::new::<Api<Request>>(),
            response: View::new::<Api<Response>>(),
            versions: Versions { version: VERSION, descriptors: DESCRIPTORS.to_vec() },
            #[cfg(feature = "host")]
            bundle: View::new::<Bundle>(),
        }
    }

    async fn old() -> Result<Yoke<Schema<'static>>> {
        let base = Self::base()?;
        let mut git = Command::new("git");
        git.args(["show", &format!("{base}:./{SIDE}.bin")]);
        let data = cmd::output(&mut git).await?.stdout.into_boxed_slice();
        // TODO: Remove everything from HERE ...
        #[cfg(feature = "host")]
        {
            if let Ok(x) = wasefire_wire::decode_yoke(data.clone()) {
                return Ok(x);
            }
            let mut data = data.into_vec();
            wasefire_wire::encode_suffix(&mut data, &View::Enum(Vec::new()))?;
            return Ok(wasefire_wire::decode_yoke(data.into_boxed_slice())?);
        }
        #[cfg(feature = "device")]
        // TODO: ... to HERE. This is a one-time migration step.
        Ok(wasefire_wire::decode_yoke(data)?)
    }

    fn base() -> Result<String> {
        use std::env::VarError::*;
        use std::env::var;
        Ok(match var("GITHUB_EVENT_NAME").as_deref() {
            Ok("pull_request") => format!("origin/{}", var("GITHUB_BASE_REF")?),
            Ok("push" | "schedule") => format!("origin/{}", var("GITHUB_REF_NAME")?),
            Ok(x) => bail!("unexpected GITHUB_EVENT_NAME {x:?}"),
            Err(NotPresent) => match var("BASE_REF") {
                Ok(x) => x,
                Err(NotPresent) => "origin/main".to_string(),
                Err(NotUnicode(x)) => bail!("invalid BASE_REF {x:?}"),
            },
            Err(NotUnicode(x)) => bail!("invalid GITHUB_EVENT_NAME {x:?}"),
        })
    }

    async fn write(&self) -> Result<()> {
        fs::write(format!("{SIDE}.bin"), wasefire_wire::encode(self)?).await
    }

    async fn print(&self) -> Result<()> {
        let mut output = String::new();
        writeln!(&mut output, "result: {}", self.result)?;
        writeln!(&mut output, "version: {}", self.versions.version)?;
        let request = get_enum(&self.request).context("in request")?.iter();
        let mut response = get_enum(&self.response).context("in response")?.iter();
        let mut descriptor = self.versions.descriptors.iter();
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
        #[cfg(feature = "host")]
        {
            writeln!(&mut output, "bundle:")?;
            for bundle in get_enum(&self.bundle).context("in bundle")?.iter() {
                let name = bundle.0;
                let tag = bundle.1;
                let bundle = ViewFields(&bundle.2);
                writeln!(&mut output, "{tag} {name}: {bundle}")?;
            }
        }
        fs::write(format!("{SIDE}.txt"), &output).await
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

    macro_rules! build_versions {
        ($ver:literal $($tag:literal [$min:literal - $($max:literal)?])*) => {
            Versions {
                version: $ver,
                descriptors: vec![$(build_versions!([1] $tag [$min - $($max)?])),*],
            }
        };
        ([1] $tag:literal [$min:literal - $($max:literal)?]) => {
            Descriptor {
                tag: $tag,
                versions: wasefire_protocol::Versions {
                    min: $min,
                    max: build_versions!([2] $($max)?),
                },
            }
        };
        ([2]) => (None);
        ([2] $max:literal) => (Some($max));
    }

    #[track_caller]
    fn assert_is_err<T: std::fmt::Debug>(x: Result<T>, m: &'static str) {
        let e = x.unwrap_err();
        match e.downcast_ref::<String>() {
            Some(a) => assert_eq!(a, m),
            None => assert_eq!(e.downcast::<&'static str>().unwrap(), m),
        }
    }

    #[test]
    fn adding_service() {
        let old = build_versions!(41 0 [13 -]);
        // Correct usage.
        let new = build_versions!(42 0 [13 -] 1 [42 -]);
        assert!(check_versions(&old, &new).is_ok());
        // The added service is already deprecated.
        let new = build_versions!(42 0 [13 -] 1 [42 - 42]);
        assert_is_err(check_versions(&old, &new), "max version is defined for 1");
        // The added service uses the wrong version.
        let new = build_versions!(42 0 [13 -] 1 [41 -]);
        assert_is_err(check_versions(&old, &new), "min version for 1 must be 42");
        // The version is not bumped.
        let new = build_versions!(41 0 [13 -] 1 [41 -]);
        assert_is_err(check_versions(&old, &new), "version must increase by 1");
        // The version is decreased.
        let new = build_versions!(40 0 [13 -] 1 [40 -]);
        assert_is_err(check_versions(&old, &new), "version cannot decrease");
        // The version is increased too much.
        let new = build_versions!(43 0 [13 -] 1 [43 -]);
        assert_is_err(check_versions(&old, &new), "version can only increase by 1");
    }

    #[test]
    #[cfg(feature = "device")]
    fn removing_service() {
        let old = build_versions!(41 0 [13 -]);
        // Correct usage.
        let new = build_versions!(42);
        assert!(check_versions(&old, &new).is_ok());
        // The version is not bumped.
        let new = build_versions!(41);
        assert_is_err(check_versions(&old, &new), "version must increase by 1");
        // The version is decreased.
        let new = build_versions!(40);
        assert_is_err(check_versions(&old, &new), "version cannot decrease");
        // The version is increased too much.
        let new = build_versions!(43);
        assert_is_err(check_versions(&old, &new), "version can only increase by 1");
    }

    #[test]
    #[cfg(feature = "host")]
    fn removing_service() {
        let old = build_versions!(41 0 [13 -]);
        // Correct usage.
        let new = build_versions!(42 0 [13 - 41]);
        assert!(check_versions(&old, &new).is_ok());
        // The removed service uses the wrong version.
        let new = build_versions!(42 0 [13 - 42]);
        assert_is_err(check_versions(&old, &new), "max version for 0 must be 41");
        // The version is not bumped.
        let new = build_versions!(41 0 [13 - 41]);
        assert_is_err(check_versions(&old, &new), "version must increase by 1");
        // The version is decreased.
        let new = build_versions!(40 0 [13 - 41]);
        assert_is_err(check_versions(&old, &new), "version cannot decrease");
        // The version is increased too much.
        let new = build_versions!(43 0 [13 - 41]);
        assert_is_err(check_versions(&old, &new), "version can only increase by 1");
    }

    #[test]
    #[cfg(feature = "device")]
    fn unchanging_service() {
        let old = build_versions!(41 0 [13 -]);
        // Correct usage.
        let new = build_versions!(41 0 [13 -]);
        assert!(check_versions(&old, &new).is_ok());
        // The service cannot change min version.
        let new = build_versions!(41 0 [14 -]);
        assert_is_err(check_versions(&old, &new), "min version changed for 0");
        // The service cannot have a max version.
        let new = build_versions!(41 0 [13 - 27]);
        assert_is_err(check_versions(&old, &new), "max version is defined for 0");
        // The version cannot decrease.
        let new = build_versions!(40 0 [13 -]);
        assert_is_err(check_versions(&old, &new), "version cannot decrease");
        // The version cannot increase.
        let new = build_versions!(42 0 [13 -]);
        assert_is_err(check_versions(&old, &new), "version must stay the same");
    }

    #[test]
    #[cfg(feature = "host")]
    fn unchanging_service() {
        let old = build_versions!(41 0 [13 - 27]);
        // Correct usage.
        let new = build_versions!(41 0 [13 - 27]);
        assert!(check_versions(&old, &new).is_ok());
        // The service cannot change min version.
        let new = build_versions!(41 0 [14 - 27]);
        assert_is_err(check_versions(&old, &new), "min version changed for 0");
        // The service cannot change max version.
        let new = build_versions!(41 0 [13 - 26]);
        assert_is_err(check_versions(&old, &new), "max version changed for 0");
        // The service cannot remove max version.
        let new = build_versions!(41 0 [13 -]);
        assert_is_err(check_versions(&old, &new), "max version changed for 0");
        // The version cannot decrease.
        let new = build_versions!(40 0 [13 - 27]);
        assert_is_err(check_versions(&old, &new), "version cannot decrease");
        // The version cannot increase.
        let new = build_versions!(42 0 [13 - 27]);
        assert_is_err(check_versions(&old, &new), "version must stay the same");
    }
}
