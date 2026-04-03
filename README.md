# commander
A lightweight command line tool built for quick command recall

## Build and Start

```bash
# Build
cargo build --release
```

### Setup (zsh)

Add the following to your `~/.zshrc`:

```zsh
function c() {
    fc -W
    selected=$(fc -rl 1 | sed 's/^[[:space:]]*[0-9]*[[:space:]]*//' | /path/to/commander)
    if [[ -n "$selected" ]]; then
        print -z "$selected"
    fi
}
```

Then: `source ~/.zshrc`

### Setup (bash)

Add the following to your `~/.bashrc`:

```bash
function c() {
    history -a
    selected=$(history | sed 's/^[[:space:]]*[0-9]*[[:space:]]*//' | /path/to/commander)
    if [[ -n "$selected" ]]; then
        history -s "$selected"
        echo "$selected"
        eval "$selected"
    fi
}
```

Then: `source ~/.bashrc`

---

Replace `/path/to/commander` with the actual path to the built binary (e.g. `~/commander/target/release/commander`).

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

## Todo
- [x] Fix the fuzzy search
- [x] Saved the commands
- [x] Handle bash
- [ ] Build
- [ ] Create crate
- [ ] Create deb package
- [ ] Create brew package
- [ ] Windows support (see [windows_support.md](windows_support.md))
- [x] Clean up errors
- [x] Clean up tests
