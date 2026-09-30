#!/bin/bash
set -euo pipefail

# Database-backed tests (migration plan 11): the crates the template's
# unit_test.sh excludes. DATABASE_URL/READ_DATABASE_URL come from the
# repo-root .env (host port the compose stack maps onto postgres);
# MIGRATIONS_ENABLED=true makes the plan-05 `database!` macro run the crate
# MIGRATOR before the tests instead of trusting a hand-migrated schema.

cd "$(dirname "$0")"

set -a
source ../../.env
set +a

export MIGRATIONS_ENABLED=true

cargo test -p oxidauth-postgres -p postgres -- --nocapture
