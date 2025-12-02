# Contributing to Orbit

Thanks for helping build Orbit! This document explains how to set up your environment, make changes, and get them reviewed.

## Code of Conduct

Participation in this project is governed by the [Contributor Covenant](CODE_OF_CONDUCT.md). Please read it before engaging with the community.

## Prerequisites

- Rust stable (install via [rustup](https://rustup.rs))
- `cargo fmt`, `cargo clippy`, and `cargo test` installed via `rustup component add rustfmt clippy`
- Optional: Node.js for the `examples/load-config` bindings, though not required for most changes

## Project Layout

- `crates/orbit-core`: lexer, parser, AST, runtime, serializers
- `crates/orbit-cli`: binary providing `orbitc`
- `crates/orbit-fmt`: formatting logic (used by CLI and future bindings)
- `crates/orbit-tests`: black-box integration tests
- `examples/`: sample configs or embedding demos

## Development Workflow

1. **Fork & clone** the repository.
2. **Create a branch** for your change.
3. **Make changes** with helpful commit messages.
4. **Run the checks** locally:
   ```powershell
   cargo fmt --all
   cargo clippy --all-targets --all-features -- -D warnings
   cargo test --all --all-features
   ```
5. **Update docs/tests**:
   - Update `README.md`, or crate-level docs when changing behavior.
   - Add unit tests in the relevant crate and integration tests in `orbit-tests`.
6. **Open a pull request** using the provided template and link any related issues.

## Coding Guidelines

- Keep public APIs stable; propose breaking changes via an RFC in Discussions.
- Favor zero-copy data structures and span-aware error reporting.
- Provide descriptive errors through the `error` module types.
- Document non-obvious logic with concise comments.
- Follow Rust 2021 idioms and keep `clippy` warnings at zero.

## Commit & PR Expectations

- Squash commits if requested by reviewers.
- Reference issues with `Fixes #123` or `Closes #123` when appropriate.
- Ensure CI is green or explain any known failures (e.g., platform-specific blockers).
- Include before/after samples for syntax, formatter, or serializer changes.

## Release Process

Maintainers tag releases following semantic versioning. Contributors do not need to update the changelog unless asked during review.

## Questions?

Use GitHub Discussions for open-ended questions, or the issue templates for bugs and feature requests.
