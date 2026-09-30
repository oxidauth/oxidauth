- [OXA-000026](https://www.pivotaltracker.com/story/show/OXA-000026) - list-all responses are now deterministically ordered: every SELECT-all gained `ORDER BY created_at ASC, id ASC`
    - **User-visible: JSON array order in `GET /api/v1/users`, `/api/v1/authorities`,
      `/api/v1/public_keys`, `/api/v1/permissions`, `/api/v1/roles` is now
      deterministic** — rows come back oldest-created first, ties broken by `id`.
      Previously the five repository queries (`select_all_users_query.sql`,
      `select_all_authorities.sql`, `select_all_public_keys.sql`,
      `select_all_permissions.sql`, `select_all_roles.sql`) had no `ORDER BY`, so
      array order was whatever the Postgres plan/heap produced: byte-unstable
      responses, order could shuffle after `VACUUM`/`ANALYZE` or a plan change, and
      read replicas could disagree. No client may legitimately have depended on the
      old non-order; no shape, route, type, or schema change.
    - `id` is a mandatory tie-break, not decoration: `CURRENT_TIMESTAMP`/`NOW()` is
      the *transaction* start time, so any bulk or same-transaction seed ties every
      row on `created_at` — `created_at` alone still yields arbitrary permutations.
      `updated_at` is never used. Direction `ASC` is the listing default for all
      five; flip together (with an API-doc note) if an admin UX wants newest-first.
    - **Bootstrap note: `public_keys.pop()` now deterministically picks the NEWEST
      signing key at boot.** `oxidauth-services/src/bootstrap/mod.rs` lists all
      public keys and `Vec::pop` takes the *last* row, so with `ASC` the existing-key
      reuse path uses the most recently created key — previously the "most recent"
      choice was an accident of heap order. This is an improvement; do not "fix" it
      back (rotation hygiene stays owned by OXA-000019).
    - Scope: the register (DATA-13) named three queries; `select_all_permissions`
      and `select_all_roles` carried the identical defect unregistered and are
      folded in — one mechanism, one commit. The three `BUG(pinned)` test markers
      (users/authorities/public_keys) are deleted; all five module tests are
      strengthened from sorted-set-only to asserting `(created_at, id)`
      tuple-monotonicity plus double-call identical-id-sequence determinism
      (sorted-set content asserts kept; fixture ids are server-generated, so no
      hardcoded sequences).
    - Deliberately NOT in scope: no pagination (the total order is its
      prerequisite; keyset on `(created_at, id)` is the future design), no index
      (operator-scale tables), no service-layer sorting — the service delegators
      are order-transparent by design (`the_stored_list_is_returned_in_query_order`
      pins that, and is why the fix goes in SQL). JWT verification is set-like (no
      `kid`; `decode_with_public_keys` brute-forces every key), so this buys wire
      determinism and pagination groundwork — it is not a verification fix and does
      not close OXA-000012 or OXA-000019.
