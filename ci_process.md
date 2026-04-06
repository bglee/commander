# CI / Release Process

## Local

```bash
cargo build              # dev build
cargo build --release    # release build
cargo test               # run tests
cargo clippy             # lint
```

## Releasing

Tag and push — CI does the rest.

```bash
git tag v0.2.0
git push --tags
```

This triggers `.github/workflows/release.yml` which:

1. Patches `Cargo.toml` version from the tag
2. Cross-compiles for Linux (x86_64, aarch64) and macOS (x86_64, aarch64)
3. Creates a GitHub Release with tarballs attached
4. Publishes to crates.io

## Secrets

`CARGO_REGISTRY_TOKEN` must be set in the repo's GitHub Settings > Secrets. Generate it at https://crates.io/settings/tokens.
