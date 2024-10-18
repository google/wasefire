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
use std::fmt::{Display, Write};
use std::io::BufRead;

use anyhow::{anyhow, bail, ensure, Context, Result};
use cargo_metadata::MetadataCommand;
use semver::{Prerelease, Version};
use tokio::process::Command;

use crate::{cmd, fs};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum)]
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

    /// Incremented to acknowledge the changelog is already up-to-date.
    ///
    /// There is a CI test to make sure the changelog is updated when the crate is modified. Some
    /// modifications are already documented in the changelog, so the test will have a false
    /// positive. This counter is used to artificially modify the changelog and explicitly
    /// acknowledge that it's already up-to-date.
    skip_counter: u32,
}

impl Changelog {
    async fn read_file(path: &str) -> Result<Changelog> {
        Self::parse(String::from_utf8(fs::read(path).await?)?.as_str())
    }

    async fn write_file(&self, path: &str) -> Result<()> {
        fs::write(path, self.to_string().as_bytes()).await
    }

    fn push_description(&mut self, severity: &Severity, content: &str) -> Result<()> {
        let current_release = self.get_or_create_release_mut();
        current_release.push_content(*severity, content)
    }

    // Gets newest (first) release. Creates a new one if newest release is not prerelease.
    fn get_or_create_release_mut(&mut self) -> &mut Release {
        let current_release = self.releases.first().unwrap();

        // Current version is released, insert a new one.
        if current_release.version.pre.is_empty() {
            let mut next_version = Version::new(
                current_release.version.major,
                current_release.version.minor,
                current_release.version.patch + 1,
            );

            next_version.pre = Prerelease::new("git").unwrap();

            let new_release = Release::from(next_version);

            self.releases.insert(0, new_release);
        }

        self.releases.first_mut().unwrap()
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

            // Parse each severity (major, minor, patch)
            while next_line.is_some_and(|line| line.starts_with("### ")) {
                // Advance
                current_line = lines.next().unwrap();
                next_line = lines.peek();

                let severity = match current_line.strip_prefix("### ") {
                    Some("Major") => Severity::Major,
                    Some("Minor") => Severity::Minor,
                    Some("Patch") => Severity::Patch,
                    _ => bail!("Failed to parse serevity: {current_line}"),
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
                    release.contents.insert(severity, release_descriptions).is_none(),
                    "Duplicate severity detected for {severity:?}"
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

        let initial_release = releases.last().context("At least 1 release is required")?;

        ensure!(
            matches!(initial_release.version.to_string().as_str(), "0.1.0" | "0.1.0-git"),
            "The first release must be version 0.1.0 or 0.1.0-git"
        );
        ensure!(initial_release.contents.is_empty(), "The last release must contain no changes");

        let changelog = Changelog { releases, skip_counter };
        let output_string = format!("{changelog}");

        ensure!(
            output_string == contents,
            "Input string does not equal rendered changelog.\n\nIn: {contents}\n\nOut: \
             {output_string}"
        );

        Ok(changelog)
    }

    fn validate_cargo_toml(&self, path: &str) -> Result<()> {
        let most_recent_release = self.releases.first().unwrap();
        let metadata = MetadataCommand::new().current_dir(path).no_deps().exec()?;
        let cargo_toml_version =
            &metadata.root_package().context("Cargo.toml version not found")?.version;

        ensure!(
            most_recent_release.version.to_string() == cargo_toml_version.to_string(),
            "The most recent version must match Cargo.toml's version"
        );

        Ok(())
    }

    async fn sync_cargo_toml(&self, path: &str) -> Result<()> {
        let expected_version = &self.releases.first().unwrap().version;
        let cargo_toml_path = format!("{path}/Cargo.toml");
        let mut cargo_toml: toml::Table = fs::read_toml(&cargo_toml_path).await?;

        cargo_toml["package"]["version"] = expected_version.to_string().into();

        fs::write_toml(&cargo_toml_path, &cargo_toml).await?;

        Ok(())
    }

    async fn sync_dependencies(&self, _path: &str) -> Result<()> {
        // TODO

        Ok(())
    }
}

impl Display for Changelog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "# Changelog\n")?;

