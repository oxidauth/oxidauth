# OXA-000012 — jwks listing panics on one malformed key row, taking down all authenticated traffic

**Original ID:** SEC-12 · **Severity:** P2 · **Type:** bug · **Status:** implemented 2026-09-30 (coder+reviewer approved, zero findings; skip+error-log per header scope, 2 pins flipped + mixed-set test, repo-Err propagation and wire shapes verified unchanged; §2 audit migration out of header scope)
**Review tier:** Tier 4 (moderate — multi-file recipes, pinned-test flips) · reviewed 2026-09-30 · SCHEDULED (owner approved after initial defer — verdict re-affirmed as SCHEDULED; scope: tolerant decode in `list_all_public_keys.rs` — skip + `tracing::error!` malformed rows, healthy keys keep serving; precedent `oxidauth-rs/src/client/mod.rs:371-375`; flip 2 should_panic pins to skip-and-serve + mixed-listing test; no API change, no migration)


## Locations

All paths relative to `src/oxidauth/`.

- **Defect:** `oxidauth-services/src/public_keys/list_all_public_keys.rs:40-44` — the per-row mapping closure calls `BASE64_STANDARD.decode(&pk.public_key).unwrap()` (line 42) and `String::from_utf8(decoded).unwrap()` (line 44) inside `.map()` over every row (`:37-48`).
- **Register drift (confirmed, matches the pattern from earlier items):** the register cites `:160,174`. Those lines are the two `BUG(pinned)` marker comments inside the test module — line 160 in `a_row_whose_public_key_is_not_base64_panics_the_whole_list` (`:157-169`) and line 174 in `valid_base64_that_is_not_utf8_panics_on_the_second_unwrap` (`:171-183`). The product defect is the two `unwrap()`s at lines 42 and 44.
- **Consumer 1 — public REST listing:** `oxidauth-api/src/server/api/v1/public_keys/list_all_public_keys.rs:19-25` (handler), mounted unauthenticated at `oxidauth-api/src/server/api/v1/public_keys/mod.rs:15` (`GET /api/v1/public_keys`). Confirmed public: only `create_public_key.rs:20-21` extracts `ExtractJwt`/`ExtractEntitlements` in that router; the listing handler takes only `State` + `Path`. The hurl suite asserts this (`hurl/tests/public_keys.hurl:36-38`, comment "listing is public", `HTTP/1.1 200` with no `Authorization` header).
- **Consumer 2 — every authenticated request:** `oxidauth-api/src/middleware/permission_extractor.rs:36-44`. `ExtractJwt::from_request_parts` calls `list_all_public_keys` at `:38-41` *before* decoding the bearer token at `:43`. `ExtractEntitlements` (`:53-75`) chains through it. Every protected endpoint in the API runs this extractor.
- **Wiring:** `oxidauth-api/src/provider/services.rs:572-579` stores the single `ListAllPublicKeysUseCase<PgPublicKeyRepository>` as the `ListAllPublicKeysService` both consumers fetch.
- **Write path (the invariant the defect assumes):** `oxidauth-services/src/public_keys/create_public_key.rs:38` — rows are only created via `KeyPair::new()?.base64_encode()`, i.e., always valid base64(PEM). The API never accepts caller-supplied key material.
- **Storage:** `oxidauth-postgres/migrations/20221021180809_create_public_keys.sql` — `public_key BYTEA NOT NULL`, no constraint; `oxidauth-postgres/src/public_keys/select_all_public_keys/select_all_public_keys.sql` — plain `SELECT id, public_key, created_at, updated_at FROM public_keys`, no server-side validation.
- **Repo-layer contrast:** `oxidauth-postgres/src/public_keys/mod.rs:57` (`TryFrom<PgPublicSanitizedKey>`) already uses `String::from_utf8(value.public_key)?` and propagates — the repo layer treats bad bytes as `Err`; the service layer then undoes that discipline with `unwrap()`.

## Problem

The jwks listing use case assumes every `public_keys` row was written by `create_public_key` and therefore contains standard-base64 of UTF-8 PEM. When that assumption breaks for **one** row, the whole listing panics instead of degrading:

1. **Non-base64 stored text** (e.g., a raw PEM seeded via SQL): PEM armor contains `-` and newlines, which are not in the `BASE64_STANDARD` alphabet → `decode()` returns `Err` → `.unwrap()` at line 42 panics.
2. **Base64 of non-UTF-8 bytes** (e.g., base64 of DER SPKI, or arbitrary binary): decode succeeds, `String::from_utf8` at line 44 panics.
3. (A third shape — raw non-UTF-8 bytes in the BYTEA column — is already caught *before* the service, at the repo mapping `oxidauth-postgres/src/public_keys/mod.rs:57`, and surfaces as a normal `Err`. The two `unwrap()`s are strictly the surviving gap.)

