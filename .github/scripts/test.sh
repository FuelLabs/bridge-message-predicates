#!/usr/bin/env bash

PROJECT=$1

if [ $PROJECT = 'contract-message-predicate' ]; then
    forc build
    cargo test
fi
