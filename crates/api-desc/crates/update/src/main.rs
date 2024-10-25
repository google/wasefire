// Copyright 2022 Google LLC
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

use std::fs::File;
use std::io::{stdout, Write};

use anyhow::Result;
use clap::{Parser, ValueEnum};
use wasefire_applet_api_desc::Api;

#[derive(Parser)]
struct Flags {
    /// Output file to generate the API [default: stdout].
    #[clap(long)]
    output: Option<String>,

    /// Language for which to generate the API.
    #[clap(long)]
    lang: Lang,
}

#[test]
fn flags() {
    <Flags as clap::CommandFactory>::command().debug_assert();
}

#[derive(Copy, Clone, ValueEnum)]
pub enum Lang {
    C,
    Assemblyscript,
}

impl From<Lang> for wasefire_applet_api_desc::Lang {
    fn from(value: Lang) -> Self {
        match value {
            Lang::C => wasefire_applet_api_desc::Lang::C,
            Lang::Assemblyscript => wasefire_applet_api_desc::Lang::Assemblyscript,
        }
    }
}

fn main() -> Result<()> {
    let flags = Flags::parse();
    let mut output: Box<dyn Write> = match flags.output {
        Some(path) => Box::new(File::create(path)?),
        None => Box::new(stdout()),
    };
    output.write_all(
        br#"// Copyright 2022 Google LLC
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

"#,
    )?;
    Api::default().wasm(&mut output, flags.lang.into())?;
    Ok(())
}
