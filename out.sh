#!/bin/zsh
fc -W
selected=$(fc -rl 1 | sed 's/^[[:space:]]*[0-9]*[[:space:]]*//' | ./target/debug/commander)
if [[ -n "$selected" ]]; then
    eval "$selected"
fi
