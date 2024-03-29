#! /bin/bash

pushd oxidauth-http 2> /dev/null

hurl --test --variables-file hurl/variables-local hurl/public_keys_create.hurl

# we run the tests twice to make sure our clean up is solid
hurl --test --glob hurl/**/*.hurl --variables-file hurl/variables-local;
hurl --test --glob hurl/**/*.hurl --variables-file hurl/variables-local;

popd