Both failure legs are pinned by `#[should_panic]` tests (`oxidauth-services/src/public_keys/list_all_public_keys.rs:157-183`, markers at 160 and 174), so the current suite documents the panic as expected behavior.

## Analysis

**Blast radius is far wider than "the jwks listing".** The service is not only wired to `GET /api/v1/public_keys`; it is the key-fetch inside `ExtractJwt` (`permission_extractor.rs:38-41`). The extractor fetches and maps the key list *before* it ever looks at the token, so with one bad row in `public_keys`:

- `GET /api/v1/public_keys` (unauthenticated) panics for any anonymous visitor.
- **Every request to every protected endpoint panics the moment it carries any `Authorization: Bearer …` header** — no valid token required. The 401 mapping at `:41` never runs; the panic unwinds through the extractor, handler, and hyper connection service.
- The repo has **no** `CatchPanicLayer`/catch-panic middleware (verified: no matches for `catch_panic` anywhere under `src/`; the router in `oxidauth-api/src/server/mod.rs:31-38` adds only `CorsLayer::permissive()`), and no `panic = "abort"` profile setting. Consequence: each poisoned request panics the per-connection tokio task — the process survives, but the connection is dropped/reset mid-flight (killing any sibling keep-alive/pipelined requests on it), no response is ever sent, and the log gets one full panic + backtrace per request. That is a permanent, self-inflicted auth-layer outage plus a free log-flood amplification from anonymous traffic.

**How bad rows appear.** The API cannot produce them (`create_public_key` always base64-wraps server-generated PEM; `create_public_key.rs:38`). Sources are out-of-band: manual DBA `INSERT` of a PEM, restore from a dump taken with different encoding assumptions, dev/bootstrap seeding, and the repo's own test fixture `seed_public_key` (`oxidauth-postgres/src/test_fixtures.rs:195-207`), which inserts arbitrary raw bytes (e.g. `b"jwks-key-one"` — not base64) — harmless at the repo layer today, but it demonstrates that the schema accepts exactly the shapes that panic the service. An attacker who obtains SQL write access (or exploits any future SQLi) can plant **one** 40-byte row and turn the entire authenticated API into connection resets — privilege escalation of a data bug into a service-killing DoS with no API access needed.

**Precedent already in the codebase for doing this right:**
- `?`-propagation of decode errors is the established style: `oxidauth-services/src/auth/register.rs:112` and `oxidauth-services/src/auth/authenticate.rs:139` both do `BASE64_STANDARD.decode(...)?`.
- Per-row *tolerant* handling is already implemented **on the client side of this very wire format**: `oxidauth-rs/src/client/mod.rs:371-375` iterates served keys and `match … Err(_) => continue` skips rows it can't decode. The Rust SDK skips malformed keys; the server panics on them.

**Why plain `?`-propagation is the wrong fix here.** In the `ExtractJwt` path the error maps to `401 UNAUTHORIZED` (`permission_extractor.rs:41`) and in the REST handler to `400` (`list_all_public_keys.rs:42`) — so always-`Err` merely converts a panic-outage into a 401-outage for all authenticated traffic; availability stays broken. Skipping the malformed row is safe on security grounds: a row that isn't base64(PEM) cannot verify any signature anyway (`Jwt::decode_with_public_keys`, `oxidauth-kernel/src/jwt/mod.rs:68-81`, just fails through such entries), so dropping it loses no authentication capability that existed before — while every *valid* key keeps working.

**Edge cases:**
- Empty table / all-rows-bad after skipping → extractor sees an empty key list → `Jwt::decode_with_public_keys` errors → all protected requests correctly return 401 (no panic; the separate empty-set panic noted under SEC-11 lives in `decode`/`decode_with_default_jwk_set`, a different call path — the extractor uses `decode_with_public_keys`).
- Bare base64 body of a PEM (no armor, e.g., copied key material) decodes fine as base64 but yields DER bytes → caught by leg 2, skipped by the fix.
- Empty-string `public_key` → decodes to empty vec → empty `public_key` string on the wire; harmless (verification fails through it) but worth flagging in the same warn path.

**Related register items (cross-refs, not duplicates):** SEC-11 (adjacent DoS in `Jwt::decode` empty JWK-set panic + audience bug — same extractor path, different defect), DATA-4 (`insert_public_key` commits before its response-side `String::from_utf8`, same table, insert side), DATA-13 (no `ORDER BY` in this very SQL — the pinned `ORDER BY` marker in `oxidauth-postgres/src/public_keys/select_all_public_keys/mod.rs` tests belongs to that item, not this one).

