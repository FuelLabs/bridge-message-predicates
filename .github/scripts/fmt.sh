#!/usr/bin/env bash

PROJECT=$1

cd $PROJECT
forc-fmt --check

if [ $PROJECT = 'contract-message-predicate' ]; then
    cargo fmt --verbose --check
fi
