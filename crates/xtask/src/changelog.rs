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

use anyhow::{anyhow, ensure, Context, Result};
use clap::ValueEnum;
use semver::Version;

#[derive(Debug, Clone, clap::ValueEnum)]
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
    fn read_file(changelog_path: &str) -> Result<Changelog> {
        let changelog_contents = wasefire_cli_tools::fs::read(&changelog_path)?;

        Self::parse(String::from_utf8(changelog_contents)?)
    }

    /// Converts raw file contents into a Changelog data structure.
    ///
    /// Validates requirements in docs/contributing/changelog.md
    fn parse(contents: String) -> Result<Changelog> {
        let mut skip_counter = 0;
        let mut releases: Vec<Release> = Vec::new();

        let mut lines = contents.lines().filter(|line| !line.is_empty()).peekable();

        // First line is H1.
        if lines.next() != Some("# Changelog") {
            return Err(anyhow!("H1 with 'Changelog' is required"));
        }

        while let Some(mut current_line) = lines.next() {
            let mut next_line = lines.peek();

            // Parse a whole release
            if let Some(version_string) = current_line.strip_prefix("## ") {
                let mut release = Release::from(version_string)?;

                // Parse each release type (major, minor, patch)
                while next_line.is_some_and(|line| line.starts_with("### ")) {
                    // Advance
                    current_line = lines.next().unwrap();
                    next_line = lines.peek();

                    let release_type_string = current_line.trim_start_matches("### ");
                    let release_type = ReleaseType::from_str(release_type_string, true)
                        .map_err(|err| anyhow!("Failed to parse version_type: {err}"))?;

                    let mut release_descriptions = Vec::new();

                    // Parse the release descriptions
                    // Some lines may be a new description. Some lines may be continuation of
                    // previous descriptions.
                    while next_line
                        .is_some_and(|line| line.starts_with("- ") || line.starts_with("  "))
                    {
                        // Advance
                        current_line = lines.next().unwrap();
                        next_line = lines.peek();

                        // New description
                        if current_line.starts_with("- ") {
                            release_descriptions
                                .push(current_line.trim_start_matches("- ").to_string());
                        } else if current_line.starts_with("  ") {
                            // Append to previous description
                            let mut last = release_descriptions
                                .pop()
                                .expect("No last description to append to.");
                            last.push_str(format!("\n{current_line}").as_str());
                            release_descriptions.push(last);
                        }
                    }

                    match release_type {
                        ReleaseType::Major => release.major = release_descriptions,
                        ReleaseType::Minor => release.minor = release_descriptions,
                        ReleaseType::Patch => release.patch = release_descriptions,
                    }
                }

                // Validate descending versions.
                if let Some(previous_version) = releases.last() {
                    if release.version > previous_version.version {
                        return Err(anyhow!("Versions out of order"));
                    }
                }

                // Validate pre-release.
                if !releases.is_empty() && !release.version.pre.is_empty() {
                    return Err(anyhow!("Only the first version can be pre-release"));
                }

                releases.push(release);
            }

            // Hit last line. Done parsing releases. Look for skip counter.
            if next_line.is_none() {
                skip_counter = current_line
                    .trim_start_matches("<!-- Increment to skip CHANGELOG.md test: ")
                    .trim_end_matches(" -->")
                    .parse::<u32>()
                    .map_err(|err| {
                        anyhow!("Invalid skip_counter at the end of the changelog ({err})")
                    })?;
            }
        }

        if let Some(last_release) = releases.last() {
            if last_release.version.to_string() != "0.1.0" {
                return Err(anyhow!("The last release must be version 0.1.0"));
            }

            if last_release.major.len() > 0
                || last_release.minor.len() > 0
                || last_release.patch.len() > 0
            {
                return Err(anyhow!("The last release must contain no changes"));
            }
        } else {
            return Err(anyhow!("At least 1 release is required"));
        }

        Ok(Changelog { releases, skip_counter })
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Release {
    version: Version,

    major: Vec<String>,
    minor: Vec<String>,
    patch: Vec<String>,
}

impl Release {
    fn from(version: &str) -> Result<Self> {
        let semver = Version::parse(version)?;

        Ok(Release { version: semver, major: Vec::new(), minor: Vec::new(), patch: Vec::new() })
    }
}

/// Validates CHANGELOG.md files for all Wasefire crates.
pub fn execute_ci() -> Result<()> {
    let dir_entries = wasefire_cli_tools::fs::read_dir("crates")?;

    let existing_changelog_paths = dir_entries.filter_map(|entry| {
        if let Ok(crate_path) = entry {
            if crate_path.metadata().is_ok_and(|metadata| metadata.is_dir()) {
                let changelog_file_path = format!("{}/CHANGELOG.md", crate_path.path().display());

                if wasefire_cli_tools::fs::exists(&changelog_file_path) {
                    return Some(changelog_file_path);
                }
            }
        }

        None
    });

    for changelog_path in existing_changelog_paths {
        // Validation done during parsing.
        Changelog::read_file(&changelog_path)
            .with_context(|| format!("validating changelog file: {changelog_path}"))?;
    }

    Ok(())
}

/// Updates a CHANGELOG.md file and CHANGELOG.md files of dependencies.
pub fn execute_change(crate_path: &str, _version: &ReleaseType, _message: &str) -> Result<()> {
    ensure!(wasefire_cli_tools::fs::exists(crate_path), "Crate does not exist: {}", crate_path);

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
            Changelog::parse(changelog.to_string()).expect("Failed to parse changelog."),
            Changelog {
                releases: vec![
                    Release {
                        version: Version::parse("0.3.0").unwrap(),
                        major: vec!["major update 1".to_string(), "major update 2".to_string()],
                        minor: vec!["minor update 1".to_string(), "minor update 2".to_string()],
                        patch: vec!["patch update 1".to_string(), "patch update 2".to_string()],
                    },
                    Release {
                        version: Version::parse("0.2.0").unwrap(),
                        major: vec!["major update 1".to_string(), "major update 2".to_string()],
                        minor: vec!["minor update 1".to_string(), "minor update 2".to_string()],
                        patch: vec!["patch update 1".to_string(), "patch update 2".to_string()],
                    },
                    Release {
                        version: Version::parse("0.1.0").unwrap(),
                        major: Vec::new(),
                        minor: Vec::new(),
                        patch: Vec::new(),
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
            Changelog::parse(changelog.to_string()).expect("Failed to parse changelog."),
            Changelog {
                releases: vec![
                    Release {
                        version: Version::parse("0.4.0").unwrap(),
                        major: vec!["major update 1".to_string(), "major update 2".to_string()],
                        minor: Vec::new(),
                        patch: Vec::new(),
                    },
                    Release {
                        version: Version::parse("0.3.0").unwrap(),
                        major: Vec::new(),
                        minor: vec!["minor update 1".to_string(), "minor update 2".to_string()],
                        patch: Vec::new(),
                    },
                    Release {
                        version: Version::parse("0.2.0").unwrap(),
                        major: Vec::new(),
                        minor: Vec::new(),
                        patch: vec!["patch update 1".to_string(), "patch update 2".to_string()],
                    },
                    Release {
                        version: Version::parse("0.1.0").unwrap(),
                        major: Vec::new(),
                        minor: Vec::new(),
                        patch: Vec::new(),
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
  that spans many lines.
- short 2

## 0.1.0

<!-- Increment to skip CHANGELOG.md test: 0 -->";

        assert_eq!(
            Changelog::parse(changelog.to_string()).expect("Failed to parse changelog."),
            Changelog {
                releases: vec![
                    Release {
                        version: Version::parse("0.2.0").unwrap(),
                        major: vec![
                            "short 1".to_string(),
                            "my long description\n  that spans many lines.".to_string(),
                            "short 2".to_string()
                        ],
                        minor: Vec::new(),
                        patch: Vec::new(),
                    },
                    Release {
                        version: Version::parse("0.1.0").unwrap(),
                        major: Vec::new(),
                        minor: Vec::new(),
                        patch: Vec::new(),
                    }
                ],
                skip_counter: 0,
            }
        );
    }

    #[test]
    fn parse_changelog_removes_prefix() {
        let changelog = r"# Changelog

## 0.1.0

<!-- Increment to skip CHANGELOG.md test: 0 -->";

        assert_eq!(
            Changelog::parse(changelog.to_string()).expect("Failed to parse changelog."),
            Changelog {
                releases: vec![Release {
                    version: Version::parse("0.1.0").unwrap(),
                    major: Vec::new(),
                    minor: Vec::new(),
                    patch: Vec::new(),
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
            Changelog::parse(changelog.to_string()).expect("Failed to parse changelog."),
            Changelog {
                releases: vec![Release {
                    version: Version::parse("0.1.0").unwrap(),
                    major: Vec::new(),
                    minor: Vec::new(),
                    patch: Vec::new(),
                }],
                skip_counter: 5,
            }
        );
    }

    #[test]
    fn parse_changelog_requires_skip_counter_at_end() {
        let changelog = r"# Changelog
## 0.1.0";

        assert!(Changelog::parse(changelog.to_string())
            .expect_err("Parse changelog was successful.")
            .to_string()
            .contains("Invalid skip_counter at the end of the changelog"))
    }

    #[test]
    fn parse_changelog_must_start_with_h1_changelog() {
        let changelog = r"
## 0.1.0";

        assert!(Changelog::parse(changelog.to_string())
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

        assert!(Changelog::parse(changelog.to_string())
            .expect_err("Parse changelog was successful.")
            .to_string()
            .contains("Versions out of order"))
    }

    #[test]
    fn parse_changelog_versions_requires_at_least_one_release() {
        let changelog = r"# Changelog

<!-- Increment to skip CHANGELOG.md test: 0 -->";

        assert!(Changelog::parse(changelog.to_string())
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

        assert!(Changelog::parse(changelog.to_string())
            .expect_err("Parse changelog was successful.")
            .to_string()
            .contains("The last release must be version 0.1.0"))
    }

    #[test]
    fn parse_changelog_versions_last_version_must_be_empty() {
        let changelog = r"# Changelog

## 0.1.0

### Major

- A change

<!-- Increment to skip CHANGELOG.md test: 0 -->";

        assert!(Changelog::parse(changelog.to_string())
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

        assert!(Changelog::parse(changelog.to_string())
            .expect_err("Parse changelog was successful.")
            .to_string()
            .contains("Only the first version can be pre-release"))
    }
}