## Impact

- **Availability (P2 as registered; case for P1 if you assume any external actor can write SQL):** one malformed row = unauthenticated panic on `GET /api/v1/public_keys` **and** connection-reset/panic on *every* bearer-token request across the API. Under a real deployment this is a total authenticated-API outage with only a DB write as the trigger, and a log flood per poisoned request.
- **Observability:** the failure mode looks like random 5xx/connection drops across unrelated endpoints, not like a key-management bug — slow to diagnose.
- **Who is affected:** every operator who seeds or restores `public_keys` out-of-band (a documented pattern given `test_fixtures::seed_public_key` accepts arbitrary bytes); every API client (including the `oxidauth-rs` SDK, whose *own* key handling already tolerates malformed rows); downstream verifiers relying on the jwks endpoint.
- **No confidentiality/integrity impact:** no key material beyond what's already public (the endpoint serves sanitized rows without `private_key`) is leaked; the defect is availability + fail-open-to-panic behavior.

## Proposed resolution

**1. Per-row error handling — skip + warn in the use case (recommended).** Replace the `unwrap()`s at `oxidauth-services/src/public_keys/list_all_public_keys.rs:37-48` with tolerant per-row mapping, mirroring the SDK's precedent (`oxidauth-rs/src/client/mod.rs:371-375`):

```rust
let mut keys = Vec::with_capacity(public_keys.len());
for mut pk in public_keys {
    let decoded = match BASE64_STANDARD.decode(&pk.public_key) {
        Ok(decoded) => decoded,
        Err(err) => {
            tracing::warn!(key_id = %pk.id, err = %err,
                "public_keys row is not standard base64; excluding from jwks listing");
            continue;
        },
    };

    match String::from_utf8(decoded) {
        Ok(pem) => {
            pk.public_key = pem;
            keys.push(pk);
        },
        Err(err) => {
            tracing::warn!(key_id = %pk.id, err = %err,
                "public_keys row decodes to non-UTF8 bytes; excluding from jwks listing");
        },
    }
}

Ok(keys)
```

Rationale for skip+warn over the alternatives:
- *vs. fail-request-with-domain-error (`?` + a domain error type):* keeps all valid keys serving; a strict variant (e.g., `ListAllPublicKeys.strict` for an admin-only audit route returning a per-row error report with ids) can be added later without changing this path's contract. The current handler error shape is `Response::bad_request().error(err.into_error())` (`list_all_public_keys.rs:42`) and the extractor maps any `Err` to a blanket 401 — neither conveys "which row is corrupt," so a fail-the-request design would degrade both consumers while telling operators less than a warn log.
- *vs. SQL-side filtering:* Postgres `decode(x,'base64')` raises on invalid input rather than returning NULL, so row-level base64 validation in SQL requires a `CREATE FUNCTION … STRICT IMMUTABLE` wrapper with an `EXCEPTION` block — pushing a Rust-format invariant into a plpgsql duplicate implementation. Wrong layer; keep SQL validation out of the hot path and handle it in the use case plus a migration audit (below).
- Keep the `tracing::instrument` on the method (`:27`) — the warns carry `key_id` so operators can find the exact row.

**2. Data-hygiene migration to detect existing bad rows.** Add a migration (follow the existing naming, e.g. `oxidauth-postgres/migrations/<ts>_audit_public_keys_key_material.sql`) that *reports*, never auto-deletes (signing key material warrants human judgment — delete the row vs. re-encode it):

```sql
-- rows whose bytes are not valid standard base64 (charset + length legs;
-- Postgres has no try_decode, so guard with the regex first)
CREATE OR REPLACE FUNCTION public_keys_audit_bad_rows()
RETURNS TABLE (id UUID, reason TEXT) AS $$
DECLARE r RECORD;
BEGIN
  FOR r IN SELECT id, public_key FROM public_keys LOOP
    IF r.public_key::text !~ '^[A-Za-z0-9+/\r\n]+=*$'
       OR length(replace(replace(r.public_key::text, E'\r', ''), E'\n', '')) % 4 <> 0 THEN
      id := r.id; reason := 'not standard base64'; RETURN NEXT;
    ELSE
      BEGIN  -- convert_from/decode RAISE on bad input, so catch per row
        IF convert_from(decode(r.public_key::text, 'base64'), 'UTF8') IS NULL THEN
          id := r.id; reason := 'decoded payload is not UTF-8'; RETURN NEXT;
        END IF;
      EXCEPTION WHEN OTHERS THEN
        id := r.id; reason := 'decoded payload is not UTF-8'; RETURN NEXT;
      END;
    END IF;
  END LOOP;
END;
$$ LANGUAGE plpgsql;
SELECT * FROM public_keys_audit_bad_rows();
```

