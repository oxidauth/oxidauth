#!/bin/bash
set -euo pipefail

# Publish the oxidauth crates to crates.io in dependency (topological) order:
# the API-facing crates depend on the stack beneath them, and cargo resolves a
# path-dependency's version requirement against the *registry* once published,
# so a dependency must exist before anything that requires it. `permission`
# comes FIRST — oxidauth-kernel path-depends on oxidauth-permission, so
# kernel is not a root: permission is the only one.
#
# Order: permission → kernel → repository → postgres → services → http →
#        rs → api
#
#   src/oxidauth/oxidauth-rs is the crate `oxidauth` (the client library).
#
# The release identity is the API crate's version (it is also what
# /api/v1/__meta/healthcheck reports and what plan 12 tags the image with), so
# it is also the git tag this script cuts. Version comes from Cargo.toml via
# bin/crate_version.sh — never from a hardcoded string here.

cd "$(dirname "$0")/.."

# `cargo publish` packages the WORKING TREE while the git tag points at HEAD:
# a dirty tree would ship content that no commit contains, and the tag would
# lie about it. Refuse to run, full stop (no --allow-dirty anywhere).
if [ -n "$(git status --porcelain)" ]; then
    echo "publish.sh: the working tree is dirty — refusing to publish (cargo" >&2
    echo "  publishes the tree, the git tag would name HEAD; they must agree)." >&2
    echo "  Commit or stash first; git status --porcelain:" >&2
    git status --porcelain | sed -e 's/^/    /' >&2
    exit 1
fi

for PROJECT in permission kernel repository postgres services http rs api
do
    FOLDER=src/oxidauth/oxidauth-$PROJECT
    # the client crate lives in oxidauth-rs but publishes as `oxidauth`
    case "$PROJECT" in
        rs) CRATE=oxidauth ;;
        *)  CRATE=oxidauth-$PROJECT ;;
    esac
    echo "==> publishing $FOLDER (crate $CRATE $(bash bin/crate_version.sh "$CRATE"))"
    pushd "$FOLDER" && cargo publish && popd
done

# Tag the commit that was just published, semver-style (tree verified clean
# above, so HEAD is exactly what shipped).
VERSION="$(bash bin/crate_version.sh oxidauth-api)"
TAG="v$VERSION"
if git rev-parse -q --verify "refs/tags/$TAG" >/dev/null; then
    echo "==> tag $TAG already exists; leaving it alone"
else
    git tag "$TAG"
    echo "==> tagged HEAD $TAG (push with: git push origin $TAG)"
fi
