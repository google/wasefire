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

use std::process::Stdio;
use std::time::Duration;

use anyhow::{Context, Result};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};
use tokio::process::{ChildStdin, Command};
use wasefire_cli_tools::cmd;

use crate::{Flash, MainOptions, ensure_command, wrap_command};

pub async fn build(elf: &str) -> Result<String> {
    // Copy ELF to binary.
    let bin = format!("{elf}.bin");
    let mut objcopy = wrap_command().await?;
    objcopy.args(["rust-objcopy", "--output-target", "binary", elf, &bin]);
    cmd::execute(&mut objcopy).await?;

    // Prepare the pre-signing artifact.
    let presign = format!("{elf}.appkey_prod_0.pre-signing");
    let mut opentitan = Command::new("opentitantool");
    opentitan.args(["--quiet", "image", "manifest", "update", &bin]);
    opentitan.arg("--manifest=crates/runner-opentitan/manifest.json");
    opentitan.arg(format!("--output={presign}"));
    opentitan.arg(
        "--ecdsa-key=third_party/lowRISC/opentitan/sw/device/silicon_creator/rom_ext/prodc/keys/\
         appkey_prod_0.der",
    );
    cmd::execute(&mut opentitan).await?;

    // Compute the digest of the artifact.
    let digest = format!("{elf}.appkey_prod_0.digest");
    let mut opentitan = Command::new("opentitantool");
    opentitan.args(["--quiet", "image", "digest"]);
    opentitan.arg(format!("--bin={digest}"));
    opentitan.arg(&presign);
    cmd::execute(&mut opentitan).await?;

    // Sign the digest.
    let ecdsasig = format!("{elf}.appkey_prod_0.ecdsa-sig");
    let mut hsmtool = Command::new("hsmtool");
    hsmtool.args([
        "--quiet",
        "--lockfile=/tmp/hsmtool.lock",
        "--profile=earlgrey_z1_prodc",
        "ecdsa",
        "sign",
        "--little-endian",
        "--format=sha256-hash",
        "--label=appkey_prod_0",
    ]);
    hsmtool.arg(format!("--output={ecdsasig}"));
    hsmtool.arg(&digest);
    cmd::execute(&mut hsmtool).await?;

    // Attach the signature.
    let signed = format!("{elf}.appkey_prod_0.signed.bin");
    let mut opentitan = Command::new("opentitantool");
    opentitan.args(["--quiet", "image", "manifest", "update", "--update-length=false"]);
    opentitan.arg(format!("--output={signed}"));
    opentitan.arg(&presign);
    opentitan.arg(format!("--ecdsa-signature={ecdsasig}"));
    cmd::execute(&mut opentitan).await?;

    // Assemble the image.
    let img = format!("{elf}.img");
    let mut opentitan = Command::new("opentitantool");
    opentitan.args(["image", "assemble", "--mirror=false"]);
    opentitan.arg(format!("--output={img}"));
    opentitan.arg(
        "third_party/lowRISC/opentitan/sw/device/silicon_creator/rom_ext/prodc/binaries/\
         rom_ext_real_prod_signed_slot_a_silicon_creator.signed.bin@0",
    );
    opentitan.arg(format!("{signed}@0x10000"));
    cmd::execute(&mut opentitan).await?;
    Ok(img)
}

pub async fn truncate(img_path: &str) -> Result<()> {
    let mut img_file = File::options().read(true).write(true).open(img_path).await?;
    img_file.seek(std::io::SeekFrom::Start(0x340)).await?;
    let mut length = [0; 4];
    img_file.read_exact(&mut length).await?;
    img_file.set_len(u32::from_le_bytes(length) as u64).await?;
    Ok(())
}

pub async fn execute(main: &MainOptions, flash: &Flash, elf: &str) -> Result<!> {
    // Fail early missing HYPERDEBUG_SERIAL or defmt-print.
    let serial = std::env::var("HYPERDEBUG_SERIAL")
        .context("HYPERDEBUG_SERIAL must be set to a HyperDebug serial")?;
    ensure_command(&["defmt-print"]).await?;

    // Build the image.
    let img = build(elf).await?;

    // Connect to serial.
    let port = format!("/dev/serial/by-id/usb-Google_LLC_HyperDebug_CMSIS-DAP_{serial}-if03-port0");
    let mut input = serialport::new(&port, 115200).timeout(Duration::from_secs(3600)).open()?;

    // Bootstrap the image.
    let mut opentitan = Command::new("opentitantool");
    opentitan.args(["--interface=teacup", "--exec=transport init", "bootstrap", &img]);
    cmd::execute(&mut opentitan).await?;

    // Discard everything until we recognize the ROM_EXT initial message.
    const PATTERN: &str = "Starting ROM_EXT";
    let mut pos = 0;
    while pos < PATTERN.len() {
        let mut byte = 0;
        input.read_exact(std::slice::from_mut(&mut byte))?;
        if byte == PATTERN.as_bytes()[pos] {
            pos += 1;
        } else {
            pos = 0;
        }
    }
    print!("{PATTERN}");

    // Print with defmt.
    let mut defmt: Option<ChildStdin> = match main.release {
        true => None,
        false => {
            let mut defmt = wrap_command().await?;
            defmt.arg("defmt-print");
            defmt.args(&flash.defmt_print);
            defmt.args(["-e", elf]).stdin(Stdio::piped());
            Some(defmt.spawn()?.stdin.take().unwrap())
        }
    };
    let mut stdout = tokio::io::stdout();
    #[derive(Copy, Clone, PartialEq, Eq)]
    enum State {
        Stdout,
        Control,
        Defmt,
    }
    use State::*;
    let mut state = Stdout;
    loop {
        let mut byte = 0;
        input.read_exact(std::slice::from_mut(&mut byte))?;
        if state == Stdout && byte == 0 {
            // Switch from stdout to defmt.
            stdout.flush().await?;
            state = Defmt;
        }
        // Write non-control byte.
        match (state, &mut defmt) {
            (Stdout, _) => stdout.write_all(&[byte]).await?,
            (Defmt, Some(defmt)) => defmt.write_all(&[byte]).await?,
            (Defmt, None) => (),
            (Control, _) => (),
        }
        // Handle control bytes and update state.
        match (state, byte) {
            (Control, 0 | 13) => state = Stdout, // must be a reset
            (Control, 1 | 2) => {
                if let Some(defmt) = defmt.as_mut() {
                    defmt.flush().await?;
                }
                std::process::exit((byte - 1) as i32);
            }
            (Control, 3) => state = Defmt,
            (Control, _) => panic!("unsupported control byte {byte}"),
            (Stdout | Defmt, 0) => state = Control,
            (Stdout | Defmt, _) => (),
        }
    }
}
