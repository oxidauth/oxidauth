- [OXA-000012](https://www.pivotaltracker.com/story/show/OXA-000012) - jwks listing skips malformed key rows instead of panicking
    - **User-visible: one corrupt `public_keys` row no longer takes down all
      authenticated traffic.** `list_all_public_keys` decoded every row with
      `BASE64_STANDARD.decode(..).unwrap()` / `String::from_utf8(..).unwrap()`,
      so a single out-of-band-written row (raw PEM, base64 of non-UTF-8 bytes)
      panicked the use case. Since `ExtractJwt` runs this listing on **every**
      bearer-token request — before even looking at the token — one bad row
      turned every protected endpoint into a connection reset plus a panic
      backtrace per request, and `GET /api/v1/public_keys` (public) panicked
      for any anonymous visitor.
    - The mapping is now tolerant per row, mirroring the `oxidauth-rs` client's
      existing skip-on-undecodable behavior: a row that is not standard base64,
      or whose decoded bytes are not UTF-8, is excluded from the listing and
      logged via `tracing::error!` carrying `key_id` and the decode error, so
      operators can find the exact row. Healthy keys keep serving; the success
      response for a clean table is byte-identical. Skipping is safe: such a
      row can verify no signature anyway (`decode_with_public_keys` fails
      through it), so no authentication capability is lost. With all rows bad
      the listing is simply empty and protected routes return their normal
      401.
    - No API change: wire format stays raw PEM, handler 400 shape and
      extractor 401 mapping untouched; repository errors still propagate
      unchanged. No migration ships here — the data-hygiene audit function from
      the ticket proposal is outside the approved scope (operator guidance:
      delete or re-seed any row named by the new error logs as
      `base64(PEM)`).
    - Two `#[should_panic]` pins that documented the panic as expected
      behavior (`a_row_whose_public_key_is_not_base64_panics_the_whole_list`,
      `valid_base64_that_is_not_utf8_panics_on_the_second_unwrap`) are flipped
      to assert skip-and-serve semantics, both `BUG(pinned)` markers deleted,
      and a new mixed-set test pins the availability invariant `ExtractJwt`
      depends on: `[base64(PEM), raw PEM, base64(0xFF 0xFE 0x00)]` yields
      `Ok` with exactly the one healthy key and a single repository call.
