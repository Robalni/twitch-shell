#!/bin/bash

cargo build --release
mkdir -p twitch-shell
cp target/release/twitch-shell twitch-shell/
kernel=$(uname -s | tr '[A-Z]' '[a-z]')
arch=$(uname -m)
version=$(sed -rn 's/^version ?= ?"([^"]+)"/\1/p' Cargo.toml)
tar -czf twitch-shell-$version-$kernel-$arch.tar.gz twitch-shell
rm twitch-shell/twitch-shell
rmdir twitch-shell
