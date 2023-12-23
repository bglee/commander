#!/bin/zsh
#Stash the current session before running
history -w
touch t.txt
cargo run t.txt
ex=$(head -n 1 t.txt)
rm t.txt
eval $ex
