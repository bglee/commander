#!/bin/zsh
pwd
touch t.txt
cargo run t.txt
ex=$(head -n 1 t.txt)
eval $ex
