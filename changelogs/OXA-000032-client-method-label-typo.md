- [OXA-000032](https://www.pivotaltracker.com/story/show/OXA-000032) - fix client METHOD error labels: `create_invitaion` typo and copy-pasted misnames (register CLI-6)
    - `Client::create_invitation`, `Client::find_invitation` and `Client::list_all_authorities`
      carried copy-pasted `METHOD` constants (`"create_invitaion"`, `"create_invitaion"`,
      `"find_authority_by_strategy"`), so empty-payload errors named the wrong method —
      `find_invitation` blamed `create_invitation`, and `list_all_authorities` blamed the
      unrelated-but-real `find_authority_by_strategy` endpoint. Each label is now the
      wrapper's own method name; the three leg-3 contract tuples were flipped and their
      `BUG(pinned)` markers deleted.
    - **Compat:** `ClientErrorKind::EmptyPayload` is public with a `&'static str` method
      field, so a consumer string-matching the typo'd label (or the rendered `Display`
      sentence) changes behavior — the corrected label is what any such matcher wanted.
    - no other change: route, verb, DTOs, signatures, `RESOURCE` values (invitations stay
      `Resource::User`) and the `was expcected` sentence typo (now register CLI-11) are
      untouched.
