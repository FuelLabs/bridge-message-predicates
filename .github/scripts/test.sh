#!/usr/bin/env bash

PROJECT=$1

if [ $PROJECT = 'contract-message-predicate' ]; then
    cd $PROJECT
    forc test
fi
