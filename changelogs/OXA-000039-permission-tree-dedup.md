- [OXA-000039](https://www.pivotaltracker.com/story/show/OXA-000039) - permission-tree flatten dedupes entitlements: a permission reachable through N grant paths now contributes ONE entry to `PermissionsResponse.permissions` and to every minted JWT
    - **Consumer-visible delta: claim/array bytes shrink only when duplicates
      existed; nothing reorders.** The flat entitlement list now carries one
      entry per *distinct* permission instead of one per grant path, so
      `PermissionsResponse.permissions` and the `txt …`/`gz …` claim on every
      token from `authenticate`, `register`/`authenticate_or_register`,
      `exchange_refresh_token` and `totp/validate` shrink by
      `Σ(len(perm)+1)` raw bytes (≈×4⁄3 base64url in the encoded JWT) per
      removed duplicate — per mint *and* per refresh. A permission granted
      both directly and through a role, or reached through a diamond of role
      paths (where the flat list previously grew with path count, doubling per
      diamond layer), now appears exactly once. A tree with no duplicate
      grant paths produces byte-identical claims before and after.
    - **Ordering resolved as first-occurrence-wins, stable:** the flatten
      keeps its nested-before-direct, declaration-order walk and only
      *removes repeats* — it never sorts and never lets a later occurrence
      move an entry. The first path to reach permission P contributes P's
      single entry. Existing order pins
      (`user_tree_flattens_nested_roles_before_direct_grants`,
      `role_tree_flattens_nested_grants`, the exact array in
      `permissions_response_serde_shape`) stay green and *are* the
      stable-order regression proof.
    - **Implementation (kernel-only):** `UserNode::permissions()` /
      `RoleNode::permissions()` in `oxidauth-kernel/src/auth/tree/mod.rs`
      now thread a shared `HashSet<Uuid>` seen-set through one recursive
      `collect` per node type, keyed on `permission.id`. Id-keying is exactly
      equivalent to string-keying while `permissions` carries
      `CONSTRAINT unique_grant_parts UNIQUE(realm, resource, action)`
      (migration 20221019222410), which makes id ↔ `realm:resource:action`
      1:1 — the invariant is stated in the code for future constraint
      changes. All callers (`oxidauth-postgres` tree queries, the four JWT
      mint paths, `oxidauth-rs` `ExtractEntitlements`) inherit the fix
      untouched; no service or API code changed.
    - **The tree JSON is unchanged:** `PermissionsResponse.tree` keeps one
      `RoleNode` per grant path — the per-path grant DTOs remain the audit
      "why" view. Only the sibling flat array, and transitively the JWT
      claim, dedupe. The graph walk (`role_permissions_as_tree`) and the SQL
      are untouched, so this composes in either landing order with
      OXA-000001's path-local cycle guard (a separate termination item; its
      pinned cycle test still pins non-termination and was not flipped here).
    - **Authorization semantics unchanged; already-issued tokens keep
      working:** `oxidauth_permission::validate` is a short-circuiting
      any-match, so allow/deny verdicts are identical over a multiset and its
      deduplicated set. Tokens minted before this change keep their duplicate
      claims until expiry/refresh and keep verifying; no rollout ordering, no
      schema change, no wire-format change. Decode-side dedup is deliberately
      NOT added (multiset-safe `validate` makes it unnecessary).
    - **Three pins flipped to the audit A3 contract:** kernel
      `permissions_are_not_deduplicated_across_nodes` →
      `permissions_are_deduplicated_first_occurrence_wins` (2 entries → 1,
      `BUG(pinned)` deleted) plus a new
      `diamond_grants_collapse_to_first_occurrence_order` pinning set-size =
      distinct permissions and first-occurrence order for a same-leaf diamond
      (fails pre-fix with the 4-entry multiset); postgres user-tree test
      `app:thing:read` 3× → sorted 3 entries `["app:other:delete",
      "app:thing:read", "app:thing:write"]` (5 → 3, `BUG(pinned)` deleted);
      postgres role-tree test 3 → 2 entries. All structural tree assertions
      and the empty-tree / unknown-user tests are untouched.
