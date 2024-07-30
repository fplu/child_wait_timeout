#!/bin/bash

cargo bench --features "pidfd"
cargo bench --features "thread"
cargo bench --features "signal" 
