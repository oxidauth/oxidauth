- [OXA-000028](https://www.pivotaltracker.com/story/show/OXA-000028) - fix stale `get_jwt()` after refresh: `Client::refresh` now writes `state.raw_jwt` (register CLI-2)
    - `Client::refresh` updated `state.jwt`, `state.refresh_token` and the
      bearer client in its validated success arm but never wrote the new
      token into `state.raw_jwt`, so `get_jwt()` — the SDK's raw-string
      accessor — kept handing out the **expired** token string after every
      successful (automatic or manual) refresh, while `get_jwt_decoded()`
      and the `Authorization` header on the wire carried the fresh one.
      The write now sits in the same validated (`Some(jwt)`) arm as the
      other state writes, after key verification — mirroring `auth()`, so
      a token that fails decode is still never stored raw and
      `NoJwtFound`-on-rejected-auth is preserved.
    - **Test-only break for consumers who mirrored our contract tests:**
      the two stale pins (`expired_jwt_triggers_refresh_via_get_jwt`
      carried a `BUG(pinned)` marker,
      `concurrent_get_jwt_after_expiry_refreshes_exactly_once` an
      unlabeled comment) asserted `get_jwt()` returns the stale string;
      both now assert the fresh token and the pins are deleted. Any
      consumer test that copied our stale-string assertion must repoint
      it at the fresh token.
    - New regression test `get_jwt_after_refresh_matches_the_bearer_on_the_wire`
      pins the restored invariant: after an expiry-driven refresh,
      `get_jwt()` equals the fresh response string and the
      `Authorization` header actually sent on a subsequent `request()`
      equals `Bearer <get_jwt()>` (wiremock received-request
      inspection).
    - no public API/DTO/signature change: `get_jwt()`, `refresh()` and
      `ClientTrait` are untouched; only the returned value changes, which
      is the bug fix, and `get_jwt()` now agrees with `get_jwt_decoded()`.
      The refresh-vs-raw-PEM wire-format failure (CLI-3 / OXA-000029,
      separate changeset) and the empty refresh-error arm (CLI-4,
      wontfix) are unaffected — their `BUG(pinned)` markers stay.
