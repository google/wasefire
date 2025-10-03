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

//! Helpers for changelog files.
//!
//! See <https://github.com/google/wasefire/blob/main/docs/contributing/changelog.md> for a
//! description of the changelog format.

use std::collections::BTreeMap;
use std::fmt::Display;
use std::io::BufRead;

use anyhow::{Context, Result, anyhow, bail, ensure};
use clap::ValueEnum;
use semver::Version;
use tokio::process::Command;

use crate::cargo::metadata;
use crate::{cmd, fs};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum)]
pub enum Severity {
    Major,
    Minor,
    Patch,
}

impl Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Major => write!(f, "Major"),
            Self::Minor => write!(f, "Minor"),
            Self::Patch => write!(f, "Patch"),
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
struct Changelog {
    releases: Vec<Release>,
    skip_counter: u32,
}

impl Changelog {
    async fn read_file(path: &str) -> Result<Changelog> {
        Self::parse(&String::from_utf8(fs::read(path).await?)?)
    }

    /// Parses and validates a changelog.
    fn parse(input: &str) -> Result<Changelog> {
        let mut releases: Vec<Release> = Vec::new();
        let mut parser = Parser::new(input.lines());
        parser.read_exact("# Changelog")?;
        parser.read_empty()?;
        parser.advance()?;
        loop {
            let version = (parser.buffer.strip_prefix("## "))
                .with_context(|| anyhow!("Expected release {parser}"))?;
            let version =
                Version::parse(version).with_context(|| anyhow!("Parsing version {parser}"))?;
            ensure!(version.build.is_empty(), "Unexpected build metadata {parser}");
            match releases.last() {
                Some(prev) => {
                    ensure!(version.pre.is_empty(), "Unexpected prerelease {parser}");
                    let severity = *prev.contents.first_key_value().unwrap().0;
                    ensure_conform(&version, severity, &prev.version)?;
                }
                None => {
                    let pre = version.pre.as_str();
                    ensure!(matches!(pre, "" | "git"), "Invalid prerelease {pre:?} {parser}");
                }
            }
            parser.read_empty()?;
            parser.advance()?;
            let mut contents = BTreeMap::new();
            if matches!(version, Version { major: 0, minor: 1, patch: 0, .. }) {
                releases.push(Release { version, contents });
                break;
            }
            while let Some(severity) = parser.buffer.strip_prefix("### ") {
                let severity = match severity {
                    "Major" => Severity::Major,
                    "Minor" => Severity::Minor,
                    "Patch" => Severity::Patch,
                    _ => bail!("Invalid severity {severity:?} {parser}"),
                };
                if let Some(&prev) = contents.last_key_value().map(|(x, _)| x) {
                    ensure!(prev < severity, "Out of order severity {parser}");
                }
                parser.read_empty()?;
                let mut descriptions = Vec::new();
                while parser.read_empty().is_err() {
                    if parser.buffer.starts_with("- ") {
                        descriptions.push(parser.buffer.to_string());
                    } else if parser.buffer.starts_with("  ") {
                        let description = descriptions
                            .last_mut()
                            .with_context(|| anyhow!("Invalid continuation {parser}"))?;
                        description.push('\n');
                        description.push_str(parser.buffer);
                    } else {
                        bail!("Invalid description {parser}");
                    }
                    ensure!(
                        !descriptions.last_mut().unwrap().ends_with("."),
                        "Description ends with dot {parser}"
                    );
                }
                assert!(contents.insert(severity, descriptions).is_none());
                parser.advance()?;
            }
            ensure!(!contents.is_empty(), "Release {version} is empty");
            releases.push(Release { version, contents });
        }
        ensure!(!releases.is_empty(), "Changelog has no releases");
        let skip_counter = parser
            .buffer
            .strip_prefix("<!-- Increment to skip CHANGELOG.md test: ")
            .with_context(|| anyhow!("Invalid skip counter prefix {parser}"))?
            .strip_suffix(" -->")
            .with_context(|| anyhow!("Invalid skip counter suffix {parser}"))?
            .parse()
            .with_context(|| anyhow!("Invalid skip counter {parser}"))?;
        parser.done()?;
        ensure!(
            skip_counter == 0 || releases.first().unwrap().version.pre.as_str() == "git",
            "Non-zero skip counter without prerelease"
        );
        let result = Changelog { releases, skip_counter };
        assert_eq!(format!("{result}"), input);
        Ok(result)
    }

