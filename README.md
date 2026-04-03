# commander
A lightweight command line tool built for quick command recall

## Quick Start

```bash
# Build
cargo build --release
```

### Setup

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

Replace `/path/to/commander` with the actual path to the built binary (e.g. `~/commander/target/release/commander`).

Then reload your shell:

```bash
source ~/.zshrc
```

Now run `c` to browse and fuzzy-search your command history.

### Controls

- **Type** to filter
- **Arrow keys** or `Ctrl+J`/`Ctrl+K` to navigate
- **Enter** to select
- **Ctrl+S** to save a command
- **Ctrl+V** to toggle between all/saved commands
- **Ctrl+C** to copy to clipboard
- **Ctrl+Q** to quit

## Todo
- [ ] Fix the fuzzy search
- [ ] Saved the commands
- [ ] Create a config
- [ ] Handle bash 
- [ ] Build
- [ ] Create crate
- [ ] Create deb package
- [ ] Create brew package
- [x] Clean up errors
- [x] Clean up tests
