#!/usr/bin/env bash

# this script DELETES ALL DATABASES in a postgres instance.
# use cautiously. 
if ! [ -x "$(command -v pqsl)" ]; then
  echo >&2 "Error: psql is not installed."
  echo >&2 "Install it according to your environment."
  exit 1
fi
psql -U postgres -d postgres -t -c "SELECT format('DROP DATABASE %I;', datname) FROM pg_database WHERE datistemplate = false AND datname <> 'postgres' AND datname <> 'authen_test';" | psql -U postgres -d postgres