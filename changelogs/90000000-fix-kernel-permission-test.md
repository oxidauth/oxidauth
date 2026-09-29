- [90000000](https://www.pivotaltracker.com/story/show/90000000) - fix wildcard permission matching
    - `validate_single` passed its arguments to `compare` in the wrong order:
      the parsed *granted* permission went in as the concrete set and the
      required *challenge* went in as the glob pattern
    - `compare` only expands `*` / `**` on its pattern side, and challenges
      reject globs (`PermissionParseErr::WildcardChallenge`), so the glob arms
      were dead for every caller of `validate` / `parse_and_validate`
    - net effect: a wildcard grant (`**:**:**`, `oxidauth:**:read`, ...) never
      matched a required permission, so `CanService` answered every superuser
      with `CanError::Unauthorized` — the panic behind
      `service::tests::it_should_allow_call_with_good_permissions`
    - swaps the call back to `compare(challenge, &parsed)` (as `df5ecd8` had
      it; `864ef8a` flipped it) and documents the argument convention
    - restores `Password`'s constant `******` debug mask too, which unmasked
      `testing_manual_password_debug_impl` in oxidauth-usecases once the kernel
      failure stopped aborting the run early: `ee31b45` had replaced the mask
      with a per-char one that kept the first character and leaked the password
      length into every tracing span (`5c85849`, the commit that added
      `Password`, masked the whole value)
    - `oxidauth-kernel` only asked tokio for `test-util`, so its own
      `#[tokio::test]`s did not compile outside a workspace build (E0433
      `could not find 'test' in 'tokio'`) and silently ran nowhere but
      `cargo test --workspace`; adds `macros` and `rt` so
      `cargo test -p oxidauth-kernel` works standalone
    - no signature changes in `oxidauth-permission` or `oxidauth-kernel`
