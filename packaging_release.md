# Packaging & Release Plan

## Name Selection

The name `commander` is taken on crates.io and npm. These names are available across **all** checked registries (crates.io, Homebrew, apt, npm):

| Name | Notes |
|------|-------|
| **cmd-recall** | Short, descriptive, available everywhere |
| **cmdrecall** | No hyphen variant |
| **shell-recall** | Emphasizes the shell context |
| **commander-recall** | Keeps the original name, adds qualifier |
| **cmdr-recall** | Abbreviated |
| **cmdr-history** | Abbreviated + history-focused |

**Recommendation:** `cmd-recall` — short, easy to type, clearly describes what it does, available on every registry checked.

---

## 1. Cargo (crates.io)

### Cargo.toml changes

```toml
[package]
name = "cmd-recall"          # published crate name
version = "0.1.0"
edition = "2021"
description = "Fuzzy-searchable TUI for command history recall"
license = "MIT"
repository = "https://github.com/you/commander"
readme = "README.md"
keywords = ["cli", "tui", "history", "fuzzy", "shell"]
categories = ["command-line-utilities"]

[[bin]]
name = "cmd-recall"          # binary name users will invoke
path = "src/main.rs"
```

### Publishing

```bash
# 1. Create an account at https://crates.io and generate an API token
cargo login

# 2. Dry run — catches missing fields, oversized package, etc.
cargo publish --dry-run

# 3. Publish (immutable once pushed)
cargo publish
```

### User install

```bash
cargo install cmd-recall
# binary lands in ~/.cargo/bin/cmd-recall
```

### Updating

Bump `version` in `Cargo.toml`, then `cargo publish` again. Users run `cargo install cmd-recall` to get the new version.

---

## 2. Homebrew (macOS / Linux)

### Approach: Homebrew Tap

A tap is your own formula repo. No approval process — you control it.

### Setup

1. Create a GitHub repo named `homebrew-tap` (e.g. `github.com/you/homebrew-tap`).

2. Create a release workflow (see section 5) that builds binaries for macOS (arm64 + x86_64) and Linux (x86_64).

3. Attach the tarballs to a GitHub Release with predictable names:
   ```
   cmd-recall-v0.1.0-aarch64-apple-darwin.tar.gz
   cmd-recall-v0.1.0-x86_64-apple-darwin.tar.gz
   cmd-recall-v0.1.0-x86_64-unknown-linux-gnu.tar.gz
   ```

4. Write the formula in your tap repo at `Formula/cmd-recall.rb`:

```ruby
class CmdRecall < Formula
  desc "Fuzzy-searchable TUI for command history recall"
  homepage "https://github.com/you/commander"
  version "0.1.0"
  license "MIT"

  if OS.mac? && Hardware::CPU.arm?
    url "https://github.com/you/commander/releases/download/v0.1.0/cmd-recall-v0.1.0-aarch64-apple-darwin.tar.gz"
    sha256 "HASH_HERE"
  elsif OS.mac?
    url "https://github.com/you/commander/releases/download/v0.1.0/cmd-recall-v0.1.0-x86_64-apple-darwin.tar.gz"
    sha256 "HASH_HERE"
  elsif OS.linux?
    url "https://github.com/you/commander/releases/download/v0.1.0/cmd-recall-v0.1.0-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "HASH_HERE"
  end

  def install
    bin.install "cmd-recall"
  end

  test do
    assert_match "cmd-recall", shell_output("#{bin}/cmd-recall --version 2>&1", 1)
  end
end
```

### User install

```bash
brew tap you/tap
brew install cmd-recall
```

### Automating formula updates

