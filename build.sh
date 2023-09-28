#!/bin/bash
cargo build --release
strip --strip-all target/release/http_health_checker
