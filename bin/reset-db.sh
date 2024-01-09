#! /bin/bash

pushd oxidauth-postgres 2> /dev/null

export DATABASE_URL=postgres://oxidauth:oxidauth@127.0.0.1:5432/oxidauth

touch Cargo.toml

sqlx database drop -y
sqlx database create
sqlx migrate run

popd
