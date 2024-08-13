# How to test code?

The goal of testing is to provide confidence that the code is correct, i.e. the code implements its
specification as described in its [documentation]. However, there is a trade-off between testing
benefits (confidence in code correctness) and testing costs (testing time). Note that maintainance
and development velocity are impacted both positively and negatively by testing: good confidence
permits developing faster, but too much testing time slows down development.

## Guidelines

This repository makes the trade-off between testing benefits and testing costs in the following way:
- Trivial code should not be tested.
- Non-trivial code should be unit-tested (best effort coverage).
- Code processing non-trivial user inputs (e.g. parsers) should also be fuzzed (ideally with a
  "breadth-first" surjection from fuzzing input to possible user inputs, i.e. small fuzzing inputs
  provide very good coverage and any user input can be expressed with a fuzzing input).
- In particular, non-exhaustive (or random) property testing should be avoided. If the search space
  is too large, then fuzzing should be used because it is coverage-guided. Otherwise, unit-testing
  should be used because it is deterministic and reliable.

Because test is code, testing also applies to testing, with the same guidelines as above. Ideally,
all tests are trivial and thus don't need to be tested. Otherwise, there is a [tracking issue][#559]
for mutation testing, which besides testing coverage of tests may also help find correctness issues
in tests.

## Tools

The `./scripts/ci.sh` script permits running the continuous integration locally. (Note that it
doesn't run the `./scripts/setup.sh` script. You'll need to run it manually if this is the first
time you clone the repository.)

Because the continuous integration tests everything, it is rather slow and should not be used for
fast iterations. Each crate provides a `./test.sh` script at its root. You can run this script to
only test that crate.

Fuzzing is not yet automated on GitHub (see the [tracking issue][#496] for continuous fuzzing). It
is manually run using `cargo fuzz`. The fuzzing targets can be listed with `cargo fuzz list` and a
specific target may be fuzzed with `cargo fuzz run TARGET`.

[documentation]: ./documentation.md
[#496]: https://github.com/google/wasefire/issues/496
[#559]: https://github.com/google/wasefire/issues/559
