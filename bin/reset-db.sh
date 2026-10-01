#! /bin/bash

# DATABASE_URL (+ anything else sqlx needs) comes from the repo-root .env:
# postgres://oxidauth:oxidauth@postgres.oxidauth.localhost:5432/oxidauth —
# the aka tcp route onto the postgres container (no host port mapping).
# Stop the api first (`docker compose stop oxidauth-api`): sqlx cannot drop
# the database while the server pool holds connections.
# Values in .env must be literal — `source` expands $/`...` (compose does not).
[ -f "$(dirname "$0")/../.env" ] || { echo "no .env — cp example.env .env first" >&2; exit 1; }
set -a; source "$(dirname "$0")/../.env"; set +a

pushd "$(dirname "$0")/../src/oxidauth/oxidauth-postgres" 2> /dev/null

touch Cargo.toml

sqlx database drop -y
sqlx database create
sqlx migrate run

popd
