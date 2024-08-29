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

use core::str;
use std::collections::BTreeMap;
use std::fmt::Write;
use std::io::BufRead;
use std::process::Command;

use anyhow::{anyhow, bail, ensure, Context, Result};
use semver::Version;
use wasefire_cli_tools::{cmd, fs};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum)]
pub enum ReleaseType {
    Major,
    Minor,
    Patch,
}

#[derive(Debug, Default, PartialEq, Eq)]
struct Changelog {
    releases: Vec<Release>,

    /// Incremented to acknowledge the changelog is already up-to-date.
    ///
    /// There is a CI test to make sure the changelog is updated when the crate is modified. Some
    /// modifications are already documented in the changelog, so the test will have a false
    /// positive. This counter is used to artificially modify the changelog and explicitly
    /// acknowledge that it's already up-to-date.
    skip_counter: u32,
}

impl Changelog {
    fn read_file(path: &str) -> Result<Changelog> {
        Self::parse(String::from_utf8(fs::read(&path)?)?.as_str())
    }

    /// Converts raw file contents into a Changelog data structure.
    ///
    /// Validates requirements in docs/contributing/changelog.md.
    fn parse(contents: &str) -> Result<Changelog> {
        let mut skip_counter: u32 = 0;
        let mut releases: Vec<Release> = Vec::new();

        let mut lines = contents.lines().filter(|line| !line.is_empty()).peekable();

        // First line is H1.
        ensure!(lines.next() == Some("# Changelog"), "H1 with 'Changelog' is required");

        while let Some(mut current_line) = lines.next() {
            let mut next_line = lines.peek();

            // Hit last line. Done parsing releases. Look for skip counter.
            if next_line.is_none() {
                skip_counter = current_line
                    .strip_prefix("<!-- Increment to skip CHANGELOG.md test: ")
                    .ok_or_else(|| anyhow!("Malformed or missing skip counter: {current_line}"))?
                    .strip_suffix(" -->")
                    .ok_or_else(|| anyhow!("Malformed or missing skip counter: {current_line}"))?
                    .parse::<u32>()?;
                break;
            }

            let mut release = match current_line.strip_prefix("## ") {
                Some(version_string) => Release::new(version_string)?,
                None => bail!("Failed to find release starting with '## ': {current_line}"),
            };

            // Parse each release type (major, minor, patch)
            while next_line.is_some_and(|line| line.starts_with("### ")) {
                // Advance
                current_line = lines.next().unwrap();
                next_line = lines.peek();

                let release_type_string = match current_line.strip_prefix("### ") {
                    Some(l) => l,
                    _ => bail!("Failed to parse version header {current_line}"),
                };
                let release_type = match release_type_string {
                    "Major" => ReleaseType::Major,
                    "Minor" => ReleaseType::Minor,
                    "Patch" => ReleaseType::Patch,
                    _ => bail!("Failed to parse version header {current_line}"),
                };

                let mut release_descriptions = Vec::new();

                // Parse the release descriptions
                // Some lines may be a new description. Some lines may be continuation of
                // previous descriptions.
                while next_line.is_some_and(|line| line.starts_with("- ") || line.starts_with("  "))
                {
                    // Advance
                    current_line = lines.next().unwrap();
                    next_line = lines.peek();

                    ensure!(
                        !current_line.ends_with("."),
                        "Each description must not end in a '.' ({current_line})"
                    );

                    // New description
                    if let Some(l) = current_line.strip_prefix("- ") {
                        release_descriptions.push(l.to_string());
                    } else if current_line.starts_with("  ") {
                        // Append to previous description
                        write!(
                            release_descriptions.last_mut().context("Invalid continuation")?,
                            "\n{current_line}"
                        )?;
                    }
                }

                ensure!(
                    release.contents.insert(release_type, release_descriptions).is_none(),
                    "Duplicate release type detected for {release_type:?}"
                );
            }

            // Validate descending versions.
            if let Some(previous_version) = releases.last() {
                ensure!(
                    release.version < previous_version.version,
                    "Versions should be order by decreasing precedence"
                );
            }

            // Validate pre-release.
            ensure!(
                releases.is_empty() || release.version.pre.is_empty(),
                "Only the first version can be pre-release"
            );

            releases.push(release);
        }

        let last_release = releases.last().context("At least 1 release is required")?;

        ensure!(
            last_release.version.to_string() == "0.1.0",
            "The first release must be version 0.1.0"
        );
        ensure!(last_release.contents.is_empty(), "The last release must contain no changes");

        Ok(Changelog { releases, skip_counter })
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Release {
    version: Version,

    contents: BTreeMap<ReleaseType, Vec<String>>,
}

impl Release {
    fn new(version: &str) -> Result<Self> {
        Ok(Release { version: Version::parse(version)?, contents: BTreeMap::new() })
    }
}

/// Validates CHANGELOG.md files for all Wasefire crates.
pub fn execute_ci() -> Result<()> {
    let paths = cmd::output(Command::new("git").args(["ls-files", "*/CHANGELOG.md"]))?;

    for path in paths.stdout.lines() {
        let path = path?;

        // Validation done during parsing.
        Changelog::read_file(&path)
            .with_context(|| format!("validating changelog file: {path}"))?;
    }

    Ok(())
}

/// Updates a CHANGELOG.md file and CHANGELOG.md files of dependencies.
pub fn execute_change(
    crate_path: &str, _release_type: &ReleaseType, _description: &str,
) -> Result<()> {
    ensure!(fs::exists(crate_path), "Crate does not exist: {}", crate_path);

    let _changelog = Changelog::read_file(format!("{crate_path}/CHANGELOG.md").as_str())?;

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

<!-- Increment to skip CHANGELOG.md test: 0 -->";

        assert_eq!(
            Changelog::parse(changelog).expect("Failed to parse changelog."),
            Changelog {
                releases: vec![
                    Release {
                        version: Version::parse("0.3.0").unwrap(),
                        contents: BTreeMap::from([
                            (
                                ReleaseType::Major,
                                vec!["major update 1".to_string(), "major update 2".to_string()]
                            ),
                            (
                                ReleaseType::Minor,
                                vec!["minor update 1".to_string(), "minor update 2".to_string()]
                            ),
                            (
                                ReleaseType::Patch,
                                vec!["patch update 1".to_string(), "patch update 2".to_string()]
                            )
                        ]),
                    },
                    Release {
                        version: Version::parse("0.2.0").unwrap(),
                        contents: BTreeMap::from([
                            (
                                ReleaseType::Major,
                                vec!["major update 1".to_string(), "major update 2".to_string()]
                            ),
                            (
                                ReleaseType::Minor,
                                vec!["minor update 1".to_string(), "minor update 2".to_string()]
                            ),
                            (
                                ReleaseType::Patch,
                                vec!["patch update 1".to_string(), "patch update 2".to_string()]
                            )
                        ]),
                    },
                    Release {
                        version: Version::parse("0.1.0").unwrap(),
                        contents: BTreeMap::new(),
                    }
                ],
                skip_counter: 0,
            }
        );
    }

    #[test]
    fn parse_changelog_with_missing_release_type_success() {
        let changelog = r"# Changelog

## 0.4.0

### Major

- major update 1
- major update 2

## 0.3.0

### Minor

- minor update 1
- minor update 2

## 0.2.0

### Patch

- patch update 1
- patch update 2

## 0.1.0

<!-- Increment to skip CHANGELOG.md test: 0 -->";

        assert_eq!(
            Changelog::parse(changelog).expect("Failed to parse changelog."),
            Changelog {
                releases: vec![
                    Release {
                        version: Version::parse("0.4.0").unwrap(),
                        contents: BTreeMap::from([(
                            ReleaseType::Major,
                            vec!["major update 1".to_string(), "major update 2".to_string()]
                        )]),
                    },
                    Release {
                        version: Version::parse("0.3.0").unwrap(),
                        contents: BTreeMap::from([(
                            ReleaseType::Minor,
                            vec!["minor update 1".to_string(), "minor update 2".to_string()]
                        )]),
                    },
                    Release {
                        version: Version::parse("0.2.0").unwrap(),
                        contents: BTreeMap::from([(
                            ReleaseType::Patch,
                            vec!["patch update 1".to_string(), "patch update 2".to_string()]
                        )]),
                    },
                    Release {
                        version: Version::parse("0.1.0").unwrap(),
                        contents: BTreeMap::new(),
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

<!-- Increment to skip CHANGELOG.md test: 0 -->";

        assert_eq!(
            Changelog::parse(changelog).expect("Failed to parse changelog."),
            Changelog {
                releases: vec![
                    Release {
                        version: Version::parse("0.2.0").unwrap(),
                        contents: BTreeMap::from([(
                            ReleaseType::Major,
                            vec![
                                "short 1".to_string(),
                                "my long description\n  that spans many lines".to_string(),
                                "short 2".to_string()
                            ]
                        )]),
                    },
                    Release {
                        version: Version::parse("0.1.0").unwrap(),
                        contents: BTreeMap::new(),
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

<!-- Increment to skip CHANGELOG.md test: 0 -->";

        assert!(Changelog::parse(changelog)
            .expect_err("Parse changelog was successful.")
            .to_string()
            .contains("Each description must not end in a '.' (- short 1.)"))
    }

    #[test]
    fn parse_changelog_removes_prefix() {
        let changelog = r"# Changelog

## 0.1.0

<!-- Increment to skip CHANGELOG.md test: 0 -->";

        assert_eq!(
            Changelog::parse(changelog).expect("Failed to parse changelog."),
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
## 0.1.0

<!-- Increment to skip CHANGELOG.md test: 5 -->";

        assert_eq!(
            Changelog::parse(changelog).expect("Failed to parse changelog."),
            Changelog {
                releases: vec![Release {
                    version: Version::parse("0.1.0").unwrap(),
                    contents: BTreeMap::new(),
                }],
                skip_counter: 5,
            }
        );
    }

    #[test]
    fn parse_changelog_requires_skip_counter_at_end() {
        let changelog = r"# Changelog
## 0.1.0";

        assert!(Changelog::parse(changelog)
            .expect_err("Parse changelog was successful.")
            .to_string()
            .contains("Malformed or missing skip counter"))
    }

    #[test]
    fn parse_changelog_must_start_with_h1_changelog() {
        let changelog = r"
## 0.1.0";

        assert!(Changelog::parse(changelog)
            .expect_err("Parse changelog was successful.")
            .to_string()
            .contains("H1 with 'Changelog' is required"))
    }

    #[test]
    fn parse_changelog_versions_must_be_in_order() {
        let changelog = r"# Changelog

## 0.1.0

### Major

- A change

## 0.2.0

### Major

- A change

<!-- Increment to skip CHANGELOG.md test: 0 -->";

        assert!(Changelog::parse(changelog)
            .expect_err("Parse changelog was successful.")
            .to_string()
            .contains("Versions should be order by decreasing precedence"))
    }

    #[test]
    fn parse_changelog_versions_requires_at_least_one_release() {
        let changelog = r"# Changelog

<!-- Increment to skip CHANGELOG.md test: 0 -->";

        assert!(Changelog::parse(changelog)
            .expect_err("Parse changelog was successful.")
            .to_string()
            .contains("At least 1 release is required"))
    }

    #[test]
    fn parse_changelog_versions_last_version_must_be_0_1_0() {
        let changelog = r"# Changelog

## 0.2.0

### Major

- A change

<!-- Increment to skip CHANGELOG.md test: 0 -->";

        assert!(Changelog::parse(changelog)
            .expect_err("Parse changelog was successful.")
            .to_string()
            .contains("The first release must be version 0.1.0"))
    }

    #[test]
    fn parse_changelog_versions_last_version_must_be_empty() {
        let changelog = r"# Changelog

## 0.1.0

### Major

- A change

<!-- Increment to skip CHANGELOG.md test: 0 -->";

        assert!(Changelog::parse(changelog)
            .expect_err("Parse changelog was successful.")
            .to_string()
            .contains("The last release must contain no changes"))
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

<!-- Increment to skip CHANGELOG.md test: 0 -->";

        assert!(Changelog::parse(changelog)
            .expect_err("Parse changelog was successful.")
            .to_string()
            .contains("Only the first version can be pre-release"))
    }
}
