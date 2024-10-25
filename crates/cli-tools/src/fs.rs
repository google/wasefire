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

//! Wrappers around `tokio::fs` with descriptive errors.

use std::fs::Metadata;
use std::future::Future;
use std::io::ErrorKind;
use std::path::{Component, Path, PathBuf};

use anyhow::{Context, Result};
use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::fs::{File, OpenOptions, ReadDir};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub async fn canonicalize(path: impl AsRef<Path>) -> Result<PathBuf> {
    let name = path.as_ref().display();
    tokio::fs::canonicalize(path.as_ref()).await.with_context(|| format!("canonicalizing {name}"))
}

pub async fn try_canonicalize(path: impl AsRef<Path>) -> Result<PathBuf> {
    match canonicalize(&path).await {
        Err(x) if x.downcast_ref::<std::io::Error>().unwrap().kind() == ErrorKind::NotFound => {
            // We could try to canonicalize the existing prefix.
            Ok(path.as_ref().to_path_buf())
        }
        x => x,
    }
}

pub async fn try_relative(base: impl AsRef<Path>, path: impl AsRef<Path>) -> Result<PathBuf> {
    let base = try_canonicalize(&base).await?;
    let path = try_canonicalize(&path).await?;
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

pub async fn copy(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<u64> {
    let src = from.as_ref().display();
    let dst = to.as_ref().display();
    create_parent(to.as_ref()).await?;
    debug!("cp {src:?} {dst:?}");
    tokio::fs::copy(from.as_ref(), to.as_ref())
        .await
        .with_context(|| format!("copying {src} to {dst}"))
}

pub async fn copy_if_changed(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<bool> {
    let dst_orig = dst.as_ref().with_added_extension("orig");
    let mut changed = !exists(&dst).await
        || metadata(&dst).await?.modified()? < metadata(&src).await?.modified()?
        || !exists(&dst_orig).await;
    if !changed {
        let src_data = tokio::fs::read(&src).await?;
        let dst_data = tokio::fs::read(&dst_orig).await?;
        changed = src_data != dst_data;
    }
    if changed {
        copy(&src, dst).await?;
        tokio::fs::copy(src, dst_orig).await?;
    }
    Ok(changed)
}

pub async fn create_dir_all(path: impl AsRef<Path>) -> Result<()> {
    let name = path.as_ref().display();
    if exists(path.as_ref()).await {
        return Ok(());
    }
    debug!("mkdir -p {name:?}");
    tokio::fs::create_dir_all(path.as_ref()).await.with_context(|| format!("creating {name}"))
}

pub async fn create_parent(path: impl AsRef<Path>) -> Result<()> {
    if let Some(parent) = path.as_ref().parent() {
        if !parent.as_os_str().is_empty() {
            create_dir_all(parent).await?;
        }
    }
    Ok(())
}

pub async fn exists(path: impl AsRef<Path>) -> bool {
    tokio::fs::try_exists(path).await.ok() == Some(true)
}

pub async fn metadata(path: impl AsRef<Path>) -> Result<Metadata> {
    let name = path.as_ref().display();
    tokio::fs::metadata(path.as_ref()).await.with_context(|| format!("reading {name} metadata"))
}

pub async fn read(path: impl AsRef<Path>) -> Result<Vec<u8>> {
    let name = path.as_ref().display();
    debug!("read < {name:?}");
    tokio::fs::read(path.as_ref()).await.with_context(|| format!("reading {name}"))
}

pub async fn read_dir(path: impl AsRef<Path>) -> Result<ReadDir> {
    let dir = path.as_ref().display();
    debug!("walk {dir:?}");
    tokio::fs::read_dir(path.as_ref()).await.with_context(|| format!("walking {dir}"))
}

pub async fn read_toml<T: DeserializeOwned>(path: impl AsRef<Path>) -> Result<T> {
    let name = path.as_ref().display();
    let contents = read(path.as_ref()).await?;
    let data = String::from_utf8(contents).with_context(|| format!("reading {name}"))?;
    toml::from_str(&data).with_context(|| format!("parsing {name}"))
}

pub async fn read_stdin() -> Result<Vec<u8>> {
    let mut data = Vec::new();
    tokio::io::stdin().read_to_end(&mut data).await.context("reading from stdin")?;
    Ok(data)
}

pub async fn remove_dir_all(path: impl AsRef<Path>) -> Result<()> {
    let name = path.as_ref().display();
    debug!("rm -r {name:?}");
    tokio::fs::remove_dir_all(path.as_ref()).await.with_context(|| format!("removing {name}"))
}

pub async fn remove_file(path: impl AsRef<Path>) -> Result<()> {
    let name = path.as_ref().display();
    debug!("rm {name:?}");
    tokio::fs::remove_file(path.as_ref()).await.with_context(|| format!("removing {name}"))
}

pub async fn rename(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<()> {
    let src = from.as_ref().display();
    let dst = to.as_ref().display();
    create_parent(to.as_ref()).await?;
    debug!("mv {src:?} {dst:?}");
    tokio::fs::rename(from.as_ref(), to.as_ref())
        .await
        .with_context(|| format!("renaming {src} to {dst}"))
}

pub async fn touch(path: impl AsRef<Path>) -> Result<()> {
    if exists(path.as_ref()).await {
        return Ok(());
    }
    write(path, "").await
}

pub struct WriteParams<P: AsRef<Path>> {
    path: P,
    options: OpenOptions,
}

impl<P: AsRef<Path>> WriteParams<P> {
    pub fn new(path: P) -> Self {
        WriteParams { path, options: OpenOptions::new() }
    }
    pub fn options(&mut self) -> &mut OpenOptions {
        &mut self.options
    }
}

pub trait WriteFile {
    fn path(&self) -> &Path;
    fn open(self) -> impl Future<Output = Result<File>>;
}

impl<P: AsRef<Path>> WriteFile for WriteParams<P> {
    fn path(&self) -> &Path {
        self.path.as_ref()
    }
    async fn open(self) -> Result<File> {
        (&self).open().await
    }
}

impl<P: AsRef<Path>> WriteFile for &WriteParams<P> {
    fn path(&self) -> &Path {
        (*self).path()
    }
    async fn open(self) -> Result<File> {
        Ok(self.options.open(&self.path).await?)
    }
}

impl<P: AsRef<Path>> WriteFile for P {
    fn path(&self) -> &Path {
        self.as_ref()
    }
    async fn open(self) -> Result<File> {
        let mut params = WriteParams::new(self);
        params.options().write(true).create(true).truncate(true);
        params.open().await
    }
}

pub async fn write(file: impl WriteFile, contents: impl AsRef<[u8]>) -> Result<()> {
    let name = format!("{}", file.path().display());
    let contents = contents.as_ref();
    create_parent(file.path()).await?;
    debug!("write > {name:?}");
    let mut file = file.open().await.with_context(|| format!("creating {name}"))?;
    file.write_all(contents).await.with_context(|| format!("writing {name}"))?;
    file.flush().await.with_context(|| format!("flushing {name}"))?;
    Ok(())
}

pub async fn write_toml<T: Serialize>(path: impl AsRef<Path>, contents: &T) -> Result<()> {
    let name = path.as_ref().display();
    let contents = toml::to_string(contents).with_context(|| format!("displaying {name}"))?;
    write(path.as_ref(), contents).await?;
    Ok(())
}

pub async fn write_stdout(contents: impl AsRef<[u8]>) -> Result<()> {
    let contents = contents.as_ref();
    tokio::io::stdout().write_all(contents).await.context("writing to stdout")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn try_relative_ok() {
        #[track_caller]
        async fn test(base: &str, path: &str, res: &str) {
            assert_eq!(try_relative(base, path).await.ok(), Some(PathBuf::from(res)));
        }
        test("/foo/bar", "/foo", "..").await;
        test("/foo/bar", "/foo/baz/qux", "../baz/qux").await;
        test("/foo/bar", "/foo/bar/qux", "qux").await;
    }
}
