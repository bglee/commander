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
