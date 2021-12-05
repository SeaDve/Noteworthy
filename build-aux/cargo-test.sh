#! /bin/sh

set -o errexit
set -o pipefail
set -x

# $1 Passed by meson and should be the builddir
export CARGO_TARGET_DIR="$1/target/"
export CARGO_HOME="$CARGO_TARGET_DIR/cargo-home"

# If this is run inside a flatpak environment, append the export the rustc
# sdk-extension binaries to the path
if [ -f "/.flatpak-info" ]
then
    export PATH="$PATH:/usr/lib/sdk/rust-stable/bin"
fi

cargo fetch --locked
cargo test --all-features --offline -- --test-threads=1 --nocapture
