#!/bin/zsh
fc -W
selected=$(fc -rl 1 | sed 's/^[[:space:]]*[0-9]*[[:space:]]*//' | ./target/debug/cmdr-recall)
if [[ -n "$selected" ]]; then
    print -z "$selected"
fi
