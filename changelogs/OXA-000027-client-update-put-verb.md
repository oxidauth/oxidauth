- [OXA-000027](https://www.pivotaltracker.com/story/show/OXA-000027) - fix `update_user`/`update_role` SDK verb drift: POST → PUT (register CLI-1)
    - `Client::update_user` and `Client::update_role` sent **POST** to
      `/api/v1/users/{id}` and `/api/v1/roles/{id}`; the api mounts those paths
      only as GET/PUT/DELETE, so every call failed with 405 against a live
      server — and because `Client::request` deserializes the reply
      unconditionally, callers saw
      `ClientErrorKind::Other("failed to deserialize response")` with the 405
      and its `Allow` header swallowed. Both wrappers now send `.put(...)`;
      route, payload, bearer header and DTOs are byte-identical, the same shape
      `update_authority` / `update_user_authority` already used successfully.
    - **Test-only break for consumers who mirrored our contract tests:** the two
      wiremock pins (`update_user_route_contract`, `update_role_route_contract`)
      passed `"POST"` as the harness verb and carried `BUG(pinned)` markers;
      both now pass `"PUT"` and the markers are deleted. Any consumer test that
      copied our `"POST"` contract string must repoint its
      `Mock::given(method("POST"))` mount and its request assertion at `PUT`.
    - Repaired SDK updates now newly reach the handler — and with it
      OXA-000015, the `update_user` null-overwrite where omitted fields are
      written as NULL (already live today via raw HTTP/hurl, not created by
      this change).
    - no public API/DTO/signature change: `UpdateUserTrait`/`UpdateRoleTrait`,
      `UpdateUserBodyReq`/`UpdateRoleReq`, `ClientMock` and re-exports are
      untouched; nothing server-side, hurl or schema moved.
