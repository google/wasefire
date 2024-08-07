# How to create a pull request?

All changes in this repository should go through a pull request.

## Documentation

The pull request should document why it is needed. This could be either omitted (for typos and other
obvious changes), a link to an issue, or a description in the pull request (for complex changes). In
both cases, the pull request title should be clear and concise.

If the change modifies the public API of a published crate, the API documentation should be
modified accordingly.

## Testing

You can run `./scripts/ci.sh` to locally run the tests that would run as part of the continuous
integration on GitHub. If this is your first change, you might need to run `./scripts/setup.sh`
after cloning the repository.

For faster iteration, you can run `./test.sh` from the directory where you are working. Such a
script is present at the root of all crates.

If the change adds a non-trivial functionality, additional tests should be added to make sure it is
working as intended and won't be broken in the future.

## Changelog

If the change modifies a published crate, the change should be logged (see the
[changelog](./changelog.md) documentation for more information).
