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

use anyhow::{Context, Result, ensure};
use serialport::SerialPort;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};
use tokio::process::Command;
use wasefire_cli_tools::{cmd, fs};

use crate::{AttachOptions, MainOptions, ensure_command, wrap_command};

pub const APPKEY: &str = "gb00-app-key-prod-0";

pub async fn build(elf: &str) -> Result<String> {
    let prov_exts_dir = std::env::var("PROV_EXTS_DIR")
        .context("PROV_EXTS_DIR must be set to ot-provisioning-google-skus/skus")?;

    // Copy ELF to binary.
    let bin = format!("{elf}.bin");
    let mut objcopy = wrap_command().await?;
    objcopy.args(["rust-objcopy", "--output-target=binary", elf, &bin]);
    cmd::execute(&mut objcopy).await?;

    // Prepare the pre-signing artifact.
    let presign = format!("{elf}.{APPKEY}.pre-signing");
    let mut opentitan = Command::new("opentitantool");
    opentitan.args(["--quiet", "image", "manifest", "update", &bin]);
    opentitan.arg("--manifest=crates/runner-opentitan/manifest.json");
    opentitan.arg("--domain=None");
    opentitan.arg(format!("--output={presign}"));
    opentitan.arg(format!(
        "--ecdsa-key={prov_exts_dir}/shared/rom_ext/gb/keys/gb00-app-key-prod-0.pub.der"
    ));
    cmd::execute(&mut opentitan).await?;

    // Compute the digest of the artifact.
    let digest = format!("{elf}.{APPKEY}.digest");
    let mut opentitan = Command::new("opentitantool");
    opentitan.args(["--quiet", "image", "digest"]);
    opentitan.arg(format!("--bin={digest}"));
    opentitan.arg(&presign);
    cmd::execute(&mut opentitan).await?;

    // Sign the digest.
    let ecdsasig = format!("{elf}.{APPKEY}.ecdsa-sig");
    let mut hsmtool = Command::new("hsmtool");
    hsmtool.args([
        "--quiet",
        "--lockfile=/tmp/hsmtool.lock",
        "--profile=earlgrey_a1_gb_owner",
        "ecdsa",
        "sign",
        "--little-endian",
        "--format=sha256-hash",
    ]);
    hsmtool.arg(format!("--label={APPKEY}"));
    hsmtool.arg(format!("--output={ecdsasig}"));
    hsmtool.arg(&digest);
    cmd::execute(&mut hsmtool).await?;

    // Attach the signature.
    let signed = format!("{elf}.{APPKEY}.signed.bin");
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
    opentitan.arg(format!(
        "{prov_exts_dir}/shared/rom_ext/gb/binaries/\
         rom_ext_dice_x509_rescue_xmodem_prod_slot_virtual_silicon_creator.signed.bin@0"
    ));
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

pub async fn execute(main: &MainOptions, attach_options: &AttachOptions, elf: &str) -> Result<!> {
    // Build the image.
    let img = build(elf).await?;

    // Connect to serial (to not miss anything).
    let input = connect().await?;

    // Bootstrap the image.
    let mut opentitan = Command::new("opentitantool");
    opentitan.args(["--interface=teacup-bga69", "--exec=transport init"]);
    opentitan.args(["bootstrap", "--speed=8000000", &img]);
    cmd::execute(&mut opentitan).await?;

    // Read the serial.
    attach(Some(input), main, attach_options, elf).await
}

pub async fn attach(
    input: Option<Box<dyn SerialPort>>, main: &MainOptions, options: &AttachOptions, elf: &str,
) -> Result<!> {
    let (wait, mut input) = match input {
        Some(x) => (true, x),
        None => (false, connect().await?),
    };
    let delimit = |msg: &str| println!("\x1b[1;35m{msg}\x1b[m");
    delimit("Attached to OpenTitan serial.");

    if wait {
        // Discard everything until we recognize the OpenTitan initial message.
        const PATTERN: &str = "OpenTitan:";
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
    }

    // Print with defmt.
    let spawn_defmt = || async {
        anyhow::Ok(match main.release {
            true => None,
            false => {
                let mut defmt = wrap_command().await?;
                defmt.arg("defmt-print");
                defmt.args(&options.defmt_print);
                defmt.args(["-e", elf]).stdin(Stdio::piped());
                Some(defmt.spawn()?.stdin.take().unwrap())
            }
        })
    };
    let mut defmt = spawn_defmt().await?;
    let mut stdout = tokio::io::stdout();
    #[derive(Copy, Clone, PartialEq, Eq)]
    enum State {
        Stdout,
        Control,
        Defmt,
    }
    use State::*;
    let mut state = if wait { Stdout } else { Control };
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
            (Control, 1 | 2) => {
                if let Some(defmt) = defmt.as_mut() {
                    defmt.flush().await?;
                }
                std::process::exit((byte - 1) as i32);
            }
            (Control, 3) => state = Defmt,
            (Control, _) => {
                // Don't print the warning when it looks like a reset.
                if !matches!(byte, 0 | b'\r' | b'O') {
                    log::warn!("unsupported control byte {byte:#02x}");
                }
                delimit("Reset detected (restarting defmt).");
                state = Stdout;
                stdout.write_all(&[byte]).await?;
                defmt = spawn_defmt().await?;
            }
            (Stdout | Defmt, 0) => state = Control,
            (Stdout | Defmt, _) => (),
        }
    }
}

async fn connect() -> Result<Box<dyn SerialPort>> {
    let serial = std::env::var("HYPERDEBUG_SERIAL")
        .context("HYPERDEBUG_SERIAL must be set to a HyperDebug serial")?;
    let port = format!("/dev/serial/by-id/usb-Google_LLC_HyperDebug_CMSIS-DAP_{serial}-if03-port0");
    ensure!(fs::exists(&port).await, "HyperDebug {serial} not found. Is it connected?");
    ensure_command(&["defmt-print"]).await?;
    Ok(serialport::new(&port, 115200).timeout(Duration::from_secs(3600)).open()?)
}
