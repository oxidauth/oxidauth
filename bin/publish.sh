#!/bin/bash

# for PROJECT in cli import-export http kernel permission postgres rs seed
for PROJECT in rs
do
    FOLDER=oxidauth-$PROJECT
    pushd $FOLDER && cargo publish && popd
done
