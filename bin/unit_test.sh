#!/bin/bash
set -euo pipefail

# Everything that runs with zero databases: the `*-postgres` / `postgres`
# crates are the database suite and live in bin/database_test.sh instead.
cargo test --workspace --exclude *-postgres --exclude postgres