        for release in &self.releases {
            writeln!(f, "{release}\n")?;
        }

        writeln!(f, "<!-- Increment to skip CHANGELOG.md test: {} -->", self.skip_counter)
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Release {
    version: Version,

    contents: BTreeMap<Severity, Vec<String>>,
}

impl Release {
    fn new(version: &str) -> Result<Self> {
        Ok(Release { version: Version::parse(version)?, contents: BTreeMap::new() })
    }

    fn push_content(&mut self, severity: Severity, content: &str) -> Result<()> {
        let entry = self.contents.entry(severity).or_default();

        entry.push(content.to_string());

        Ok(())
    }
}

impl From<Version> for Release {
    fn from(value: Version) -> Self {
        Release { version: value, contents: BTreeMap::new() }
    }
}

impl Display for Release {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "## {}", self.version)?;

        for (severity, descriptions) in &self.contents {
            writeln!(f, "\n\n### {severity}\n")?;

            for (index, description) in descriptions.iter().enumerate() {
                write!(f, "- {description}")?;

                if index != descriptions.len() - 1 {
                    writeln!(f)?;
                }
            }
        }

        Ok(())
    }
}

/// Validates CHANGELOG.md files for all Wasefire crates.
pub async fn execute_ci() -> Result<()> {
    let paths = cmd::output(Command::new("git").args(["ls-files", "*/CHANGELOG.md"])).await?;

    for path in paths.stdout.lines() {
        let path = path?;

        // Validation done during parsing.
        let changelog = Changelog::read_file(&path).await?;

        let crate_dir = path.strip_suffix("/CHANGELOG.md").unwrap();

        changelog.validate_cargo_toml(crate_dir)?;
    }

    Ok(())
}

