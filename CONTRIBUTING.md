# Contributing to Lumina

Thank you for dedicating your time to advancing the Lumina Reactive Language ecosystem! We are a community of systems developers, and we expect a high degree of rigor and professionalism from all merged contributions.

## Technical Setup

To build and compile the Lumina compiler from source:

1. Guarantee you have the latest stable Rust toolchain (`>=1.70.0`) configured via `rustup`.
2. For WebAssembly contributions, ensure `wasm-pack` is initialized in your `$PATH`.

## Standard Operating Procedure for Patches

1. **Format Validation**: All code must conform to strictly defined `rustfmt` rules. Run `cargo fmt` prior to commit.
2. **Static Analysis**: Changes must pass `clippy` checks strictly. Run `cargo clippy --workspace` to verify zero warnings.
3. **Test Coverage**: We mandate robust testing integration.
   * If you introduce a new feature or lexer token, add corresponding tests within `tests/spec/`.
   * If fixing a bug, implement a regression test validating the exact reproduction steps.

## Pull Request Lifecycle

1. Fork the `Lumina` repository.
2. Implement your architectural shifts synchronously on an isolated branch.
3. Once pushed, verify the GitHub Actions Continuous Integration pipeline reflects a passing (green) status.
4. Provide a highly detailed summary in your pull request, referencing specific AST manipulation and any benchmark metrics you obtained verifying performance non-regression.

## Code of Conduct

Always remain professional in code reviews. Treat your fellow engineers with scholarly respect. Emphasize data, technical merits, and algorithmic complexity over subjective opinions.
