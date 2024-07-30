#!/bin/bash

cargo test --features "pidfd"
status1=$?
if [ $status1 -ne 0 ]; then
    echo "Tests with pidfd failed. Exiting."
    exit $status1
fi

cargo test --features "thread"
status1=$?
if [ $status1 -ne 0 ]; then
    echo "Tests with thread failed. Exiting."
    exit $status1
fi

cargo test --features "signal" -- --test-threads=1
status1=$?
if [ $status1 -ne 0 ]; then
    echo "Tests with signal failed. Exiting."
    exit $status1
fi
