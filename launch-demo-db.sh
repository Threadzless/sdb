#!/usr/bin/env sh

# cd to script directory
cd "$(dirname "$0")"

surreal start file://sdb/example.db \
    --user demo_user \
    --pass demo_pass \
    --bind 127.0.0.1:8000
    --strict \
    --log demo-db.log