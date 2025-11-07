# Code review instructions

## General guidance

### Duplicate findings

Review comments should be about aspects of software development (like correctness, security,
maintenance, performance, user experience, etc) that are not already captured by the continuous
integration.

For example, all code is supposed to type-check. This is checked by the continuous integration based
on the `test.sh` files in each directory. So, while there is no need to review whether the code
type-checks, it is important to review that the configurations listed in the `test.sh` files are
covering all the code (like when a new cargo feature is added).

### Actionable findings

Review comments should be actionable. There is no need to create a review comment only to explain
what the code is doing. A review comment should demonstrate an issue in the pull request.

## Specific guidance

### Panics in examples

It is ok for code under `examples/rust` to panic.

### Variable quoting in shell scripts

It is ok to not quote variables that are known to expand to themselves.

### Magic numbers

It is ok to have magic numbers used in exactly one place. Named constants are only needed when the
value needs to be shared in multiple places.
