# commander
A lightweight command line tool built for quick command recall

## Install

Pick **one** of the two install methods, then add the matching shell setup below.

### Option A: Install from crate

```bash
cargo install cmdr-recall
```

The binary installs to `~/.cargo/bin/cmdr-recall`. Make sure `~/.cargo/bin` is on your `$PATH`.

### Option B: Build from source

```bash
git clone https://github.com/bglee/commander.git
cd commander
cargo build --release
```

The binary will be at `./target/release/cmdr-recall`.

## Shell Setup

Pick the snippet for your shell. If you used **Option A**, use `cmdr-recall` directly (it's already on your PATH). If you used **Option B**, replace `cmdr-recall` with the full path to the binary (e.g. `~/commander/target/release/cmdr-recall`).

### zsh

Add to `~/.zshrc`:

```zsh
function c() {
    fc -W
    selected=$(fc -rl 1 | sed 's/^[[:space:]]*[0-9]*[[:space:]]*//' | cmdr-recall)
    if [[ -n "$selected" ]]; then
        print -z "$selected"
    fi
}
```

Then: `source ~/.zshrc`

### bash

Add to `~/.bashrc`:

```bash
function c() {
    history -a
    selected=$(history | sed 's/^[[:space:]]*[0-9]*[[:space:]]*//' | cmdr-recall)
    if [[ -n "$selected" ]]; then
        history -s "$selected"
        echo "$selected"
        eval "$selected"
    fi
}
```

Then: `source ~/.bashrc`

---

Now run `c` to browse and fuzzy-search your command history.

## Trust Prompt

Commander supports project-level saved commands via a `.commander.json` file in the current directory. Because these commands are executed in your shell, a malicious repository could include a `.commander.json` that injects harmful commands.

When Commander detects an untrusted `.commander.json`, it will prompt you before loading it. **Review the file contents before trusting.** You can inspect it with:

```bash
cat .commander.json
```

- Press `y` to trust and load the file. The decision is stored in `~/.config/commander/trusted.json` (keyed by absolute path and SHA-256 hash).
- Press `n` to skip loading project commands for this session. Your stdin history still works normally.
- Press `ctrl+q` to quit.

If the file changes, Commander will re-prompt since the hash no longer matches.

### Why this matters

Commander's selected output gets `eval`'d by your shell with your full user permissions. A malicious `.commander.json` checked into a repo could slip dangerous commands into your list — and a single accidental Enter runs them. This is the same class of attack as a malicious `Makefile` or `.env` file: any file from an untrusted source that feeds into shell execution deserves scrutiny. Always read what you're about to trust.

## Secrets in the Terminal

You already know this: don't put secrets directly in commands. Your shell history records them, and Commander makes that history even more accessible. Saved commands in `.commander.json` are plaintext on disk. Use environment variables, config files, or a secret manager instead — never `--password=hunter2` inline.

## Todo
- [x] Fix the fuzzy search
- [x] Saved the commands
- [x] Handle bash
- [x] Build
- [x] Create crate
- [x] Clean up errors
- [x] Clean up tests

### Package Work
- [ ] Switch to release-please for automated version bumps (see [package_release.md] (package-release.md))
- [ ] Create deb package
- [ ] Create brew package
- [ ] Windows support (see [windows_support.md](windows_support.md))

### UX Work
- [ ] Update the UX for command templating and template running to use overlay
- [ ] Polish settings 

### Settings Work 
- [ ] add hirachical settings for local -> project -> global 
- [ ] merge saved and templating local -> project -> global


