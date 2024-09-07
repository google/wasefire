# How to change code?

All changes in this repository should go through a pull request.

## Review

If a pull request has review comments that must be addressed by changing the code, those changes
should be additional commits on top of the existing ones. Reviewers can then only review those
additional changes.

Pull requests are squash-merged such that only the title and description are used to build a single
commit with all the changes.

## Documentation

If a pull request is not trivial (fixing a typo or other obvious change), it should have a
description. In all cases, the pull request title should be clear and concise.

If a pull request adds or modifies specified code, documentation should be added or updated to make
sure everyone agrees on what the code should do.

See the [documentation page](./documentation.md) for more information.

## Testing

If a pull request adds non-trivial code, tests should be added to make sure it is working as
intended and won't be broken in the future.

See the [testing page](./testing.md) for more information.

## Changelog

If a pull request modifies a published crate, the change should be logged to make sure it will be
documented when released.

See the [changelog page](./changelog.md) for more information.
