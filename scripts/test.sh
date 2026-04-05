#!/usr/bin/env bash

# this script runs the tests, forwarding the bunyan json output to the jq formatter

if ! [ -x "$(command -v cargo)" ]; then
  echo >&2 "Error: rust is not installed."
  echo >&2 "Install it according to your environment."
  exit 1
fi

cargo test --quiet --package authen --test api -- --nocapture --format=terse  | jq -CSR 'fromjson?'