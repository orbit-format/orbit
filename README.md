# Orbit

[![CI](https://github.com/orbit-format/orbit/actions/workflows/ci.yml/badge.svg)](https://github.com/orbit-format/orbit/actions/workflows/ci.yml)

Orbit is a universal configuration language written in Rust. The repository contains the full toolchain: lexer, parser, AST, runtime evaluator, CLI, formatter, and integration tests. The full language specification lives in [`docs/language-spec.md`](docs/language-spec.md).

## Crates

| Crate | Description |
| ----- | ----------- |
| `orbit-core` | Language primitives: lexer, parser, AST, runtime, serializer |
| `orbit-cli` | Command-line interface (`orbit`) for parsing, evaluating, formatting |
| `orbit-fmt` | Formatter and pretty-printer |
| `orbit-tests` | Integration test suite |
| `examples/*` | Sample configs and embedding examples |

## Features

- Deterministic, typed configuration syntax inspired by HCL/TOML
- Zero-copy lexer with span tracking
- Recursive-descent parser that produces a typed AST
- Evaluator producing `OrbitValue` trees with duplicate-key detection
- CLI support for parsing, evaluating to JSON, formatting, and AST inspection
- Serializer hooks for JSON / YAML / MessagePack targets

## Development Workflow

1. `cargo fmt --all`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo test --all --all-features`
4. Add or update integration tests in `crates/orbit-tests`

GitHub Actions runs the same steps in [`ci.yml`](.github/workflows/ci.yml).

## Contributing

We welcome issues, feature proposals, and pull requests. Please read [`CONTRIBUTING.md`](CONTRIBUTING.md) and follow the [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md). Security issues should be reported privately following [`SECURITY.md`](SECURITY.md).

## License

BSD 3-Clause © Orbit contributors. See [`LICENSE`](LICENSE).
