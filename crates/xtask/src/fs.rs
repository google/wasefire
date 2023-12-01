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

//! Wrappers around `std::fs` with descriptive errors.

use std::fs::Metadata;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

pub fn canonicalize(path: impl AsRef<Path>) -> Result<PathBuf> {
    let name = path.as_ref().display();
    std::fs::canonicalize(path.as_ref()).with_context(|| format!("canonicalizing {name}"))
}

pub fn copy(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<u64> {
    let src = from.as_ref().display();
    let dst = to.as_ref().display();
    create_parent(to.as_ref())?;
    std::fs::copy(from.as_ref(), to.as_ref()).with_context(|| format!("copying {src} to {dst}"))
}

fn create_dir_all(path: impl AsRef<Path>) -> Result<()> {
    let name = path.as_ref().display();
    std::fs::create_dir_all(path.as_ref()).with_context(|| format!("creating {name}"))
}

fn create_parent(path: impl AsRef<Path>) -> Result<()> {
    if let Some(parent) = path.as_ref().parent() {
        create_dir_all(parent)?;
    }
    Ok(())
}

pub fn exists(path: impl AsRef<Path>) -> bool {
    path.as_ref().exists()
}

pub fn metadata(path: impl AsRef<Path>) -> Result<Metadata> {
    let name = path.as_ref().display();
    std::fs::metadata(path.as_ref()).with_context(|| format!("reading {name} metadata"))
}

pub fn read(path: impl AsRef<Path>) -> Result<Vec<u8>> {
    let name = path.as_ref().display();
    std::fs::read(path.as_ref()).with_context(|| format!("reading {name}"))
}

pub fn remove_file(path: impl AsRef<Path>) -> Result<()> {
    let name = path.as_ref().display();
    std::fs::remove_file(path.as_ref()).with_context(|| format!("removing {name}"))
}

pub fn touch(path: impl AsRef<Path>) -> Result<()> {
    if exists(path.as_ref()) {
        return Ok(());
    }
    write(path, "")
}

pub fn write(path: impl AsRef<Path>, contents: impl AsRef<[u8]>) -> Result<()> {
    let name = path.as_ref().display();
    let contents = contents.as_ref();
    create_parent(path.as_ref())?;
    std::fs::write(path.as_ref(), contents).with_context(|| format!("writing {name}"))?;
    Ok(())
}
