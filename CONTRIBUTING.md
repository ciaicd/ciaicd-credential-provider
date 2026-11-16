# Contributing

Contributions are welcome. Please open an issue describing the problem before a
large change, and keep pull requests focused. Security issues follow `SECURITY.md`.

## Development

Install Rust with rustup. The version, formatter and linter are pinned in
`rust-toolchain.toml`. The executable's dependencies are pinned in `Cargo.lock`;
commit lockfile changes when dependencies change.

```sh
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo install cargo-deny --version 0.21.0 --locked
cargo deny --locked check
```

Add meaningful tests for changed credential handling, bounded input, exact registry
matching, expiry, error redaction and the stdin/stdout protocol. Test data must be
synthetic. Never add a real JWT, credential file or private kubeconfig.

Build Linux release archives using `./package.sh` after installing the musl targets.
The release process and versioning policy are documented in `docs/releases.md`.
Use clear English in public documentation, commit messages, issues and pull requests.

## Licensing

By submitting a contribution, you agree to license it under the project's Apache
License 2.0. Preserve third-party copyright/license notices where applicable.
