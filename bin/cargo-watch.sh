#! /bin/bash

DATABASE_URL="postgres://oxidauth:oxidauth@127.0.0.1:5432/oxidauth" cargo watch -c -s 'cargo test -- --nocapture'
