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

use std::fs::{Metadata, ReadDir};
use std::io::{ErrorKind, Read, Write};
use std::path::{Component, Path, PathBuf};

use anyhow::{Context, Result};
use serde::de::DeserializeOwned;
use serde::Serialize;

pub fn canonicalize(path: impl AsRef<Path>) -> Result<PathBuf> {
    let name = path.as_ref().display();
    std::fs::canonicalize(path.as_ref()).with_context(|| format!("canonicalizing {name}"))
}

pub fn try_canonicalize(path: impl AsRef<Path>) -> Result<PathBuf> {
    match canonicalize(&path) {
        Err(x) if x.downcast_ref::<std::io::Error>().unwrap().kind() == ErrorKind::NotFound => {
            // We could try to canonicalize the existing prefix.
            Ok(path.as_ref().to_path_buf())
        }
        x => x,
    }
}

pub fn try_relative(base: impl AsRef<Path>, path: impl AsRef<Path>) -> Result<PathBuf> {
    let base = try_canonicalize(&base)?;
    let path = try_canonicalize(&path)?;
    let mut base = base.components().peekable();
    let mut path = path.components().peekable();
    // Advance the common prefix.
    while matches!((base.peek(), path.peek()), (Some(x), Some(y)) if x == y) {
        base.next();
        path.next();
    }
    // Add necessary parent directory.
    let mut result = PathBuf::new();
    while base.next().is_some() {
        result.push(Component::ParentDir);
    }
    // Add path suffix.
    result.extend(path);
    Ok(result)
}

pub fn copy(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<u64> {
    let src = from.as_ref().display();
    let dst = to.as_ref().display();
    create_parent(to.as_ref())?;
    debug!("cp {src:?} {dst:?}");
    std::fs::copy(from.as_ref(), to.as_ref()).with_context(|| format!("copying {src} to {dst}"))
}

pub fn copy_if_changed(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<bool> {
    let dst_orig = dst.as_ref().with_added_extension("orig");
    let mut changed = !exists(&dst)
        || metadata(&dst)?.modified()? < metadata(&src)?.modified()?
        || !exists(&dst_orig);
    if !changed {
        let src_data = std::fs::read(&src)?;
        let dst_data = std::fs::read(&dst_orig)?;
        changed = src_data != dst_data;
    }
    if changed {
        copy(&src, dst)?;
        std::fs::copy(src, dst_orig)?;
    }
    Ok(changed)
}

pub fn create_dir_all(path: impl AsRef<Path>) -> Result<()> {
    let name = path.as_ref().display();
    if exists(path.as_ref()) {
        return Ok(());
    }
    debug!("mkdir -p {name:?}");
    std::fs::create_dir_all(path.as_ref()).with_context(|| format!("creating {name}"))
}

pub fn create_parent(path: impl AsRef<Path>) -> Result<()> {
    if let Some(parent) = path.as_ref().parent() {
        if !parent.as_os_str().is_empty() {
            create_dir_all(parent)?;
        }
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
    debug!("read < {name:?}");
    std::fs::read(path.as_ref()).with_context(|| format!("reading {name}"))
}

pub fn read_dir(path: impl AsRef<Path>) -> Result<ReadDir> {
    let dir = path.as_ref().display();
    debug!("walk {dir:?}");
    std::fs::read_dir(path.as_ref()).with_context(|| format!("walking {dir}"))
}

pub fn read_toml<T: DeserializeOwned>(path: impl AsRef<Path>) -> Result<T> {
    let name = path.as_ref().display();
    let contents = read(path.as_ref())?;
    let data = String::from_utf8(contents).with_context(|| format!("reading {name}"))?;
    toml::from_str(&data).with_context(|| format!("parsing {name}"))
}

pub fn read_stdin() -> Result<Vec<u8>> {
    let mut data = Vec::new();
    std::io::stdin().read_to_end(&mut data).context("reading from stdin")?;
    Ok(data)
}

pub fn remove_file(path: impl AsRef<Path>) -> Result<()> {
    let name = path.as_ref().display();
    debug!("rm {name:?}");
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
    debug!("write > {name:?}");
    std::fs::write(path.as_ref(), contents).with_context(|| format!("writing {name}"))?;
    Ok(())
}

pub fn write_toml<T: Serialize>(path: impl AsRef<Path>, contents: &T) -> Result<()> {
    let name = path.as_ref().display();
    let contents = toml::to_string(contents).with_context(|| format!("displaying {name}"))?;
    write(path.as_ref(), contents)?;
    Ok(())
}

pub fn write_stdout(contents: impl AsRef<[u8]>) -> Result<()> {
    let contents = contents.as_ref();
    std::io::stdout().write_all(contents).context("writing to stdout")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_relative_ok() {
        #[track_caller]
        fn test(base: &str, path: &str, res: &str) {
            assert_eq!(try_relative(base, path).ok(), Some(PathBuf::from(res)));
        }
        test("/foo/bar", "/foo", "..");
        test("/foo/bar", "/foo/baz/qux", "../baz/qux");
        test("/foo/bar", "/foo/bar/qux", "qux");
    }
}
