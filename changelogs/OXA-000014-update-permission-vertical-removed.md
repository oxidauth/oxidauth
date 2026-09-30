- [OXA-000014](https://www.pivotaltracker.com/story/show/OXA-000014) - remove the dead `update_permission` vertical instead of fixing it (register DATA-1)
    - `oxidauth-postgres/src/permissions/update_permission/` (whole module:
      `mod.rs` with its `BUG(pinned)` failing test + `update_permission.sql`)
      and `oxidauth-repository/src/permissions/update_permission.rs` are
      deleted, together with the two `pub mod update_permission;`
      declarations. This is a dead-code removal, not a behavior fix: owner
      verdict is that permissions are not updateable by design, so the
      corrected implementation the ticket originally sketched is NOT built.
    - **Public trait deleted:** `UpdatePermission` (crate
      `oxidauth-repository`), along with `UpdatePermissionParams` and
      `UpdatePermissionError`. It had zero in-repo callers — no kernel
      request, service use case, HTTP DTO, api route, SDK wrapper or hurl
      test ever reached it (grep-verified) — and its SQL targeted the wrong
      table (`UPDATE authorities SET realm/resource/action`), so the one
      code path could never succeed anyway.
    - no schema or migration change; no route/DTO/SDK surface existed to
      remove. Permissions remain create/find-by-parts/list/delete only.
    - Residual operator note (unchanged by this removal): renaming a
      permission stays DELETE + re-POST, and `ON DELETE CASCADE` on
      `role_permission_grants`/`user_permission_grants` silently strips the
      old triple from every holder. A rename-preserving update path, if ever
      wanted, is a fresh product decision (ticket Step 4B).
