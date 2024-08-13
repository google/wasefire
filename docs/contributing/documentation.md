# How to document code?

The goal of documentation is to specify what the code should do, for both those using and those
implementing the API. However, there is a trade-off between documentation benefits (specification
clarity) and documentation costs (reading and writing time).

## Guidelines

This repository makes the trade-off between documentation benefits and documentation costs in the
following way:
- All public items (function, module, trait, macro, etc) should be documented[^lints]. The amount of
  documentation to write may follow the spirit of the remaining guidelines.
- Functions with clear name and signature should not be documented. Names and signatures are part of
  the language, so in addition to reducing reading time, maintenance time is also reduced.
- Comments should be used for non-trivial logic where there is no clear API boundary to document it.
- Comments should not be used to repeat what the code does, but instead describe what the code
  should do at a higher abstraction level.

[^lints]: This should ideally be tested using the `rust.missing-docs` lint. See the [tracking
    issue][#565] for lints.

[#565]: https://github.com/google/wasefire/issues/565
