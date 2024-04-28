#!/bin/bash
cargo build --release
watchexec -e html,css,js -w hyper-src -w templates -w styles ./html &
./target/release/server

