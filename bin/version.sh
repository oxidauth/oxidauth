#!/bin/bash
set -euo pipefail

# Move every crate in the workspace to one version, in one step. That number is
# the tag every service image gets (`src/oxidauth/oxidauth-api/build/build.sh`
# from plan 12 tags its image `:<crate version>` via bin/crate_version.sh) — so
# bump it here and everything that advertises a version agrees. The workspace
# is still divergent (kernel 0.4.0, http 0.9.0, api 0.8.0, rs 0.4.0-rc2, xlib
# 0.1.0 …); this is the tool that ends that, per migration plan 11/14.
#
#   ./bin/version.sh                      # what the crates say now
#   ./bin/version.sh --dry-run 0.2.0      # what would change
#   ./bin/version.sh 0.2.0                # every crate -> 0.2.0
#
# It is a thin, checked wrapper around cargo-edit's `cargo set-version <version>
# --workspace`, which rewrites every member's Cargo.toml and refreshes
# Cargo.lock; the checks here are the ones that tool has been seen to skip.
# One-time install per machine:
#
#   cargo install --no-default-features --features set-version cargo-edit
#
# Not the standalone `cargo-set-version` crate: it takes --workspace/--dry-run,
# ignores them, bumps part of the workspace, and leaves the lock untouched.
#
# cargo-edit will not go backwards, so a downgrade rewrites the package version
# lines directly — same result, same checks afterwards.

cd "$(dirname "$0")/.."

show() { # every member crate and its version, straight from cargo metadata
    cargo metadata --format-version 1 --no-deps |
        tr '{' '\n' |
        grep -o '"name":"[a-z0-9_-]*","version":"[^"]*"' |
        sed -e 's/^"name":"//' -e 's/","version":"/  /' -e 's/"$//' |
        sort
}

reject() {
    echo "version.sh: '$1' is not usable as a version and a docker tag" >&2
    echo "  start with a letter or digit, then [A-Za-z0-9._-], no '..' —" >&2
    echo "  e.g. 0.2.0, 1.0.0-rc.1" >&2
    exit 2
}
case "${1:-}" in

    ""|show|--show)
        show
        exit 0
        ;;
    -h|--help)
        echo "usage: $(basename "$0") [--dry-run] <version>   (no arguments: show)"
        exit 0
        ;;
esac

dry_run=0
if [ "${1:-}" = "--dry-run" ]; then
    dry_run=1
    shift
fi

[ $# -eq 1 ] || {
    echo "usage: $(basename "$0") [--dry-run] <version>   (or --show)" >&2
    exit 2
}

version="$1"

# A version here is also a docker tag, and tags are stricter than semver; cargo
# set-version still has the last word on whether it is semver at all.
case "$version" in
    [0-9A-Za-z]*) ;;
    *) reject "$version" ;;
esac
case "$version" in
    *[!0-9A-Za-z._-]*|*..) reject "$version" ;;
esac

# cargo finds external subcommands on PATH, and cargo-edit's takes only
# versions, so probing it with a flag would fail for the wrong reason.
command -v cargo-set-version >/dev/null 2>&1 || {
    echo "version.sh: \`cargo set-version\` is not installed" >&2
    echo "  cargo install --no-default-features --features set-version cargo-edit" >&2
    exit 1
}

before="$(bash bin/crate_version.sh oxidauth-api)"

if [ "$dry_run" = 1 ]; then
    cargo set-version "$version" --workspace --dry-run

    # The standalone `cargo-set-version` crate takes --dry-run and writes
    # anyway, so a dry run is only trustworthy if nothing moved.
    after="$(bash bin/crate_version.sh oxidauth-api)"
    [ "$before" = "$after" ] || {
        echo "version.sh: --dry-run rewrote the manifests ($before -> $after)." >&2
        echo "  That is the incomplete standalone crate, not cargo-edit's:" >&2
        echo "    cargo uninstall cargo-set-version" >&2
        echo "    cargo install --no-default-features --features set-version cargo-edit" >&2
        echo "  Then undo what it wrote (tracked manifests) and bump the untracked" >&2
        echo "  crates back: git checkout -- 'src/*/Cargo.toml' && $(basename "$0") $before" >&2
        exit 1
    }
    exit 0
fi

# Version first: cargo-edit takes flags on either side, and it is also the only
# ordering the standalone crate parses.
if ! out="$(cargo set-version "$version" --workspace 2>&1)"; then
    case "$out" in
        *downgrade*)
            # cargo-edit will not go backwards, and a bump nobody can undo is a
            # trap. Its rewrite moves three things — every package version line,
            # every path-dep version requirement on a workspace member, and the
            # lock — so the fallback must too. (The lock is refreshed by the
            # resolve check below, once the manifests agree again.) Member names
            # are still readable with --no-deps: it does not resolve.
            echo "==> cargo-edit will not downgrade; setting the version directly"
            members="$(cargo metadata --format-version 1 --no-deps |
                tr '{' '\n' |
                grep -o '"name":"[a-z0-9_-]*","version' |
                sed -e 's/^"name":"//' -e 's/","version//' |
                tr '\n' ' ')"
            cargo metadata --format-version 1 --no-deps |
                tr '{' '\n' |
                grep -o '"manifest_path":"[^"]*"' |
                sed -e 's/^"manifest_path":"//' -e 's/"$//' |
                while IFS= read -r manifest; do
                    awk -v version="$version" -v members="$members" '
                        BEGIN { n = split(members, m, " ") }
                        !moved && /^version = / {
                            print "version = \"" version "\""
                            moved = 1
                            next
                        }
                        {
                            # `member = { version = "x", path = ... }`: only lines
                            # with both `version` and `path`, so a same-named
                            # crates.io dep (http, postgres) is never touched.
                            for (i = 1; i <= n; i++) {
                                if ($0 ~ "^" m[i] " = " && $0 ~ /version = "[^"]*"/ && $0 ~ /path = /) {
                                    sub(/version = "[^"]*"/, "version = \"" version "\"")
                                    break
                                }
                            }
                            print
                        }
                    ' "$manifest" > "$manifest.tmp" && mv "$manifest.tmp" "$manifest"
                done

            ;;
        *)
            printf '%s\n' "$out" >&2
            echo "version.sh: cargo set-version failed for $version" >&2
            case "$out" in
                *"failed to select"*)
                    echo "  the workspace did not resolve before this run either:" >&2
                    echo "  compare manifests against the lock and undo the mismatch:" >&2
                    echo "    git diff -- '*Cargo.toml' Cargo.lock" >&2
                    ;;
            esac
            exit 1
            ;;
    esac
fi

# The bump is only real once cargo can resolve the workspace again: a partial
# bump leaves path-dep requirements pointing at versions that do not exist. This
# resolve also refreshes Cargo.lock.
cargo metadata --format-version 1 >/dev/null || {
    echo "version.sh: the workspace does not resolve after the bump to $version" >&2
    echo "  compare the manifests: git diff -- '*Cargo.toml'" >&2
    exit 1
}

echo "==> every workspace crate is now $version (Cargo.lock refreshed)"
show

cat <<EOF

next steps:
  bin/build.sh      # pushes <image>:$version for every service
  bin/publish.sh    # publishes the crates in topological order
  bin/unit_test.sh  # cargo test --workspace (minus the database crates)
EOF
