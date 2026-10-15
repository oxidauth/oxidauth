-- A refresh token minted by a TOTP-enabled authority is only half a session: the
-- holder still owes the emailed code (`oxidauth-services/src/auth/authenticate.rs`
-- answers the first leg with a JWT entitled to nothing but
-- `oxidauth:totp_code:validate`). The exchange path could not tell such a token
-- from a validated one — it signed the caller's full permission tree either way —
-- so refreshing the phase-1 token completed the login without ever presenting the
-- second factor. `false` means "issued, second factor still owed".
--
-- Existing rows are `true`: they were already exchangeable for a full JWT under
-- the previous contract, and re-challenging sessions that predate the flag would
-- strand every live login at deploy time.
ALTER TABLE refresh_tokens
ADD COLUMN totp_validated BOOLEAN NOT NULL DEFAULT TRUE;
