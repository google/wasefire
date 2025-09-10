# Code review instructions

## General guidance

Please only focus your code review on aspects of software development (like correctness, security,
maintenance, performance, user experience, etc) that are not already captured by the continuous
integration.

For example, all code is supposed to type-check. This is checked by the continuous integration based
on the `test.sh` files in each directory. So, while there is no need to review whether the code
type-checks, it is important to review that the configurations listed in the `test.sh` files are
covering all the code (like when a new cargo feature is added).

## Specific guidance

### Panics in examples

It is ok for code under `examples/rust` to panic.
