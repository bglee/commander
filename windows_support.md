# Windows Support

## Clipboard

`pbcopy` is macOS-only. Windows needs `clip.exe`. Fix with conditional compilation or runtime detection.

## `use-dev-tty`

The crossterm `use-dev-tty` feature reads keyboard input from `/dev/tty` while stdin carries piped data. That's Unix-only. However, on Windows, crossterm reads from the console input handle (`CONIN$`) by default, which is already separate from a piped stdin — so it *might* just work without `use-dev-tty`. Needs testing.

Make `use-dev-tty` a Unix-only feature in `Cargo.toml`.

## Shell function

PowerShell equivalent for `~/$PROFILE`:

```powershell
function c {
    $selected = Get-History | ForEach-Object { $_.CommandLine } | /path/to/commander.exe
    if ($selected) {
        Invoke-Expression $selected
    }
}
```

## Steps

1. Make `use-dev-tty` a Unix-only feature in `Cargo.toml`
2. Swap `pbcopy` for `clip.exe` on Windows
3. Add PowerShell setup to the README
4. Test that piped stdin + console input works on Windows
