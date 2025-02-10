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

use std::path::PathBuf;

use anyhow::{Result, ensure};
use cargo_metadata::{Metadata, MetadataCommand};

pub async fn metadata(dir: impl Into<PathBuf>) -> Result<Metadata> {
    let dir = dir.into();
    let metadata =
        tokio::task::spawn_blocking(|| MetadataCommand::new().current_dir(dir).no_deps().exec())
            .await??;
    ensure!(metadata.packages.len() == 1, "not exactly one package");
    Ok(metadata)
}
