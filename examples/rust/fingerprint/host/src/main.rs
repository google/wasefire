// Copyright 2025 Google LLC
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

#![feature(exit_status_error)]

use std::borrow::Cow;
use std::io::Write;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;
use std::time::Duration;

use anyhow::{Result, bail};
use clap::Parser;
use common::{Request, Response};
use data_encoding::HEXLOWER_PERMISSIVE as HEX;
use wasefire_cli_tools::action::ConnectionOptions;
use wasefire_error::Error;
use wasefire_protocol::applet::{self, AppletId};
use wasefire_protocol::{self as service, ConnectionExt as _, DynDevice};
use wasefire_wire::{Wire, Yoke};

#[derive(Parser)]
struct Flags {
    #[command(flatten)]
    options: ConnectionOptions,
    #[clap(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand)]
enum Command {
    /// Captures an image.
    Capture,

    /// Enrolls a finger and prints its template.
    Enroll,

    /// Identifies a finger.
    Identify {
        /// Template to identify against (all templates otherwise).
        id: Option<String>,
    },

    /// Deletes a template.
    Delete {
        /// Template to delete (all templates otherwise).
        id: Option<String>,
    },

    /// Lists all templates.
    List,

    /// Prints each time a finger is detected.
    Detect,
}

macro_rules! call {
    ($x:ident($c:expr): $i:tt -> $o:tt) => {
        call!(1, $c, $x, $i, $o)
    };
    (1, $c:expr, $x:ident, (), $o:tt) => {
        call!(2, $c, $x, Request::$x, $o)
    };
    (1, $c:expr, $x:ident, ($i:expr), $o:tt) => {
        call!(2, $c, $x, Request::$x($i), $o)
    };
    (2, $c:expr, $x:ident, $i:expr, ()) => {
        call!(3, $c, $x, $i, |x| match x {
            Response::$x => Ok(()),
            _ => Err(x),
        })
    };
    (2, $c:expr, $x:ident, $i:expr, (_)) => {
        call!(3, $c, $x, $i, |x| match x {
            Response::$x(x) => Ok(x),
            _ => Err(x),
        })
    };
    (3, $c:expr, $x:ident, $i:expr, $o:expr) => {
        call($c, stringify!($x), &$i, $o).await?
    };
}

#[tokio::main]
async fn main() -> Result<()> {
    let flags = Flags::parse();
    let device = flags.options.connect().await?;
    let stop = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGINT, stop.clone())?;
    macro_rules! loop_stop {
        ($($body:tt)*) => {
            loop {
                if stop.load(Relaxed) {
                    call!(Reset(&device): () -> ()).get();
                    return Ok(());
                }
                $($body)*
            }

        };
    }
    match flags.command {
        Command::Capture => {
            call!(CaptureStart(&device): () -> ()).get();
            let mut stdout = std::io::stdout().lock();
            write!(stdout, "Waiting for touch").unwrap();
            let width = loop_stop! {
                let width = call!(CaptureDone(&device): () -> (_));
                match width.try_map(|x| x.ok_or(())) {
                    Ok(width) => break *width.get(),
                    Err(()) => tokio::time::sleep(Duration::from_millis(300)).await,
                }
                write!(stdout, ".").unwrap();
                stdout.flush().unwrap();
            };
            drop(stdout);
            println!("\nSaved as image.raw and image.png");
            let data = call_raw(&device, &Request::CaptureImage).await?;
            tokio::fs::write("image.raw", data.get()).await?;
            let height = data.get().len() / width;
            let size = format!("{width}x{height}");
            let mut convert = tokio::process::Command::new("convert");
            convert.args(["-size", &size, "-depth", "8", "gray:image.raw", "image.png"]);
            convert.status().await?.exit_ok()?;
        }
        Command::Enroll => {
            call!(EnrollStart(&device): () -> ()).get();
            let mut stdout = std::io::stdout().lock();
            loop_stop! {
                match call!(EnrollDone(&device): () -> (_)).try_map(|x| x) {
                    Ok(id) => {
                        drop(stdout);
                        println!(
                            "\r\x1b[KEnroll finger at template id {}",
                            HEX.encode_display(id.get())
                        );
                        break;
                    }
                    Err((detected, remaining)) => {
                        let percent = detected * 100 / (detected + remaining.unwrap_or(1));
                        write!(stdout, "\r\x1b[KWaiting for touch ({percent}%).").unwrap();
                        stdout.flush().unwrap();
                        tokio::time::sleep(Duration::from_millis(300)).await;
                    }
                }
            }
        }
        Command::Identify { id } => {
            let id = id.map(|x| HEX.decode(x.as_bytes())).transpose()?;
            call!(IdentifyStart(&device): (id) -> ()).get();
            let mut stdout = std::io::stdout().lock();
            write!(stdout, "Waiting for touch").unwrap();
            let result = loop_stop! {
                let result = call!(IdentifyDone(&device): () -> (_));
                match result.try_map(|x| x.ok_or(())) {
                    Ok(result) => break result,
                    Err(()) => tokio::time::sleep(Duration::from_millis(300)).await,
                }
                write!(stdout, ".").unwrap();
                stdout.flush().unwrap();
            };
            drop(stdout);
            println!();
            match result.try_map(|x| x.ok_or(())) {
                Ok(id) => println!("Matched template id {}", HEX.encode_display(id.get())),
                Err(()) => println!("No match"),
            }
        }
        Command::Delete { id } => {
            let id = id.map(|x| HEX.decode(x.as_bytes())).transpose()?;
            call!(Delete(&device): (id) -> ()).get();
            println!("Deleted");
        }
        Command::List => {
            let ids = call!(List(&device): () -> (_));
            println!("There are {} template ids:", ids.get().len());
            for id in ids.get() {
                println!("- {}", HEX.encode_display(id));
            }
        }
        Command::Detect => {
            call!(DetectStart(&device): () -> ()).get();
            let mut stdout = std::io::stdout().lock();
            write!(stdout, "Waiting for touch").unwrap();
            while !stop.load(Relaxed) {
                if *call!(DetectConsume(&device): () -> (_)).get() {
                    writeln!(stdout, "touched").unwrap();
                    continue;
                }
                write!(stdout, ".").unwrap();
                stdout.flush().unwrap();
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            call!(DetectStop(&device): () -> ()).get();
        }
    }
    Ok(())
}

async fn call<T: Wire<'static>>(
    device: &DynDevice, name: &str, request: &Request,
    extract: impl FnOnce(Response) -> Result<T, Response>,
) -> Result<Yoke<T>> {
    let response = call_raw(device, request).await?;
    let response = response.try_map(|x| wasefire_wire::decode::<Result<Response, Error>>(x)?)?;
    match response.try_map(extract) {
        Ok(x) => Ok(x),
        Err(x) => bail!("unexpected response {x:?} for {name}"),
    }
}

async fn call_raw(device: &DynDevice, request: &Request) -> Result<Yoke<&'static [u8]>> {
    let request = wasefire_wire::encode(request)?.into_vec();
    let request = applet::Request { applet_id: AppletId, request: Cow::Owned(request) };
    device.call::<service::AppletRequest>(request).await?.get();
    loop {
        let response = device.call::<service::AppletResponse>(AppletId).await?;
        if let Ok(response) = response.try_map(|x| x.ok_or(())) {
            break Ok(response.map(|x| match x {
                Cow::Borrowed(x) => x,
                Cow::Owned(_) => unreachable!(),
            }));
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}