(the regex leg ensures `decode()`/`convert_from()` — which *raise* rather than return NULL on invalid input in Postgres — only ever run inside the per-row `EXCEPTION` block, so the audit can never abort the migration; verify exact regex semantics against the target PG version at implementation time). Pair it with an ops runbook: any reported row is either deleted via `DELETE /api/v1/public_keys/{id}` or re-seeded as `base64(PEM)`. Optionally (only after the audit reports zero rows): `ALTER TABLE public_keys ADD CONSTRAINT public_keys_b64 CHECK (public_key::text ~ '^[A-Za-z0-9+/]+={0,2}$')` to block future raw-PEM seeds at the door — the constraint cannot express the UTF-8-PEM leg, and it will fail the migration if any legacy bad row remains, hence "optional, after cleanup."

**3. Pin flips** (all in `oxidauth-services/src/public_keys/list_all_public_keys.rs`; these are the only `BUG(pinned)` markers for this item — the `ORDER BY` pin in `oxidauth-postgres/src/public_keys/select_all_public_keys/mod.rs` belongs to DATA-13):
- `a_row_whose_public_key_is_not_base64_panics_the_whole_list` (`:157-169`): drop `#[should_panic]` (`:158`), delete the `BUG(pinned)` comment (`:160-163`), rename to e.g. `a_row_whose_public_key_is_not_base64_is_skipped`, and assert `Ok(list)` with the raw-PEM row excluded and the call-count on the mock unchanged.
- `valid_base64_that_is_not_utf8_panics_on_the_second_unwrap` (`:171-183`): drop `#[should_panic]` (`:172`), delete the marker comment (`:174-176`), flip to asserting the `BASE64_STANDARD.encode([0xff, 0xfe, 0x00])` row is excluded from an otherwise-successful listing.
- Add one new test: **mixed set survives** — `[valid_base64(PEM), raw PEM, base64(0xff,0xfe,0x00)]` → `Ok` containing exactly the one healthy key. This is the availability invariant `ExtractJwt` depends on, and neither flipped test covers it alone.

**4. Compat/migration concerns.** No API contract change: wire format stays raw PEM (the `oxidauth-rs` client's `auth()` path consumes exactly this, `oxidauth-rs/src/client/mod.rs:708-717`), success responses for clean tables are byte-identical, and the 400 error shape is untouched. Behavior change is only for corrupted databases: panic → healthy subset. The audit migration is `SELECT`-only + a function definition, so it is safe to replay; the optional CHECK constraint is the only destructive-if-mis-sequenced step and must ship in a separate migration after cleanup. The `oxidauth-rs` SDK needs no change (its `refresh()` double-decode mismatch is tracked separately as a client-contract item, section 3 of the register).

## Verification

- `cargo test -p oxidauth-services public_keys::list_all_public_keys` — the two flipped tests + new mixed-set test pass; the old pins would fail the new assertions (regression proof).
- `cargo test -p oxidauth-api permission_extractor` — extractor still compiles/returns 401-on-error semantics (`permission_extractor.rs` tests) with the use case now infallible for malformed rows.
- `cargo test -p oxidauth-postgres public_keys` (requires Postgres per the `#[sqlx::test(migrator = "crate::MIGRATOR")]` convention) — new migration applies on a clean DB; add a repo test that seeds one `seed_public_key(&pool, b"-----BEGIN PUBLIC KEY-----\nraw\n-----END PUBLIC KEY-----\n")` row alongside a valid one and asserts the audit function reports exactly the bad id (no false positives).
- `hurl --test src/oxidauth/hurl/tests/public_keys.hurl` against a running server — `GET /api/v1/public_keys` still `200` with the bootstrap key present (public listing contract preserved).
- End-to-end repro (the acceptance scenario): against a live server, `psql -c "INSERT INTO public_keys (public_key, private_key) VALUES ('-----BEGIN PUBLIC KEY-----
raw
-----END PUBLIC KEY-----', 'x')"` then (a) `curl GET /api/v1/public_keys` — before: connection closed / panic in server log; after: `200` listing with the bad row absent and one `tracing::warn!` naming its id; (b) `curl` any protected endpoint with an arbitrary `Authorization: Bearer garbage` — before: connection reset + panic log; after: clean `401`.
- Confirm absence of the class: `grep -rn 'BUG(pinned)' src/oxidauth/oxidauth-services/src/public_keys/` returns nothing after the flips.
