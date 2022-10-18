#!/usr/bin/env bash

# Runs doc-tests without capturing output.

# Source: <https://github.com/rust-lang/cargo/pull/9705>

RUSTDOCFLAGS="-Z unstable-options --nocapture" cargo +nightly test --doc