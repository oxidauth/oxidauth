#!/bin/bash
set -euo pipefail

# hurl suite against the LIVE oxidauth-api (migration plan 11).
#
# Target resolution, in order:
#   1. OXIDAUTH_HURL_HOST (+ OXIDAUTH_HURL_PORT / OXIDAUTH_HURL_SCHEME) —
#      for CI or any run without the local compose stack.
#   2. `docker compose port oxidauth-api 80` at the repo root — the stack
#      publishes the API on an EPHEMERAL host port, so it must be read from
#      compose at run time, never hardcoded.
#
# The login secrets (client key + admin password) are NEVER stored in the
# tracked variables file: they come from the gitignored repo-root .env
# (OXIDAUTH_DEFAULT_CLIENT_KEY / OXIDAUTH_DEFAULT_ADMIN_PASSWORD) or from an
# already-exported environment, and are injected per run as --variable.
#
# The suite runs twice with the same `stamp` variable: every entity name the
# tests create embeds it, so the second pass only succeeds if the first pass
# deleted everything it created (unique constraints catch the residue). That
# is the old bin/hurl-tests.sh cleanup-verification trick, kept intact.

cd "$(dirname "$0")"
repo_root="$(cd ../.. && pwd)"

if [ -n "${OXIDAUTH_HURL_HOST:-}" ]; then
    scheme="${OXIDAUTH_HURL_SCHEME:-https}"
    target="${OXIDAUTH_HURL_HOST}${OXIDAUTH_HURL_PORT:+:${OXIDAUTH_HURL_PORT}}"
else
    scheme="${OXIDAUTH_HURL_SCHEME:-http}"
    mapping="$(cd "$repo_root" && docker compose port oxidauth-api 80 2>/dev/null | head -1)" || true
    if [ -z "$mapping" ]; then
        echo "hurl.sh: no OXIDAUTH_HURL_HOST set and oxidauth-api is not published by docker compose." >&2
        echo "  start the stack (docker compose up -d) or point me at one:" >&2
        echo "  OXIDAUTH_HURL_HOST=api.example.com OXIDAUTH_HURL_SCHEME=https ./src/oxidauth/hurl.sh" >&2
        exit 1
    fi
    target="127.0.0.1:${mapping##*:}"
fi

# Login secrets: prefer an exported environment, fall back to the repo .env.
if [ -z "${OXIDAUTH_DEFAULT_CLIENT_KEY:-}" ] || [ -z "${OXIDAUTH_DEFAULT_ADMIN_PASSWORD:-}" ]; then
    if [ -f "$repo_root/.env" ]; then
        set -a
        source "$repo_root/.env"
        set +a
    fi
fi
if [ -z "${OXIDAUTH_DEFAULT_CLIENT_KEY:-}" ] || [ -z "${OXIDAUTH_DEFAULT_ADMIN_PASSWORD:-}" ]; then
    echo "hurl.sh: OXIDAUTH_DEFAULT_CLIENT_KEY / OXIDAUTH_DEFAULT_ADMIN_PASSWORD are not set." >&2
    echo "  put them in the repo-root .env (see example.env) or export them." >&2
    exit 1
fi

# One stamp per INVOCATION (shared by both passes) — see header.
stamp="$(date +%s%N | tail -c 9)"

# The healthcheck endpoints echo the API crate's version; assert against the
# real Cargo.toml number rather than a copy pasted into a variables file.
api_version="$(bash "$repo_root/bin/crate_version.sh" oxidauth-api)"

echo "hurl.sh: target=$target scheme=$scheme stamp=$stamp api_version=$api_version"

vars=(
    --variables-file hurl/variables-local
    --variable "scheme=$scheme"
    --variable "host=$target"
    --variable "stamp=$stamp"
    --variable "api_version=$api_version"
    --variable "admin_client_key=$OXIDAUTH_DEFAULT_CLIENT_KEY"
    --variable "admin_password=$OXIDAUTH_DEFAULT_ADMIN_PASSWORD"
)

# make sure a jwt-signing keypair exists (bootstrap idempotency check)
hurl --test "${vars[@]}" hurl/public_keys_create.hurl

# pass 1 — exercise the suite; pass 2 — only green if pass 1 cleaned up
hurl --test "${vars[@]}" --glob "hurl/tests/*.hurl"
hurl --test "${vars[@]}" --glob "hurl/tests/*.hurl"
