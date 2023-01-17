#!/bin/bash
cd $(dirname "${BASH_SOURCE[0]}")
NEW_DIR=logs/$(date +%Y%m%d)
mkdir -p $NEW_DIR
mv logs/*.json.gz $NEW_DIR
