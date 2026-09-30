- [OXA-000024](https://www.pivotaltracker.com/story/show/OXA-000024) - **BREAKING (internal API)** `oxidauth-postgres`: misnamed module `select_public_key_by_user_id` renamed to `select_public_key_by_id`
    - The module always queried by primary key (`select_public_key_by_id.sql` is
      `WHERE id = $1`; `public_keys` has never had a `user_id` column, single
      migration, ever) — only the directory/`pub mod` path lied. The Rust path
      `oxidauth_postgres::public_keys::select_public_key_by_user_id` is removed;
      consumers repoint at `...::select_public_key_by_id`. Zero in-workspace
      consumers of the old path (proven by `cargo check --workspace` green after
      the rename); the label precedent for this class is
      `90000008-pg-repositories` ("BREAKING (internal API)").
    - Pure rename: no SQL, trait, DTO, wire, or span change. The
      `select_public_key_by_id_query` tracing span was already true-to-SQL and is
      untouched (owned by OXA-000052's ruling); the module's two `#[sqlx::test]`s
      pass byte-identical, and the stale `BUG(pinned)` misnomer comment was
      deleted with no assertion flips.
