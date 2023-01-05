#!/usr/bin/env sh

cd "$(dirname "$0")"

surreal start file://sdb/example.db \
    --user test_user \
    --pass test_pass \
    --bind 127.0.0.1:8000