Use [cargo-dist](https://opensource.axo.dev/cargo-dist/) or a custom CI step that updates the formula SHA and URL on each release.

---

## 3. APT (Debian / Ubuntu)

### Approach: GitHub-hosted APT repo via `cargo-deb`

Building a `.deb` is straightforward. Hosting it as a proper apt repo avoids the Debian packaging queue (which can take months).

### Setup

1. Add `cargo-deb` config to `Cargo.toml`:

```toml
[package.metadata.deb]
maintainer = "Your Name <you@example.com>"
copyright = "2024, Your Name"
license-file = ["LICENSE", "0"]
extended-description = "A fuzzy-searchable TUI for recalling shell command history"
section = "utils"
priority = "optional"
assets = [
    ["target/release/cmd-recall", "usr/bin/", "755"],
]
```

2. Build the `.deb`:

```bash
cargo install cargo-deb
cargo deb
# outputs target/debian/cmd-recall_0.1.0-1_amd64.deb
```

3. Host as a GitHub-hosted APT repo using a tool like [reprepro](https://wiki.debian.org/SettingUpSignedApt) or just attach the `.deb` to GitHub Releases for manual install.

### User install (manual .deb)

```bash
# Download from GitHub Releases
curl -LO https://github.com/you/commander/releases/download/v0.1.0/cmd-recall_0.1.0-1_amd64.deb
sudo dpkg -i cmd-recall_0.1.0-1_amd64.deb
```

### User install (apt repo, if you set one up)

```bash
# Add repo
curl -fsSL https://you.github.io/apt-repo/pubkey.gpg | sudo gpg --dearmor -o /usr/share/keyrings/cmd-recall.gpg
echo "deb [signed-by=/usr/share/keyrings/cmd-recall.gpg] https://you.github.io/apt-repo stable main" | sudo tee /etc/apt/sources.list.d/cmd-recall.list
sudo apt update
sudo apt install cmd-recall
```

---

## 4. Other Package Managers to Consider

### AUR (Arch Linux)

Arch users expect packages in the AUR. Create a `PKGBUILD`:

```bash
pkgname=cmd-recall
pkgver=0.1.0
pkgrel=1
pkgdesc="Fuzzy-searchable TUI for command history recall"
arch=('x86_64')
url="https://github.com/you/commander"
license=('MIT')
makedepends=('cargo')
source=("$pkgname-$pkgver.tar.gz::https://github.com/you/commander/archive/v$pkgver.tar.gz")
sha256sums=('HASH_HERE')

build() {
    cd "$srcdir/commander-$pkgver"
    cargo build --release --locked
}

package() {
    cd "$srcdir/commander-$pkgver"
    install -Dm755 "target/release/cmd-recall" "$pkgdir/usr/bin/cmd-recall"
    install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
}
```

Submit to https://aur.archlinux.org. Users install with `yay -S cmd-recall` or `paru -S cmd-recall`.

### Nix (NixOS / nix package manager)

Add a `flake.nix` to the repo. Nix users can then run directly without installing:

```bash
nix run github:you/commander
```

### cargo-binstall (pre-built binary shortcut)

If you attach binaries to GitHub Releases with the expected naming convention, users with `cargo-binstall` skip compilation entirely:

```bash
cargo binstall cmd-recall
# downloads the pre-built binary instead of compiling
```

No extra config needed if your release artifacts follow the `{name}-v{version}-{target}.tar.gz` pattern.

---

## 5. CI/CD Release Workflow

A single GitHub Actions workflow that triggers on version tags handles everything. The tag is the single source of truth for the version — CI patches `Cargo.toml` automatically so you never need to edit it by hand.

### How GitHub Actions YAML works

A workflow file is a hierarchy with four levels:

```
workflow                          # the file itself
  └─ jobs:                        # independent units of work
       └─ steps:                  # sequential commands within a job
            └─ (uses / run)       # what each step actually does
```

**Workflow level** — top of the file:
- `name:` — display name in the GitHub Actions UI
- `on:` — trigger events. `push: tags: ["v*"]` means "run when a tag starting with `v` is pushed"
- `permissions:` — what the workflow's `GITHUB_TOKEN` can do (e.g. `contents: write` to create releases)

**Jobs level** — `jobs:` contains named jobs that run in parallel by default:
- `runs-on:` — which OS/runner to use (e.g. `ubuntu-latest`, `macos-latest`)
- `needs:` — declares dependencies. `needs: build` means "wait for the `build` job to finish first". Without this, jobs run in parallel.
- `strategy.matrix:` — runs the same job multiple times with different variable combos. Each `include:` entry becomes a separate runner. You reference matrix values with `${{ matrix.varname }}`.

**Steps level** — `steps:` is an ordered list within a job. Each step is one of:
- `uses:` — runs a published action (someone else's reusable code). The format is `owner/repo@version`. Think of it like calling a function from a library.
- `run:` — runs a shell command directly.
- `name:` — optional label for the step (shows in the UI).
- `with:` — passes input parameters to a `uses:` action. Each action defines what inputs it accepts. This only works with `uses:`, not `run:`.
- `if:` — conditional. The step only runs when the expression is true.
- `env:` — sets environment variables for that step. `${{ secrets.X }}` pulls from your repo's encrypted secrets.

So when you see:

```yaml
- uses: dtolnay/rust-toolchain@stable    # action to call
  with:                                   # inputs to that action
    targets: ${{ matrix.target }}         # which rust target to install
```

That's: "run the `rust-toolchain` action published by `dtolnay`, and pass it `targets` as an input parameter, using the current matrix value."

And when you see:

```yaml
- name: Set version from tag
  run: |
    VERSION="${GITHUB_REF_NAME#v}"
    sed -i "s/^version = .*/version = \"$VERSION\"/" Cargo.toml
```

That's: "run this shell script directly on the runner."

### `.github/workflows/release.yml`

```yaml
name: Release

# ── Trigger ──────────────────────────────────────────────────────────────────
# Runs when you push a tag like v0.2.0
on:
  push:
    tags: ["v*"]

# ── Permissions ──────────────────────────────────────────────────────────────
# The GITHUB_TOKEN needs write access to create releases and upload artifacts
permissions:
  contents: write

# ── Jobs ─────────────────────────────────────────────────────────────────────
# Jobs run in parallel unless `needs:` declares a dependency.
jobs:

  # ── Build ────────────────────────────────────────────────────────────────
  # Runs 4 times in parallel (one per matrix entry), producing a binary for
  # each OS/arch combo.
  build:
    strategy:
      matrix:
        include:
          - target: x86_64-unknown-linux-gnu
            os: ubuntu-latest
          - target: aarch64-unknown-linux-gnu
            os: ubuntu-latest
          - target: x86_64-apple-darwin
            os: macos-latest
          - target: aarch64-apple-darwin
            os: macos-latest
    runs-on: ${{ matrix.os }}
    steps:
      # Check out the repo at the tagged commit
      - uses: actions/checkout@v4

      # Install the Rust toolchain for this matrix target
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      # Patch Cargo.toml version from the git tag (single source of truth)
      # v0.2.0 → 0.2.0, written into Cargo.toml so cargo build picks it up
      - name: Set version from tag
        run: |
          VERSION="${GITHUB_REF_NAME#v}"
          sed -i'' -e "s/^version = .*/version = \"$VERSION\"/" Cargo.toml

      # Linux ARM needs `cross` (a cross-compilation tool) because the
      # runner is x86 but the target is aarch64
      - name: Install cross (Linux ARM)
        if: matrix.target == 'aarch64-unknown-linux-gnu'
        run: cargo install cross

      # Build the release binary
      - name: Build
        run: |
          if [ "${{ matrix.target }}" = "aarch64-unknown-linux-gnu" ]; then
            cross build --release --target ${{ matrix.target }}
          else
            cargo build --release --target ${{ matrix.target }}
          fi

      # Tar up the binary with a predictable name for the release
      - name: Package
        run: |
          cd target/${{ matrix.target }}/release
          tar czf ../../../cmdr-recall-${{ github.ref_name }}-${{ matrix.target }}.tar.gz cmdr-recall

      # Upload the tarball so the `release` job can download it later
      - uses: actions/upload-artifact@v4
        with:
          name: cmdr-recall-${{ matrix.target }}
          path: cmdr-recall-*.tar.gz

  # ── Release ──────────────────────────────────────────────────────────────
  # Waits for all build matrix jobs to finish, downloads all artifacts,
  # and creates a GitHub Release with the tarballs attached.
  release:
    needs: build
    runs-on: ubuntu-latest
    steps:
      - uses: actions/download-artifact@v4
        with:
          merge-multiple: true
      - uses: softprops/action-gh-release@v2
        with:
          files: cmdr-recall-*.tar.gz
          generate_release_notes: true

  # ── Publish to crates.io ─────────────────────────────────────────────────
  # Runs in parallel with build (no `needs:`). Patches the version from the
  # tag and publishes.
  publish-crate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Set version from tag
        run: |
          VERSION="${GITHUB_REF_NAME#v}"
          sed -i "s/^version = .*/version = \"$VERSION\"/" Cargo.toml
      - run: cargo publish
        env:
          CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}

  # ── Build .deb ───────────────────────────────────────────────────────────
  # Builds a Debian package and attaches it to the same GitHub Release.
  build-deb:
    needs: release
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Set version from tag
        run: |
          VERSION="${GITHUB_REF_NAME#v}"
          sed -i "s/^version = .*/version = \"$VERSION\"/" Cargo.toml
      - run: cargo install cargo-deb
      - run: cargo deb
      - uses: softprops/action-gh-release@v2
        with:
          files: target/debian/*.deb
```

### Release process

The tag is the only thing you touch. CI handles everything else.

```bash
# 1. Tag the current commit
git tag v0.2.0

# 2. Push the tag — this triggers the workflow
git push --tags

# 3. CI automatically:
#    - patches Cargo.toml with 0.2.0
#    - builds binaries for 4 targets
#    - creates GitHub Release with .tar.gz and .deb attached
#    - publishes to crates.io
#    - (you update Homebrew formula SHA hashes, or automate that too)
```

### Visual: what runs when

```
git push --tags v0.2.0
  │
  ├─► build (4x parallel via matrix)
  │     ├─ x86_64-linux    ─┐
  │     ├─ aarch64-linux    │
  │     ├─ x86_64-macos     ├─► release (creates GitHub Release, attaches tarballs)
  │     └─ aarch64-macos   ─┘       │
  │                                  └─► build-deb (attaches .deb to same release)
  │
  └─► publish-crate (parallel, publishes to crates.io)
```

---

## 6. Security Scanning

### In CI (add to your existing CI workflow)

```yaml
security:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
    - run: cargo install cargo-audit cargo-deny
    - run: cargo audit          # checks deps against RustSec advisory DB
    - run: cargo deny check     # license compliance + advisory + duplicate deps
```

### `cargo audit`

Checks `Cargo.lock` against the [RustSec Advisory Database](https://rustsec.org). Catches known CVEs in your dependency tree. Run it in CI on every PR and on a weekly cron schedule to catch newly published advisories.

```bash
cargo install cargo-audit
cargo audit
```

### `cargo deny`

More comprehensive — checks licenses, bans specific crates, flags duplicate versions, plus advisory checks. Configure via `deny.toml`:

```bash
cargo install cargo-deny
cargo deny init    # generates deny.toml
cargo deny check
```

### `cargo vet`

Mozilla's supply-chain audit tool. Tracks whether each dependency version has been reviewed by you or an organization you trust. Heavier process but strongest assurance.

### GitHub Dependabot

Add `.github/dependabot.yml` to get automatic PRs when dependencies have updates or security fixes:

```yaml
version: 2
updates:
  - package-ecosystem: "cargo"
    directory: "/"
    schedule:
      interval: "weekly"
```

---

## Priority Order

| Priority | Task | Effort |
|----------|------|--------|
| 1 | Publish to crates.io | ~30 min |
| 2 | GitHub Actions release workflow | ~2 hours |
| 3 | Homebrew tap | ~1 hour (after CI builds binaries) |
| 4 | `.deb` via cargo-deb | ~1 hour (after CI builds binaries) |
| 5 | AUR PKGBUILD | ~30 min |
| 6 | cargo-deny / cargo-audit in CI | ~30 min |
| 7 | Nix flake | ~1 hour |
| 8 | Hosted APT repo | ~2 hours |
