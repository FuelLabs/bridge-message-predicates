#!/usr/bin/env bash

PROJECT=$1

if [ $PROJECT = 'contract-message-script' ]; then
    cd $PROJECT
    forc test
fi
