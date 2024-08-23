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

use anyhow::{ensure, Context, Result};
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
    skip_counter: Option<u32>,
}

impl Changelog {
    fn read_file(changelog_path: &str) -> Result<Changelog> {
        let changelog_contents = wasefire_cli_tools::fs::read(&changelog_path)?;

        Self::parse(String::from_utf8(changelog_contents)?)
    }

    /// Converts raw file contents into a Changelog data structure.
    fn parse(contents: String) -> Result<Changelog> {
        let mut skip_counter = None;
        let mut releases = Vec::new();

        // Trim off the trailing comment: <!-- Increment to skip ... -->
        let contents: Vec<&str> =
            contents.split("\n\n<!-- Increment to skip CHANGELOG.md test:").collect();

        // Populate skip_counter if we have one.
        if contents.len() > 1 {
            let ending = contents[1]
                .trim()
                .strip_suffix("-->")
                .expect("Malformed 'Increment to skip CHANGELOG.md test'")
                .trim();

            skip_counter = ending.parse::<u32>().ok();
        }

        let contents = contents[0];

        // Split by release.
        // Skip(1) to skip the prefix.
        for each_release in contents.split("\n## ").skip(1) {
            let mut version_iter = each_release.split("\n### ").into_iter();
            let release_string = version_iter.next().expect("No release version detected.").trim();
            let mut release = Release::from(release_string)?;

            // Each 'Major', 'Minor', 'Patch' section within this release.
            for each_version in version_iter {
                let version_parts: Vec<_> = each_version.split_terminator("\n\n").collect();

                ensure!(version_parts.len() == 2, "Failed to parse release: {}", each_version);

                let version_type = ReleaseType::from_str(version_parts[0], true).expect(
                    format!(
                        "Failed to parse version_type: {}. Must be 'Major', 'Minor', or 'Patch'.",
                        version_parts[0]
                    )
                    .as_str(),
                );
                let version_contents = version_parts[1]
                    .split("- ")
                    .map(|str| str.trim().to_string())
                    .filter(|str| !str.is_empty())
                    .collect();

                match version_type {
                    ReleaseType::Major => release.major = version_contents,
                    ReleaseType::Minor => release.minor = version_contents,
                    ReleaseType::Patch => release.patch = version_contents,
                }
            }

            releases.push(release);
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

### Major

- major update 1
- major update 2

### Minor

- minor update 1
- minor update 2

### Patch

- patch update 1
- patch update 2
";

        assert_eq!(
            Changelog::parse(changelog.to_string()).expect("Failed to parse changelog."),
            Changelog {
                releases: vec![
                    Release {
                        version: Version::parse("0.2.0").unwrap(),
                        major: vec!["major update 1".to_string(), "major update 2".to_string()],
                        minor: vec!["minor update 1".to_string(), "minor update 2".to_string()],
                        patch: vec!["patch update 1".to_string(), "patch update 2".to_string()],
                    },
                    Release {
                        version: Version::parse("0.1.0").unwrap(),
                        major: vec!["major update 1".to_string(), "major update 2".to_string()],
                        minor: vec!["minor update 1".to_string(), "minor update 2".to_string()],
                        patch: vec!["patch update 1".to_string(), "patch update 2".to_string()],
                    }
                ],
                skip_counter: None,
            }
        );
    }

    #[test]
    fn parse_changelog_with_missing_release_type_success() {
        let changelog = r"# Changelog

## 0.2.0

### Major

- major update 1
- major update 2

## 0.1.0

### Minor

- minor update 1
- minor update 2

## 0.0.1

### Patch

- patch update 1
- patch update 2
";

        assert_eq!(
            Changelog::parse(changelog.to_string()).expect("Failed to parse changelog."),
            Changelog {
                releases: vec![
                    Release {
                        version: Version::parse("0.2.0").unwrap(),
                        major: vec!["major update 1".to_string(), "major update 2".to_string()],
                        minor: Vec::new(),
                        patch: Vec::new(),
                    },
                    Release {
                        version: Version::parse("0.1.0").unwrap(),
                        major: Vec::new(),
                        minor: vec!["minor update 1".to_string(), "minor update 2".to_string()],
                        patch: Vec::new(),
                    },
                    Release {
                        version: Version::parse("0.0.1").unwrap(),
                        major: Vec::new(),
                        minor: Vec::new(),
                        patch: vec!["patch update 1".to_string(), "patch update 2".to_string()],
                    }
                ],
                skip_counter: None,
            }
        );
    }

    #[test]
    fn parse_changelog_handles_multi_line_description() {
        let changelog = r"# Changelog

## 0.1.0

### Major

- short 1
- my long description
  that spans many lines.
- short 2

";

        assert_eq!(
            Changelog::parse(changelog.to_string()).expect("Failed to parse changelog."),
            Changelog {
                releases: vec![Release {
                    version: Version::parse("0.1.0").unwrap(),
                    major: vec![
                        "short 1".to_string(),
                        "my long description\n  that spans many lines.".to_string(),
                        "short 2".to_string()
                    ],
                    minor: Vec::new(),
                    patch: Vec::new(),
                }],
                skip_counter: None,
            }
        );
    }

    #[test]
    fn parse_changelog_removes_prefix() {
        let changelog = r"# Changelog

## 0.1.0
";

        assert_eq!(
            Changelog::parse(changelog.to_string()).expect("Failed to parse changelog."),
            Changelog {
                releases: vec![Release {
                    version: Version::parse("0.1.0").unwrap(),
                    major: Vec::new(),
                    minor: Vec::new(),
                    patch: Vec::new(),
                }],
                skip_counter: None,
            }
        );
    }

    #[test]
    fn parse_changelog_handles_increment_comment_at_end() {
        let changelog = r"# Changelog
## 0.1.0

### Major

- A change

<!-- Increment to skip CHANGELOG.md test: 5 -->";

        assert_eq!(
            Changelog::parse(changelog.to_string()).expect("Failed to parse changelog."),
            Changelog {
                releases: vec![Release {
                    version: Version::parse("0.1.0").unwrap(),
                    major: vec!["A change".to_string()],
                    minor: Vec::new(),
                    patch: Vec::new(),
                }],
                skip_counter: Some(5),
            }
        );
    }
}
