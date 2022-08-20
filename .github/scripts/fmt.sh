#!/usr/bin/env bash

PROJECT=$1

cd $PROJECT
cargo fmt --verbose --check
