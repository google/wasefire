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

use std::io::BufRead;

use anyhow::{Context, Result, ensure};
use tokio::process::Command;
use wasefire_cli_tools::cmd;

pub async fn execute() -> Result<()> {
    let paths = cmd::output(Command::new("git").args(["ls-files", ":(attr:textreview)"])).await?;
    let mut errors = Vec::new();
    for path in paths.stdout.lines() {
        let path = path?;
        let content = std::fs::read(&path).with_context(|| format!("reading {path}"))?;
        verify_file(&path, content.as_slice(), &mut errors)?;
    }
    for Error { path, count, kind, line } in &errors {
        println!("{path}:{count}: {kind}\n{line}");
    }
    ensure!(errors.is_empty(), "Found {} errors.", errors.len());
    Ok(())
}

fn verify_file(path: &str, mut content: &[u8], errors: &mut Vec<Error>) -> Result<()> {
    let mut count = 0usize;
    loop {
        count = count.wrapping_add(1);
        let mut line = String::new();
        if content.read_line(&mut line)? == 0 {
            return Ok(()); // end of file
        }
        if let Err(kind) = verify_line(line.as_bytes()) {
            let path = path.to_string();
            errors.push(Error { path, count, kind, line });
        }
    }
}

/// Ensures that a line can be reviewed in printed form.
///
/// The following properties must hold:
/// - The line must end with a line-feed (0x0a).
/// - The line-feed may only be preceded by a graphic character (from 0x21 to 0x7f).
/// - The line may start with a sequence of tabs (0x09).
/// - Otherwise, the line must only contain space or graphic characters (from 0x20 to 0x7f).
///
/// # Panics
///
/// Panics if the sequence of bytes is empty (which indicates the end of the file).
fn verify_line(mut bytes: &[u8]) -> std::result::Result<(), Kind> {
    match bytes.split_last().unwrap() {
        (b'\n', x) => bytes = x, // remove newline
        (&x, _) => return Err(Kind::End(x)),
    }
    match bytes.last() {
        Some(b'!' ..= b'~') => (),
        Some(&x) => return Err(Kind::Trail(x)),
        None => return Ok(()), // empty line
    }
    while let Some((b'\t', x)) = bytes.split_first() {
        bytes = x; // skip leading tabs
    }
    for &byte in bytes {
        match byte {
            b' ' ..= b'~' => (),
            x => return Err(Kind::Inner(x)),
        }
    }
    Ok(())
}

struct Error {
    path: String,
    count: usize,
    kind: Kind,
    line: String,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Kind {
    End(u8),
    Trail(u8),
    Inner(u8),
}

impl std::fmt::Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Kind::End(x) => write!(f, "line ends with {x:#02x} instead of 0x0a"),
            Kind::Trail(x) => write!(f, "line trails with {x:#02x} outside of 0x21 to 0x7f"),
            Kind::Inner(b'\t') => write!(f, "line contains 0x09 outside indentation"),
            Kind::Inner(x) => write!(f, "line contains {x:#02x} outside of 0x20 to 0x7f"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_line_ok() {
        assert_eq!(verify_line(b"\n"), Ok(()));
        assert_eq!(verify_line(b"hello\n"), Ok(()));
        assert_eq!(verify_line(b"\thello\n"), Ok(()));
        assert_eq!(verify_line(b"\t\thello world\n"), Ok(()));
    }

    #[test]
    fn verify_line_err_end() {
        assert_eq!(verify_line(b"hello "), Err(Kind::End(b' ')));
        assert_eq!(verify_line(b"\t\t"), Err(Kind::End(b'\t')));
        assert_eq!(verify_line(b"\xff"), Err(Kind::End(0xff)));
    }

    #[test]
    fn verify_line_err_trail() {
        assert_eq!(verify_line(b"hello \n"), Err(Kind::Trail(b' ')));
        assert_eq!(verify_line(b"\t\t\n"), Err(Kind::Trail(b'\t')));
        assert_eq!(verify_line(b"\xff\n"), Err(Kind::Trail(0xff)));
    }

    #[test]
    fn verify_line_err_inner() {
        assert_eq!(verify_line(b"\xffworld\n"), Err(Kind::Inner(0xff)));
        assert_eq!(verify_line(b"\t\t\xffworld\n"), Err(Kind::Inner(0xff)));
        assert_eq!(verify_line(b"hello\xffworld\n"), Err(Kind::Inner(0xff)));
        assert_eq!(verify_line(b"hello\tworld\n"), Err(Kind::Inner(b'\t')));
    }

    #[test]
    fn verify_file_correct() {
        #[track_caller]
        fn test(content: &[u8], expected: &[(usize, Kind)]) {
            let mut actual = Vec::new();
            verify_file("", content, &mut actual).unwrap();
            let actual: Vec<_> = actual.into_iter().map(|x| (x.count, x.kind)).collect();
            assert_eq!(actual, expected);
        }
        test(b"", &[]);
        test(b"hello\nworld\n", &[]);
        test(b"hello \nworld", &[(1, Kind::Trail(b' ')), (2, Kind::End(b'd'))]);
        test(b"hel\xce\xbbo\nworld\toeu\n", &[(1, Kind::Inner(0xce)), (2, Kind::Inner(b'\t'))]);
    }
}
