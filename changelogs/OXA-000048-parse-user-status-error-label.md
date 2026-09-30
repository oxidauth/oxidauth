- [OXA-000048](https://www.pivotaltracker.com/story/show/OXA-000048) - fix ParseUserStatusErr error label (register SRV-12)
    - `ParseUserStatusErr`'s `Display` copy-pasted the kind error's text and
      reported `failed to parse user_kind, unknown: <input>` for every rejected
      *status* token; a poisoned `users.status` row (no CHECK constraint on the
      column) misdirected operators to the `kind` column in every `400`
      envelope body and log line. The message now reads
      `failed to parse user_status, unknown: <input>`.
    - **External alert rules grep-ing `failed to parse user_kind` for status
      parse failures must repoint at `failed to parse user_status`.** No
      in-repo client, fixture, or runbook asserted the old string; the
      `user_kind` message itself is unchanged.
    - no API/DTO/signature change: same words as the `user_kind` sibling with
      one token swapped; `UserStatus::FromStr` now matches the
      `ENABLED`/`INVITED`/`DISABLED` consts instead of raw literals
      (behavior-identical, mirrors `UserKind::FromStr`).
