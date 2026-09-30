# OXA-000035 — Forgot-password wrapper carries a phantom generic that forces a turbofish

**Original ID:** CLI-9 · **Severity:** P3 · **Type:** bug · **Status:** implemented (2026-09-30)
**Review tier:** Tier 3 (small precise recipe, multi-file) — ranked #22 of 27 · reviewed 2026-09-30 · IMPLEMENTED 2026-09-30 (verified in review pass: both wrappers take load-bearing `params: T` + `params.into()`, pins call bare, turbofish + `BUG(pinned)` marker gone; status line here was stale)


## Locations

- **Register:** `BUGS_AND_NOTES.md:59` — CLI-9, P3: "Generic `T: Into<ForgotPasswordParams>` is unconstrained (no fn argument carries T) → callers must turbofish an irrelevant type," citing `oxidauth-rs/src/client/auth/username_password/forgot_password.rs:45`.
- **Register drift (confirmed):** line 45 of that file is the `// BUG(pinned): the generic ... callers are forced to turbofish an irrelevant type.` comment *inside the test module* (`forgot_password.rs:44-53`) — the pin, not the defect. Same pattern as every other CLI item.
- **Actual defects** (both under full path root `src/oxidauth/`):
  - `oxidauth-rs/src/client/auth/username_password/forgot_password.rs:13-18` — `Client::username_password_forgot_password<T>` declares `<T>` (line 13) but its only argument is the **concrete** `params: ForgotPasswordParams` (line 15); `T` appears nowhere except the where-clause `T: Into<ForgotPasswordParams> + fmt::Debug` (17-18). Body (20-24) forwards the concrete `params` unchanged.
  - `oxidauth-rs/src/client/auth/username_password/update_password.rs:13-18` — exact twin: `username_password_update_password<T>` with concrete `params: UpdatePasswordParams` (15) and dead bound `T: Into<UpdatePasswordParams> + fmt::Debug` (17-18). Its test pins the quirk too: comment at `:43` ("turbofish pin: same unused-generic quirk as forgot_password") + turbofish at `:45`.
- **Pinned call sites:** `forgot_password.rs:49` (`::username_password_forgot_password::<ForgotPasswordParams>(...)`) and `update_password.rs:45` (`::<UpdatePasswordParams>(...)`).
- **Correct template in the same crate:** `oxidauth-rs/src/client/auth/oauth2/redirect.rs:16-23` — `oauth2_redirect<T>(&self, params: T) where T: Into<Oauth2RedirectParams> + fmt::Debug` does `let params = params.into();` (23); its test calls it bare, no turbofish (`redirect.rs:51`).
- **House convention:** every other wrapper binds the generic *in the argument*, e.g. `oxidauth-rs/src/client/users/find_user_by_id.rs:13-15` (`params: T`, `T: Into<FindUserByIdReq> + fmt::Debug + Send`), test call bare at `:77`. A sweep of all `fn …<T>` declarations in `oxidauth-rs/src/client/` (grep `pub async fn .*<T>` plus a multiline sweep for concrete-typed args in generic signatures) finds **exactly two** phantom-generic wrappers — the pair above. `oauth2_redirect` is the third generic-bearing auth wrapper and is healthy.
- **No mock/trait/wasm surface:** these are plain inherent `impl Client` methods (no `#[async_trait]`, no `*Trait`); `client/mock.rs`, `src/wasm/`, and every non-test caller grep for `username_password_forgot_password`/`username_password_update_password` outside the two files are empty (the same-named hits in `oxidauth-api/src/server/api/v1/auth/username_password/*.rs` are tracing span names for the server handlers, not SDK calls).
- **Supporting facts:** `Client::post` requires `Req: Serialize + std::fmt::Debug` (`oxidauth-rs/src/client/mod.rs:516-519`); kernel DTOs `ForgotPasswordParams` (`oxidauth-kernel/src/auth/username_password/forgot_password.rs:10-11`) and `UpdatePasswordParams` (`.../update_password.rs:10-11`) both `#[derive(Debug, Serialize, Deserialize)]`; grep `for ForgotPasswordParams|for UpdatePasswordParams` across `src/` and `xlib/` returns **zero** conversion impls, so the only types satisfying `T: Into<XxxParams>` are the structs themselves (via the reflexive `impl<T> From<T> for T`).

## Problem

Both password-reset wrappers declare a generic `T` that no argument, the return type, or the body ever mentions. Because inference needs `T` to be determined by the call, and nothing in the signature carries it, a call without an explicit type argument fails type inference — the standard E0283 "type annotations needed" [INFERENCE: language semantics; not recompiled per work rules; the repo pin states it outright: the generic is "inferable from nothing but the fn takes no T-typed argument, so callers are forced to turbofish," `forgot_password.rs:45-47`].

