# How to log a change?

All changes in this repository should go through a pull request. If a change modifies a published
crate, the change should be logged in the `CHANGELOG.md` file of that crate. Those files follow a
specific format parsed by different tools and scripts, in particular during continuous integration.

## Changelog format

A changelog file is a Markdown file named `CHANGELOG.md` at the root of a publish crate, i.e. next
to a `Cargo.toml` file with `package.publish = true` (which is the default but always explicit in
this repository). Its content is formatted as follows (see the [prelude
changelog](../../crates/prelude/CHANGELOG.md) for an example):

- It starts with a level-1 header titled `Changelog`.
- It contains a sequence of versions ordered by [SemVer] precedence (highest first).
- Each version is a level-2 header titled by the version number.
- Only the first version may be a pre-release and it must be `-git`.
- The first version matches the `Cargo.toml` version.
- Each version contains at least one `Major`, `Minor`, and/or `Patch` level-3 header, in that order.
- Each level-3 header lists the changes for that version (compared to the previous one, i.e. the one
  below) for that category (according to the [Cargo Book] where 0.x.y track major changes with x and
  minor/patch changes with y) in reverse-chronological order (most recent first).
- Each item in those lists is a short description of the change.
- The last version is 0.1.0 and contains no changes.
- The last line is a comment with a counter to skip the CI test for that changelog.

The `scripts/ci-changelog.sh` script verifies some of those rules, but not all.

## Logging a change

Changes are currently logged manually. When the pre-release version already exists at the correct
version, then this is just a matter of adding an item to the list for that category in the
pre-release version.

However, if this is the first change since the last release, or if the change needs to create a high
category that would update the pre-release version, then the `Cargo.toml` file needs to be modified
to update the version there too. If this crate is used by other crates, those crates also need to be
modified to reflect the new version. In turn, those crates also need to update their changelog file
with a patch change that their dependencies were updated. This can result in automatable manual
work. There should ultimately be a tool to update a changelog (and so recursively when needed).

[Cargo Book]: https://doc.rust-lang.org/cargo/reference/resolver.html#semver-compatibility
[SemVer]: https://semver.org/
