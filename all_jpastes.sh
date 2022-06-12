#!/usr/bin/env bash
set -euo pipefail

redis-cli --raw keys 'jpaste:*' | while read -r KEY; do
    VAL=$(redis-cli get "${KEY}")
    TTL=$(redis-cli ttl "${KEY}")
    echo "${KEY} (${TTL}) -> ${VAL:0:64}"
done
