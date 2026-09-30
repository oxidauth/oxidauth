- [OXA-000040](https://www.pivotaltracker.com/story/show/OXA-000040) - `Entitlements::Gz` is now symmetric: the variant holds the wire payload (base64-of-gzip) on both the mint side and the decoded side, so decoded `Gz` claims re-serialize/re-sign into tokens every verifier accepts
    - **Zero wire drift**: the claim format (`"txt …"` / `"gz <base64-of-gzip>"`) is
      unchanged and issued-token bytes are byte-identical before/after — tokens
      minted pre-fix decode post-fix and vice versa. No migration, no version
      gating, no DB change.
    - **`Entitlements::decode` keeps validation, stores the payload verbatim**
      (`oxidauth-kernel/src/jwt/mod.rs`): the `gz` arm still runs
      `BASE64_STANDARD.decode` + `GzDecoder::read_to_string` (base64, gzip and
      UTF-8 checked) so malformed input keeps failing at parse time with the
      exact same error surface (`entitlements_decode_rejects_malformed_input`
      stays green), but the decompressed text is only validated, never stored —
      `Gz(..)` carries the same bytes `encode` put on the wire.
    - **`as_vec` inflates `Gz` on demand**: base64 → gunzip → split on `' '`,
      `Txt` branch unchanged. The `Option` now carries real meaning — a payload
      that cannot be inflated yields `None` instead of one garbage entry — and
      both extractors (`oxidauth-api` `ExtractEntitlements`, `oxidauth-rs`
      `ExtractEntitlements`) keep their fail-closed `unwrap_or_default()` shape.
    - **`Serialize`/`Deserialize` unchanged** — with the payload symmetric,
      `Serialize` is correct on every value constructible through the API,
      `Deserialize ∘ Serialize` is identity, mint-side ≡ decoded-side under the
      derived `Eq`, and re-issuing a decoded `Jwt` without rebuilding its
      entitlements (introspection, re-signing after key rotation, refresh-copy)
      no longer mints valid-signature/undecodable-claims tokens. Consumer-visible
      kernel-surface delta: decoded claims' `Gz(..)` payload is now the base64
      wire payload instead of decompressed plaintext — reading permissions via
      `as_vec()` (all current consumers) is unaffected.
    - **Test pins flipped to the stronger identity contract**:
      `gz_entitlements_survive_jwt_round_trip` now asserts
      `decoded.entitlements == claims.entitlements` (its `BUG(pinned)` marker
      deleted), `gz_entitlements_wire_format_round_trips_via_from_str` asserts
      `parsed == gz`; mint-side base64 pins and all `as_vec`/`Txt` assertions
      stay. New regression test `decoded_gz_claims_re_sign_to_redecodable_tokens`
      (decode → re-encode without rebuilding entitlements → decode → `as_vec`,
      plus decoded-claim serde equality with the original claim string) fails
      pre-fix at the second decode (`Invalid symbol 58` — the `gz <plaintext>`
      corruption) and passes after. The end-to-end `oxidauth-api`
      `extract_entitlements_yields_gz_permissions` contract test is untouched
      and green.
    - **Setting docs**: `AuthoritySettings.entitlements_encoding` documents the
      size caveat — base64 expands ~4/3 before gzip nets out, so `gz` claims are
      *larger* than `txt` for typical permission lists and only pay off for very
      large entitlement sets.