    async fn validate_cargo_toml(&self, path: &str) -> Result<()> {
        let metadata = metadata(path).await?;
        ensure!(
            self.releases.first().unwrap().version == metadata.packages[0].version,
            "Version mismatch between Cargo.toml and CHANGELOG.md for {path}"
        );
        Ok(())
    }
}

impl Display for Changelog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Changelog { releases, skip_counter } = self;
        writeln!(f, "# Changelog\n")?;
        for release in releases {
            write!(f, "{release}")?;
        }
        writeln!(f, "<!-- Increment to skip CHANGELOG.md test: {skip_counter} -->")
    }
}

struct Parser<'a, I: Iterator<Item = &'a str>> {
    count: usize,
    buffer: &'a str,
    lines: I,
}

impl<'a, I: Iterator<Item = &'a str>> Display for Parser<'a, I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let line = if f.alternate() { "Line" } else { "line" };
        let count = self.count;
        write!(f, "{line} {count}")
    }
}

impl<'a, I: Iterator<Item = &'a str>> Parser<'a, I> {
    fn new(lines: I) -> Self {
        Parser { count: 0, buffer: "", lines }
    }

    fn done(mut self) -> Result<()> {
        Ok(ensure!(self.lines.next().is_none(), "Expected end of file after {self}"))
    }

    fn advance(&mut self) -> Result<&'a str> {
        self.buffer =
            self.lines.next().with_context(|| anyhow!("Unexpected end of file after {self}"))?;
        self.count += 1;
        Ok(self.buffer)
    }

    fn read_exact(&mut self, line: &str) -> Result<()> {
        self.advance()?;
        ensure!(self.buffer == line, "{self:#} should be {line:?}");
        Ok(())
    }

    fn read_empty(&mut self) -> Result<()> {
        self.advance()?;
        ensure!(self.buffer.is_empty(), "{self:#} should be empty");
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Release {
    version: Version,
    contents: BTreeMap<Severity, Vec<String>>,
}

impl Display for Release {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "## {}\n", self.version)?;
        for (severity, descriptions) in &self.contents {
            writeln!(f, "### {severity}\n")?;
            for description in descriptions {
                writeln!(f, "{description}")?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

fn ensure_conform(old: &Version, severity: Severity, new: &Version) -> Result<()> {
    let effective = match (new.major, severity) {
        (0, Severity::Major) => Severity::Minor,
        (0, _) => Severity::Patch,
        (_, x) => x,
    };
    fn aux(threshold: Severity, actual: Severity, old: u64) -> u64 {
        match actual.cmp(&threshold) {
            std::cmp::Ordering::Less => 0,
            std::cmp::Ordering::Equal => old + 1,
            std::cmp::Ordering::Greater => old,
        }
    }
    let mut expected = new.clone();
    expected.major = aux(Severity::Major, effective, old.major);
    expected.minor = aux(Severity::Minor, effective, old.minor);
    expected.patch = aux(Severity::Patch, effective, old.patch);
    ensure!(
        *new == expected,
        "Release {new} should be {expected} due to {} bump from {old}",
        severity.to_possible_value().unwrap().get_name()
    );
    Ok(())
}

/// Validates changelog files for all crates.
pub async fn execute_ci() -> Result<()> {
    let paths = cmd::output(Command::new("git").args(["ls-files", "*/CHANGELOG.md"])).await?;
    for path in paths.stdout.lines() {
        let path = path?;
        let changelog = Changelog::read_file(&path).await?;
        changelog.validate_cargo_toml(path.strip_suffix("/CHANGELOG.md").unwrap()).await?;
    }
    Ok(())
}

/// Updates a changelog file and changelog files of dependencies.
pub async fn execute_change(path: &str, _severity: &Severity, _description: &str) -> Result<()> {
    ensure!(fs::exists(path).await, "Crate does not exist: {path}");

    let _changelog = Changelog::read_file(&format!("{path}/CHANGELOG.md")).await?;

    todo!("Implement changelog updates");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_changelog_success() {
        let changelog = r"# Changelog

## 0.3.0

### Major

- major update 1
- major update 2

### Minor

- minor update 1
- minor update 2

### Patch

- patch update 1
- patch update 2

## 0.2.0

### Major

- major update 1
- major update 2

### Minor

- minor update 1
- minor update 2

### Patch

- patch update 1
- patch update 2

## 0.1.0

<!-- Increment to skip CHANGELOG.md test: 0 -->
";

        assert_eq!(
            Changelog::parse(changelog).unwrap(),
            Changelog {
                releases: vec![
                    Release {
                        version: Version::parse("0.3.0").unwrap(),
                        contents: BTreeMap::from([
                            (
                                Severity::Major,
                                vec![
                                    "- major update 1".to_string(),
                                    "- major update 2".to_string()
                                ]
                            ),
                            (
                                Severity::Minor,
                                vec![
                                    "- minor update 1".to_string(),
                                    "- minor update 2".to_string()
                                ]
                            ),
                            (
                                Severity::Patch,
                                vec![
                                    "- patch update 1".to_string(),
                                    "- patch update 2".to_string()
                                ]
                            )
                        ]),
                    },
                    Release {
                        version: Version::parse("0.2.0").unwrap(),
                        contents: BTreeMap::from([
                            (
                                Severity::Major,
                                vec![
                                    "- major update 1".to_string(),
                                    "- major update 2".to_string()
                                ]
                            ),
                            (
                                Severity::Minor,
                                vec![
                                    "- minor update 1".to_string(),
                                    "- minor update 2".to_string()
                                ]
                            ),
                            (
                                Severity::Patch,
                                vec![
                                    "- patch update 1".to_string(),
                                    "- patch update 2".to_string()
                                ]
                            )
                        ]),
                    },
                    Release {
                        version: Version::parse("0.1.0").unwrap(),
                        contents: BTreeMap::new()
                    }
                ],
                skip_counter: 0,
            }
        );
    }

    #[test]
    fn write_changelog_success() {
        let changelog = r"# Changelog

## 0.3.0

### Major

- major update 1
- major update 2

### Minor

- minor update 1
- minor update 2

### Patch

- patch update 1
- patch update 2

## 0.2.0

### Major

- major update 1
- major update 2

### Minor

- minor update 1
- minor update 2

### Patch

- patch update 1
- patch update 2

## 0.1.0

<!-- Increment to skip CHANGELOG.md test: 0 -->
";

        assert_eq!(format!("{}", Changelog::parse(changelog).unwrap()), changelog);
    }

    #[test]
    fn parse_changelog_with_missing_severity_success() {
        let changelog = r"# Changelog

## 0.2.0

### Major

- major update 1
- major update 2

## 0.1.2

### Minor

- minor update 1
- minor update 2

## 0.1.1

### Patch

- patch update 1
- patch update 2

## 0.1.0

<!-- Increment to skip CHANGELOG.md test: 0 -->
";

        assert_eq!(
            Changelog::parse(changelog).unwrap(),
            Changelog {
                releases: vec![
                    Release {
                        version: Version::parse("0.2.0").unwrap(),
                        contents: BTreeMap::from([(
                            Severity::Major,
                            vec!["- major update 1".to_string(), "- major update 2".to_string()]
                        )]),
                    },
                    Release {
                        version: Version::parse("0.1.2").unwrap(),
                        contents: BTreeMap::from([(
                            Severity::Minor,
                            vec!["- minor update 1".to_string(), "- minor update 2".to_string()]
                        )]),
                    },
                    Release {
                        version: Version::parse("0.1.1").unwrap(),
                        contents: BTreeMap::from([(
                            Severity::Patch,
                            vec!["- patch update 1".to_string(), "- patch update 2".to_string()]
                        )]),
                    },
                    Release {
                        version: Version::parse("0.1.0").unwrap(),
                        contents: BTreeMap::new()
                    }
                ],
                skip_counter: 0,
            }
        );
    }

    #[test]
    fn parse_changelog_handles_multi_line_description() {
        let changelog = r"# Changelog

## 0.2.0

### Major

- short 1
- my long description
  that spans many lines
- short 2

## 0.1.0

<!-- Increment to skip CHANGELOG.md test: 0 -->
";

        assert_eq!(
            Changelog::parse(changelog).unwrap(),
            Changelog {
                releases: vec![
                    Release {
                        version: Version::parse("0.2.0").unwrap(),
                        contents: BTreeMap::from([(
                            Severity::Major,
                            vec![
                                "- short 1".to_string(),
                                "- my long description\n  that spans many lines".to_string(),
                                "- short 2".to_string()
                            ]
                        )]),
                    },
                    Release {
                        version: Version::parse("0.1.0").unwrap(),
                        contents: BTreeMap::new()
                    }
                ],
                skip_counter: 0,
            }
        );
    }

    #[test]
    fn parse_changelog_handles_description_must_not_end_with_period() {
        let changelog = r"# Changelog

## 0.2.0

### Major

- short 1.

## 0.1.0

<!-- Increment to skip CHANGELOG.md test: 0 -->
";

        assert_eq!(
            Changelog::parse(changelog).unwrap_err().to_string(),
            "Description ends with dot line 7"
        );
    }

    #[test]
    fn parse_changelog_removes_prefix() {
        let changelog = r"# Changelog

## 0.1.0

<!-- Increment to skip CHANGELOG.md test: 0 -->
";

        assert_eq!(
            Changelog::parse(changelog).unwrap(),
            Changelog {
                releases: vec![Release {
                    version: Version::parse("0.1.0").unwrap(),
                    contents: BTreeMap::new(),
                }],
                skip_counter: 0,
            }
        );
    }

    #[test]
    fn parse_changelog_handles_skip_counter_at_end() {
        let changelog = r"# Changelog

## 0.1.1-git

### Patch

- Update dependencies

## 0.1.0

<!-- Increment to skip CHANGELOG.md test: 5 -->
";

        assert_eq!(
            Changelog::parse(changelog).unwrap(),
            Changelog {
                releases: vec![
                    Release {
                        version: Version::parse("0.1.1-git").unwrap(),
                        contents: BTreeMap::from([(
                            Severity::Patch,
                            vec!["- Update dependencies".to_string(),]
                        )])
                    },
                    Release {
                        version: Version::parse("0.1.0").unwrap(),
                        contents: BTreeMap::new(),
                    }
                ],
                skip_counter: 5,
            }
        );
    }

    #[test]
    fn parse_changelog_requires_skip_counter_at_end() {
        let changelog = r"# Changelog

## 0.1.0

";

        assert_eq!(
            Changelog::parse(changelog).unwrap_err().to_string(),
            "Unexpected end of file after line 4"
        );
    }

    #[test]
    fn parse_changelog_must_start_with_h1_changelog() {
        let changelog = r"
## 0.1.0";

        assert_eq!(
            Changelog::parse(changelog).unwrap_err().to_string(),
            "Line 1 should be \"# Changelog\""
        );
    }

    #[test]
    fn parse_changelog_versions_must_be_in_order() {
        let changelog = r"# Changelog

## 0.1.1

### Major

- A change

## 0.2.0

### Major

- A change

<!-- Increment to skip CHANGELOG.md test: 0 -->
";

        assert_eq!(
            Changelog::parse(changelog).unwrap_err().to_string(),
            "Release 0.1.1 should be 0.3.0 due to major bump from 0.2.0"
        );
    }

    #[test]
    fn parse_changelog_versions_requires_at_least_one_release() {
        let changelog = r"# Changelog

<!-- Increment to skip CHANGELOG.md test: 0 -->
";

        assert_eq!(Changelog::parse(changelog).unwrap_err().to_string(), "Expected release line 3");
    }

    #[test]
    fn parse_changelog_versions_last_version_must_be_0_1_0() {
        let changelog = r"# Changelog

## 0.2.0

### Major

- A change

<!-- Increment to skip CHANGELOG.md test: 0 -->
";

        assert_eq!(Changelog::parse(changelog).unwrap_err().to_string(), "Expected release line 9");
    }

    #[test]
    fn parse_changelog_versions_last_version_must_be_empty() {
        let changelog = r"# Changelog

## 0.1.0

### Major

- A change

<!-- Increment to skip CHANGELOG.md test: 0 -->
";

        assert_eq!(
            Changelog::parse(changelog).unwrap_err().to_string(),
            "Invalid skip counter prefix line 5"
        );
    }

    #[test]
    fn parse_changelog_skip_counter_is_zero_unless_prerelease() {
        let changelog = r"# Changelog

## 0.1.0

<!-- Increment to skip CHANGELOG.md test: 1 -->
";

        assert_eq!(
            Changelog::parse(changelog).unwrap_err().to_string(),
            "Non-zero skip counter without prerelease"
        );
    }

    #[test]
    fn parse_changelog_versions_only_first_can_be_prerelease() {
        let changelog = r"# Changelog

## 0.6.0-git

### Major

- A change

## 0.5.0-git

### Major

- A change

<!-- Increment to skip CHANGELOG.md test: 0 -->
";

        assert_eq!(
            Changelog::parse(changelog).unwrap_err().to_string(),
            "Unexpected prerelease line 9"
        );
    }
}
