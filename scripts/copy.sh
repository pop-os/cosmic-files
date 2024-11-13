#!/usr/bin/env bash

set -ex
cargo fmt
cargo build --release --example copy
rm -rf test
mkdir test
cp -a samples test/a
mkdir test/a/link
touch test/a/link/a
ln -s a test/a/link/b
mkdir test/a/perms
touch test/a/perms/400
chmod 400 test/a/perms/400
touch test/a/perms/600
chmod 600 test/a/perms/600
touch test/a/perms/700
chmod 700 test/a/perms/700
time target/release/examples/copy
ls -lR test
meld test/a test/c