/// Updates a CHANGELOG.md file and CHANGELOG.md files of dependencies.
pub async fn execute_change(path: &str, severity: &Severity, description: &str) -> Result<()> {
    let changelog_file_path = format!("{path}/CHANGELOG.md");

    ensure!(fs::exists(&changelog_file_path).await, "Crate does not exist: {changelog_file_path}");

    let mut changelog = Changelog::read_file(&changelog_file_path).await?;

    changelog.push_description(severity, description)?;
    changelog.write_file(&changelog_file_path).await?;
    changelog.sync_cargo_toml(path).await?;
    changelog.sync_dependencies(path).await?;

    Ok(())
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
            Changelog::parse(changelog).expect("Failed to parse changelog."),
            Changelog {
                releases: vec![
                    Release {
                        version: Version::parse("0.3.0").unwrap(),
                        contents: BTreeMap::from([
                            (
                                Severity::Major,
                                vec!["major update 1".to_string(), "major update 2".to_string()]
                            ),
                            (
                                Severity::Minor,
                                vec!["minor update 1".to_string(), "minor update 2".to_string()]
                            ),
                            (
                                Severity::Patch,
                                vec!["patch update 1".to_string(), "patch update 2".to_string()]
                            )
                        ]),
                    },
                    Release {
                        version: Version::parse("0.2.0").unwrap(),
                        contents: BTreeMap::from([
                            (
                                Severity::Major,
                                vec!["major update 1".to_string(), "major update 2".to_string()]
                            ),
                            (
                                Severity::Minor,
                                vec!["minor update 1".to_string(), "minor update 2".to_string()]
                            ),
                            (
                                Severity::Patch,
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

        assert_eq!(
            format!("{}", Changelog::parse(changelog).expect("Failed to parse changelog.")),
            changelog
        );
    }

    #[test]
    fn parse_changelog_with_missing_severity_success() {
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

<!-- Increment to skip CHANGELOG.md test: 0 -->
";

        assert_eq!(
            Changelog::parse(changelog).expect("Failed to parse changelog."),
            Changelog {
                releases: vec![
                    Release {
                        version: Version::parse("0.4.0").unwrap(),
                        contents: BTreeMap::from([(
                            Severity::Major,
                            vec!["major update 1".to_string(), "major update 2".to_string()]
                        )]),
                    },
                    Release {
                        version: Version::parse("0.3.0").unwrap(),
                        contents: BTreeMap::from([(
                            Severity::Minor,
                            vec!["minor update 1".to_string(), "minor update 2".to_string()]
                        )]),
                    },
                    Release {
                        version: Version::parse("0.2.0").unwrap(),
                        contents: BTreeMap::from([(
                            Severity::Patch,
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

<!-- Increment to skip CHANGELOG.md test: 0 -->
";

        assert_eq!(
            Changelog::parse(changelog).expect("Failed to parse changelog."),
            Changelog {
                releases: vec![
                    Release {
                        version: Version::parse("0.2.0").unwrap(),
                        contents: BTreeMap::from([(
                            Severity::Major,
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

<!-- Increment to skip CHANGELOG.md test: 0 -->
";

        assert!(Changelog::parse(changelog)
            .expect_err("Parse changelog was successful.")
            .to_string()
            .contains("Each description must not end in a '.' (- short 1.)"))
    }

    #[test]
    fn parse_changelog_removes_prefix() {
        let changelog = r"# Changelog

## 0.1.0

<!-- Increment to skip CHANGELOG.md test: 0 -->
";

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

<!-- Increment to skip CHANGELOG.md test: 5 -->
";

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

<!-- Increment to skip CHANGELOG.md test: 0 -->
";

        assert!(Changelog::parse(changelog)
            .expect_err("Parse changelog was successful.")
            .to_string()
            .contains("Versions should be order by decreasing precedence"))
    }

    #[test]
    fn parse_changelog_versions_requires_at_least_one_release() {
        let changelog = r"# Changelog

<!-- Increment to skip CHANGELOG.md test: 0 -->
";

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

<!-- Increment to skip CHANGELOG.md test: 0 -->
";

        assert!(Changelog::parse(changelog)
            .expect_err("Parse changelog was successful.")
            .to_string()
            .contains("The first release must be version 0.1.0 or 0.1.0-git"))
    }

    #[test]
    fn parse_changelog_versions_last_version_must_be_empty() {
        let changelog = r"# Changelog

## 0.1.0

### Major

- A change

<!-- Increment to skip CHANGELOG.md test: 0 -->
";

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

<!-- Increment to skip CHANGELOG.md test: 0 -->
";

        assert!(Changelog::parse(changelog)
            .expect_err("Parse changelog was successful.")
            .to_string()
            .contains("Only the first version can be pre-release"))
    }

    #[test]
    fn get_or_create_release_mut_uses_first_when_prerelease() {
        let changelog_str = r"# Changelog

## 0.6.0-git

### Major

- A change

## 0.1.0

<!-- Increment to skip CHANGELOG.md test: 0 -->
";

        let mut changelog = Changelog::parse(&changelog_str).expect("Failed to parse changelog.");

        let current_release = changelog.get_or_create_release_mut();

        assert_eq!(current_release.version, Version::parse("0.6.0-git").unwrap());
    }

    #[test]
    fn get_or_create_release_mut_creates_new_when_current_is_released() {
        let changelog_str = r"# Changelog

## 0.6.0

### Major

- A change

## 0.1.0

<!-- Increment to skip CHANGELOG.md test: 0 -->
";

        let mut changelog = Changelog::parse(&changelog_str).expect("Failed to parse changelog.");

        let current_release = changelog.get_or_create_release_mut();

        assert_eq!(current_release.version, Version::parse("0.6.1-git").unwrap());
        // Assert the new release already inserted
        assert_eq!(changelog.releases.len(), 3);
    }
}
