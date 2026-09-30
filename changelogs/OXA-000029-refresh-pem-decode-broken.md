- [OXA-000029](https://www.pivotaltracker.com/story/show/OXA-000029) - SDK re-auth restored: `refresh()` now validates its jwt against the raw-PEM wire format `GET /public_keys` actually serves
    - **New kernel helper `Jwt::decode_with_flexible_public_keys`**
      (`oxidauth-kernel/src/jwt/mod.rs`, adjacent to `decode_with_public_keys`):
      for each served key it tries the real wire format first (raw PEM, the
      format `ListAllPublicKeysUseCase` base64-decodes the storage rows into),
      then a base64-decoded PEM leg as version-skew insurance (the storage /
      create / find-by-id encoding, or double-encoded rows surviving
      OXA-000012's filter). Keys that verify nothing are skipped — signature
      validation still gates every acceptance — and an unmatched walk ends with
      the same `"no valid public key found"` sentinel. `decode_with_public_keys`
      and all other decode paths are unchanged; `auth()` keeps calling it.
    - **`Client::refresh()` hand-rolled base64 decode loop deleted**
      (`oxidauth-rs/src/client/mod.rs`): the per-key `BASE64_STANDARD.decode`
      loop silently skipped every raw-PEM key (PEM armor is outside the base64
      alphabet), so against a live server `refresh()` could never validate its
      jwt — every expiry-driven `get_jwt`/`request` died with
      `Other("failed to validate jwt")` after the server had already rotated
      (and burned) the refresh token. One `decode_with_flexible_public_keys`
      call now mirrors `auth()`'s shape, keeping the
      `Other("failed to validate jwt")` error shape and the wildcard
      `Other("")` arm untouched; the crate-level `base64::*` import retired with
      the loop. No public SDK API change.
    - **Test pins flipped to the fixed contract**:
      `refresh_is_broken_against_the_real_raw_pem_wire_format` ->
      `refresh_validates_the_real_raw_pem_wire_format` (raw-PEM-only keyset,
      `refresh()` -> `Ok(true)`, fresh `exp` visible via `get_jwt_decoded`; the
      raw accessor stays CLI-2's territory), and `full_keyset` now mounts a
      single raw-PEM entry so every state-machine test drives the real wire
      format. New fixtures: base64(PEM)-only keyset validates on the fallback
      leg (`refresh_falls_back_to_the_base64_storage_format`), foreign keys in
      either encoding still `Err("failed to validate jwt")`
      (`refresh_rejects_foreign_keys_in_either_encoding`). The
      `auth_rejects_base64_storage_format_the_api_never_serves` anti-pin and the
      CLI-2/CLI-4 pins are untouched and green. Kernel gained a direct unit test
      for the helper (raw leg, b64 leg, garbage skipped, foreign/b64-non-PEM/
      empty set yield the sentinel error, no panic).