So every caller MUST write:

```rust
client.username_password_forgot_password::<ForgotPasswordParams>(
    ForgotPasswordParams { user_id },
).await?;
```

even though the argument type *is already* `ForgotPasswordParams` and the turbofish can legally be **exactly one thing** — since no conversion impls exist, `::<ForgotPasswordParams>` is the sole spelling that satisfies the bound. The type parameter carries zero information, changes zero codegen (the body monomorphizes identically), and exists only to be typed at every call site.

## Analysis

- **Mechanism / origin:** the two files look like copy-paste from the `params: T where T: Into<Params>` wrapper template (`find_user_by_id.rs` style, ~52 wrappers) where the generic is *load-bearing* — it lets callers pass `Uuid`, `&str`, etc., and the wrapper calls `.into()`. Here the author kept the declaration and bound but made the argument concrete and dropped the `.into()` call, converting a flexibility feature into pure call-site tax. `oauth2_redirect.rs` in the same directory retains the correct shape, confirming the drift is per-file, not a design choice.
- **The `fmt::Debug` bound is equally vestigial:** what actually needs `Debug` on these paths is the *concrete* params — `#[tracing::instrument(skip(self))]` (line 12 of each file) records `params`, and `Client::post` bounds `Req: Serialize + fmt::Debug` (`client/mod.rs:518`). Both DTOs derive `Debug`, so those requirements are met without any `T`. The bound on `T` never constrains anything real.
- **API-surface lie:** the where-clause advertises "accepts anything `Into<ForgotPasswordParams>`" while the signature refuses it — readers can't tell whether `user_id`-style conversions are supported without opening the file. The register grades this P3 correctly: pure ergonomics, no runtime behavior delta, but it sits on public API used by exactly the UI flows (password reset) where SDK friction matters.
- **Blast radius is external-only:** zero in-repo call sites besides the two pinned tests; no `ClientMock` plumbing (methods aren't trait methods), no wasm duplicate. All compatibility surface is third-party `oxidauth` (package name per `oxidauth-rs/Cargo.toml:2`, v0.9.0 — `Cargo.toml:3`).
- **Adjacent defects, deliberately excluded:**
  - **OXA-000031 (CLI-5)** owns these same two functions for ignoring `success:false` envelopes and explicitly fenced this quirk out: "Related but distinct defects (do not fold in): the unused-generic turbofish quirk… (`forgot_password.rs:45-47` `BUG(pinned)`, echoed at `update_password.rs:43`)" (OXA-000031 §Analysis) and lists both pins as "Untouched, adjacent pins" (§Proposed resolution). **This ticket owns the `forgot_password.rs:45-47` marker and the `update_password.rs:43` pin that OXA-000031 released.** Likewise OXA-000027's "do NOT touch" list (`OXA-000027:92`, "forgot_password.rs:45 (CLI-9)") is CLI-1 scope hygiene, not a claim on the marker. Whichever ticket merges first, the other's diff must not re-add the pin; no edit collision either way (OXA-000031 changes the `Ok(result)` path, this one the signature and test call syntax).
  - **OXA-000005 (SEC-5)** contemplates deprecating/removing the SDK `username_password_forgot_password` wrapper alongside a gated kernel DTO. If that lands first, the forgot-password half of this ticket shrinks to `update_password` only; if this lands first, that deprecation just applies to the cleaned signature. No ordering constraint.
  - **OXA-000008** (raw-secret logging) replaces the derived `Debug` on `UpdatePasswordParams` with a redacting custom `Debug`. Note the client-side mirror of that leak already exists: `instrument` on `update_password.rs:12` records `params` (password included) into client tracing spans today. The recommended fix below neither worsens nor fixes that — it preserves the exact recording behavior and leaves the redaction to OXA-000008 (any `Debug` that stays `Debug` compiles unchanged here).

## Impact

- **Who:** every external consumer of the `oxidauth` crate using the username_password reset flows (`oxidauth-cli`, `oxidauth-import-export`, in-repo wasm client: greps empty). Each call site pays a mandatory `::<ForgotPasswordParams>`/`::<UpdatePasswordParams>` suffix that conveys nothing and can never vary.
- **Discovery cost:** the failure mode is a compile-time E0283 pointing at the call, which most users misread as "my argument type is wrong" (they just passed a `ForgotPasswordParams`), then cargo-cult the turbofish. The pinned tests demonstrate the confusion is intrinsic — the harness must call it out in a comment.
- **Inconsistency tax:** the rest of the SDK either takes `params: T` (52 wrappers + `oauth2_redirect`) and infers cleanly, or takes no generics at all. These two are the only signatures where "the type of my argument" must be stated twice.

## Proposed resolution

**Step 1 — make `T` load-bearing (recommended), in both files.** Adopt the `oauth2_redirect` shape verbatim:

```rust
// forgot_password.rs (mirror in update_password.rs with Update*)
pub async fn username_password_forgot_password<T>(
    &self,
    params: T,
) -> Result<Response<ForgotPasswordResponse>, BoxedError>
where
    T: Into<ForgotPasswordParams> + fmt::Debug,
{
    let params = params.into();
    let result: Response<ForgotPasswordResponse> =
        self.post("/auth/username_password/forgot_password", params).await?;
    Ok(result)
}
```

Do **not** add the trait wrappers' `+ Send`: these are inherent methods, not `async_trait` (the `Send` there exists for boxed futures); `oauth2_redirect:21` is the precedent.

**Step 2 — flip the pins in the same commit** (see below); call sites drop to bare inference.

**Alternative (simpler, more breaking):** delete `<T>` and the where-clause outright, keeping concrete `params: ForgotPasswordParams`. `T` is provably dead code, so this is behavior-identical — but it breaks *every* existing consumer call site (they must all carry a turbofish today, which stops compiling: "function takes 0 generic arguments"), whereas Step 1 breaks none. Choose only if maintainers prefer minimizing the signature over compatibility.

**Compat analysis (Step 1):**
- Existing turbofish call sites keep compiling: `::<ForgotPasswordParams>(ForgotPasswordParams { .. })` type-checks via reflexive `Into` (`ForgotPasswordParams: Debug` derived, `forgot_password.rs:10`). No other turbofish spelling can exist today (zero conversion impls → the struct is the unique bound-satisfier), so **100% of legal current call sites compile unchanged**.
- The pathological case — turbofish a *foreign* type `::<U>(params)` with `U: Into<Params>` but `U ≠ Params` — compiles under the old signature but not Step 1 (the argument must now be `U`). Unreachable in practice: it requires a conversion impl that doesn't exist in this repo, so a consumer would have had to `impl From<U> for ForgotPasswordParams` themselves (orphan-rule constrained). Not worth a shim; note it in the changelog anyway.
- Wire behavior: `params.into()` is reflexive identity for the struct → byte-identical request body (contract fixture `{"code": "CODE-123"}`, `forgot_password.rs:43`). Tracing still records `params`, now as `T: Debug` — same recorded value for struct callers. Pre-1.0 (v0.9.0): ship as a patch/minor with a changelog line; no release-note warning needed beyond "call sites may drop the turbofish."
- Out of scope, explicitly untouched: the `Ok(result)` envelope pass-through (OXA-000031), the span's recording of the plaintext password (OXA-000008 client-side mirror), any `Send`/trait/mock plumbing.

**Pin flips:**
- `forgot_password.rs:45-47` — delete the `BUG(pinned)` comment; line 49 becomes `.username_password_forgot_password(ForgotPasswordParams { user_id })` (no turbofish). The test now *proves inference works*: if the signature regresses to a phantom generic, this call stops compiling.
- `update_password.rs:43` — delete the "turbofish pin" comment; drop `::<UpdatePasswordParams>` at line 45. (No literal `BUG(pinned)` tag there, but it pins the same defect and must flip with it.)
- Register/ledger hygiene: mark CLI-9 closed; no other pin in the repo touches these signatures (grep `<:` for these two fns finds only the two sites).

## Verification

- **Before/after proof (throwaway, not committed):** a snippet calling `client.username_password_forgot_password(ForgotPasswordParams { user_id })` **without** turbofish — `cargo check` fails with E0283 on the pre-fix tree and succeeds after Step 1. This is the regression that the flipped tests themselves encode; no separate permanent test warranted (the pinned contract tests, edited per Step 2, are it).
- **Targeted tests:** `cargo test -p oxidauth username_password_forgot_password_route_contract` and `cargo test -p oxidauth username_password_update_password_route_contract` (native; wiremock dev-deps are `not(target_arch = "wasm32")`-gated; package name is `oxidauth`, per `oxidauth-rs/Cargo.toml:2` — same note as OXA-000031). Success-leg wire assertions unchanged: the `contract_raw` fixtures at `forgot_password.rs:40-43` / `update_password.rs:38-41` must pass byte-identically.
- **Post-fix greps:** `BUG(pinned)` absent from `forgot_password.rs`; `::<ForgotPasswordParams>` / `::<UpdatePasswordParams>` absent from `oxidauth-rs/src/client/` (and repo-wide grep for these two turbofishes empty).
- **Crate suite:** `cargo test -p oxidauth` green — no other consumer, mock, or wasm surface exists to break (grep-verified above).
