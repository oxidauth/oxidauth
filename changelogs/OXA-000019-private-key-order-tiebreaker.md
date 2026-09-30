- [OXA-000019](https://www.pivotaltracker.com/story/show/OXA-000019) - the JWT signing key is now picked deterministically: `ORDER BY created_at DESC, id DESC`, and key inserts stamp `statement_timestamp()`
    - **`select_most_recent_private_key.sql` was only a partial order.** `created_at`
      carries no uniqueness constraint, so every tie resolved by whatever row order the
      plan happened to produce (seq-scan short-circuit vs top-N heapsort, `VACUUM` and
      page reclamation, `ANALYZE` stats, and *which replica* answers on
      `READ_DATABASE_URL`): two calls over untouched data could hand the signing service
      two different keys. The `ORDER BY` is now a total order on its own. Postgres
      compares `uuid` byte-wise and Rust `Uuid: Ord` uses the same byte order, so the
      winner is one row on every replica and is expressible in test asserts.
      `created_at` is `NOT NULL` (no `NULLS LAST` needed), the empty-table
      `fetch_one` → `RowNotFound` contract is untouched, and `updated_at` deliberately
      never enters the ordering — it is mutable by unrelated UPDATEs, and the
      newest-wins test keeps pinning that it must lose.
    - **The write side stopped stamping whole batches with one timestamp.**
      `insert_public_key.sql` used `NOW()`, which is the *transaction* start time —
      constant for the whole transaction — so a rotation batch (`BEGIN; INSERT …;
      INSERT …; COMMIT;`, `psql -1` seeds, a single-transaction restore, or a future
      bulk-rotation service following the `insert_totp_secrets` `write_pool().begin()`
      precedent) tied every row by construction, leaving zero ordering information in
      `created_at`. Both timestamp columns now use `statement_timestamp()`: per-statement,
      so within a batch the semantically newest key wins by `created_at` alone. The
      app's own path (`POST /api/v1/public_keys` → one auto-committed INSERT per request,
      plus bootstrap's single-key creation) is behaviorally unchanged; `RETURNING *` still
      yields `timestamptz`, so no DTO/route/response-shape change and no migration.
    - **Consumer-visible delta:** the only shift is *which* row wins a previously
      ambiguous tie — arbitrary before, greatest `id` now for out-of-band rows (seeds that
      take the `CURRENT_TIMESTAMP` column default still tie and fall back to that
      deterministic tie-break), and true insertion order going forward for anything
      written through the app. Ties already in production data are not repaired, only
      made deterministic; tokens still carry no `kid`, so verification keeps accepting
      every served key.
    - Tests (`private_keys/select_most_recent_private_key/mod.rs`):
      `it_should_pin_the_same_created_at_tie_as_unspecified` →
      `it_should_deterministically_break_same_created_at_ties` — the `BUG(pinned)` block
      is deleted (zero markers remain under `src/private_keys/`) and the disjunctive
      "one of the two" assert became
      `assert_eq!(winner.id, std::cmp::max(first, second), "ties must break to the greater id")`
      plus `assert_ne!` against `std::cmp::min(first, second)`. New
      `it_should_pick_the_last_key_of_a_same_transaction_batch` runs the real
      `insert_public_key.sql` twice inside one `pool.begin()` (helper takes
      `&mut PgConnection`, the `insert_totp_secret_query` idiom) with ids chosen so the
      *first* insert owns the greater id, then asserts the *second* key signs — a
      deterministic failure before `statement_timestamp()`, not a coin flip.
      `it_should_error_when_no_keys_exist` and
      `it_should_pick_the_newest_created_at_regardless_of_insert_order` are invariants,
      not pins, and stay untouched.
    - Deliberately NOT in scope: the report-only `created_at` tie audit migration was
      dropped at review, so existing ties are surfaced by hand instead —
      `SELECT created_at, count(*) FROM public_keys GROUP BY 1 HAVING count(*) > 1`;
      which tied key should sign stays an operator decision (delete the unintended row
      via `DELETE /api/v1/public_keys/{id}`, or desynchronize its `created_at`). Also
      out: the missing `kid` header (deferred with the OXA-000011 family — it is the
      wire-format change that removes the blind post-rotation cleanup hazard), no
      `seq GENERATED ALWAYS AS IDENTITY` column, no index (operator-scale table; add
      `CREATE INDEX ON public_keys (created_at DESC, id DESC)` only if it ever grows),
      jwks/listing order (OXA-000026), and duplicate key material (OXA-000017).
