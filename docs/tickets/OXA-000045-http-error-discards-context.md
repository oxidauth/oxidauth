# OXA-000045 — Response::error() wipes the whole envelope when its argument won't serialize

**Original ID:** SRV-9 · **Severity:** P2 · **Type:** bug · **Status:** open
**Review tier:** Tier 4 (moderate — multi-file recipes, pinned-test flips) · reviewed 2026-09-30 · REJECTED — **wontfix** (owner decision; defect verified real but dormant: every in-tree diagnostic argument type infallible (ApiError always serializes), so the wipe branch is unreachable in prod — owner declines to patch vendored `http/src/response/mod.rs` for an unreachable branch; register note SRV-9 stands, pins intentionally remain; reopen trigger: first handler passing a raw `Value`/non-ApiError diagnostic to `.error()` (the `register.rs:130` path) makes it live — fix recipe (mutate-not-replace + machine-readable diagnostic + tracing::warn + vendoring comment) recorded in ticket if reopened)


## Locations

- `src/xlib/http/src/lib.rs:77-95` — `Response::error()`: the defect (failure branch at :78-81).
- `src/xlib/http/src/lib.rs:97-115` — `Response::warning()`: identical reset pattern (:98-101).
- `src/xlib/http/src/lib.rs:117-135` — `Response::notice()`: identical reset pattern (:118-121).
- `src/xlib/http/src/lib.rs:282-300` — pinned test `error_that_cannot_serialize_resets_builder_to_500`; `BUG(pinned)` marker at **:284**.
- `src/xlib/http/src/lib.rs:302-324` — companion pinned test `warning_and_notice_serialize_failures_reset_to_500_too` ("Same pinned reset path", :304).
- `src/xlib/http/src/lib.rs:142-149` — `IntoResponse`: the *actual* payload-serialization path (`Json(self)`).
- Callers (representative): `src/oxidauth/oxidauth-api/src/server/api/v1/auth/authenticate.rs:42`, `.../can/mod.rs:24,31,36,38`, `.../settings/save_setting.rs:39` (~104 `.error(` sites across 61 files in `oxidauth-api`; see Impact).
- Client-side consumers: `src/oxidauth/oxidauth-rs/src/client/mod.rs:649-674` (`handle_response`), `:300-316` (auth path re-serializes `errors`), `:1431-1449` (test building multi-error envelopes).
- Re-export surface: `src/oxidauth/oxidauth-http/src/lib.rs:25` (`pub use http::Response;`).
- Error type flowing through `.error()`: `src/oxidauth/oxidauth-kernel/src/error.rs:7-45` (`OxidAuthError<C, E>`, `IntoOxidAuthError::into_error`).

## Problem

When the argument passed to `Response::error()` fails `serde_json::to_value`, the builder throws away the response accumulated so far and returns a brand-new `fail(500)` envelope containing only the generic string `"unable to serialize error to value"` (`src/xlib/http/src/lib.rs:78-81`):

```rust
pub fn error(mut self, err: impl Serialize) -> Self {
    let Ok(err) = serde_json::to_value(err) else {
        return Response::fail(StatusCode::INTERNAL_SERVER_ERROR)
            .error("unable to serialize error to value");
    };
    ...
```

The discarded state is broader than the register states: the reset drops the **payload**, **prior errors**, **warnings**, **notices** (the register omits these two), the **`success` flag**, and any **caller-chosen status** — e.g. an envelope built as `Response::bad_request()` silently re-emits as 500. `warning()` (:98-101) and `notice()` (:118-121) contain the same reset verbatim, each with its own canned message.

**Register drift (two corrections):**
1. `:284` is the `BUG(pinned)` marker inside the test module, not the defect; the defect bodies live at :77-95, :97-115, :117-135.
2. The trigger is **not** "payload-serialize failure". `Response::error()` never serializes the payload — it serializes the *error/warning/notice argument*. Payload serialization happens later, in `IntoResponse::into_response` via `Json(self)` (:146-147); if *that* fails, axum-core 0.5.6 replaces the entire response with a plain-text 500 ("Failure during serialization…"), losing the envelope *including* the caller's status. That second path is real but is a different mechanism, is exercised by no test, and is noted here only as related context (no existing SRV/OXA item covers it; worth a follow-up note, not folded into this ticket).

## Analysis

**What preserves vs. discards today.** On success, `error()` appends under `errors` in call order (first call allocates the `Vec`, later calls push — :83-93). On failure, `Response::fail(500)` builds a fresh struct (`success: false`, all `Option`s `None`, status 500) and the appended `"unable to serialize error to value"` becomes the *only* content. The fallback is recursion-safe: it re-enters `error()` with a `&str`, which cannot fail to serialize.

**Who calls this and what accumulates.**
- `oxidauth-services`: zero uses — the service layer returns `Result`; only `oxidauth-api` builds envelopes (61 files; the sole other reference is `oxidauth-rs` test code constructing fake server responses, `client/mod.rs:1437`).
- Occurrence breakdown of the ~104 production `.error(` call sites: 56 × `Response::bad_request().error(err.into_error())`, ~47 × `Response::bad_request().error(err.to_string())`, 2 string literals (client test). Every production site attaches exactly **one** error to a **fresh** `bad_request()` envelope; no production handler chains `.payload(...).error(...)`, two `.error()` calls, or `.error()` onto a `success()` envelope. `can/mod.rs:29-36` is the only site mixing `payload` + diagnostic lists, and it only appends string-literal `.notice`/`.warning`.

**How often can the trigger actually fire?** Today, effectively never. The reachable argument types are JSON-infallible:
- `String` / `&str` (`err.to_string()`, literals) — infallible.
- `OxidAuthError<(), String>` (`kernel/src/error.rs:7-22,32-45`) — derived `Serialize` over `String`, `Option<usize>`, `Option<()>`, `Option<String>` fields — infallible. (`into_error` hard-codes `context: ()` and stringified `source`.)

`serde_json::to_value` fails only when a `Serialize` impl errors by construction, on serde_json-specific constraints, or on structures exceeding serde_json's recursion-depth limit — none reachable from current argument types. The unit test drives the branch with a purpose-built `Unserializable` type (`lib.rs:156-164`, whose comment also notes `f64::NAN` maps to `Null` rather than erroring). So this is a **latent footgun baked into a vendored, reused builder**, not a currently firing defect. That is precisely why it's P2 rather than P1: the cost lands the first time someone passes a richer error context (e.g. a future `OxidAuthError<ValidationDetails, _>` whose nested types can fail, or a hand-written `Serialize` impl) into a multi-error/`payload`-bearing envelope.

**What clients lose when it fires.** The envelope is the whole API contract:
- Server side: all previously appended business errors, warnings, notices, and payload are gone, with **no log/trace** of what was dropped or which argument failed — silent data destruction in the response path.
- Typed client (`oxidauth-rs`): `handle_response` (`client/mod.rs:656-669`) maps `success:false` to `ClientErrorKind::APIResponseError` whose `source` is the `errors` array joined with `", "`; callers would see only `"unable to serialize error to value"` (JSON-quoted, per the pin at :1441-1446) — every actual validation/domain error vanishes. The `authenticate` path (:300-316) likewise stringifies `errors` for its failure source.
- Status semantics: a 400-class client error re-labels as 500, which misdirects retry policies, circuit breakers, and error dashboards (clients reasonably retry/back off on 5xx, not 4xx).
- A success envelope (`success:true` + payload + notices, e.g. the `can` shape) would flip to `success:false` with `payload` dropped, breaking any consumer that had already committed to treating the call as done.

**Vendoring constraint.** `src/xlib/http/src/lib.rs:1` declares the file "Vendored from freshbrewlabs project-template @ c38ec0a (2026-09); re-diff when templates change." Any local fix is a template divergence that must be recorded for the re-diff.

## Impact

- **Today:** no observable wire change for any current API consumer — the reset branch is unreachable with the infallible argument types actually in use.
- **Latent:** the moment any caller (or a template feature) appends an unserializable diagnostic to an envelope with state, consumers lose all accumulated errors/warnings/notices/payload, get a misleading 500 instead of the caller's status, and get a generic string where business errors belong. Because the builder API invites chaining (`success().payload(x).error(a).error(b)`), every future consumer of this vendored crate inherits the trap. `oxidauth-api` (all endpoints), `oxidauth-rs` typed client, and any dashboard/CLI reading `errors` are in the blast radius; `oxidauth-services` is not (no Response use).

## Proposed resolution

Fix all three methods (`error`, `warning`, `notice`) with one shared helper; do not just patch `error`:

1. On `to_value` failure, mutate `self` instead of replacing it:
   - retain previously accumulated `errors`/`warnings`/`notices`,
   - `payload = None` (the response can no longer be trusted to be self-consistent — document this in the method),
   - `success = false`,
   - `status_code = StatusCode::INTERNAL_SERVER_ERROR` (a diagnostic failing to serialize is a server defect; 5xx is the honest label),
   - append a **machine-readable** diagnostic instead of the bare string, e.g. `json!({ "kind": "response_diagnostics_serialize_failure", "field": "errors", "detail": "<truncate of serde err>" })` so clients can distinguish it from business errors,
   - `tracing::warn!` the discarded payload/error state and the serde error (the builder has no logger today; add `tracing`, already a workspace dependency pattern — otherwise this stays a silent-loss path with a slightly better shape).
2. Factor the push-or-init logic (:83-93, :103-113, :123-133) and the new failure branch into one `fn push_diagnostic(dst: &mut Option<Vec<Value>>, v: Value)` + one failure handler used by all three methods — today's bug is triplicated, so the fix should be unifiable once.
3. **Compat:** the change alters only the previously-unreachable failure branch. Wire shape for every currently-reachable response is byte-identical (skips-if-`None` serde attrs unchanged; `errors` stays `Vec<Value>`; client `handle_response` keeps working since it stringifies each `Value`). Status choice (force 500 vs. keep caller's 400) is the one judgment call: forcing 500 follows the register's proposal and honestly flags a server bug; keeping the caller's status keeps 4xx semantics for the primary failure. Recommend forcing 500 and preserving prior errors — the response is malformed-by-construction at that point, and the retained errors still explain the user-facing problem.
4. **Vendoring:** add a one-line comment at each touched site ("local divergence from template @ c38ec0a — see OXA-000045") so the re-diff ritual doesn't silently revert the fix; consider upstreaming.
5. **Pinned-test handling:** rewrite both pins to assert retention and drop the markers:
   - `error_that_cannot_serialize_resets_builder_to_500` (:282-300; delete `BUG(pinned)` comment :284-286; rename e.g. `error_that_cannot_serialize_keeps_prior_errors_and_fails_500`): flip the assertion from `{"success":false,"errors":["unable to serialize error to value"]}` to `{"success":false,"errors":[ <prior "first">, <diagnostic object> ]}` with status 500 and **no** `payload` key (payload dropped, `skip_serializing_if` applies).
   - `warning_and_notice_serialize_failures_reset_to_500_too` (:302-324; update companion comment :304): same flip, asserting prior `warnings`/`notices` survive and the diagnostic lands under `errors` (diagnostics are errors even when triggered by warning/notice args — keep that choice explicit in the test).
   - Add one new regression test chaining `success().payload(..).error(..).warning(..).notice(..).error(Unserializable)` asserting every prior entry survives and `success`/status/payload follow the resolution.

## Verification

- `cargo test -p http` (xlib/http package name is `http`): the two rewritten pins plus the new chained-builder test; confirm the old reset assertions are gone, not just duplicated.
- `cargo test -p oxidauth-rs` and `cargo test -p oxidauth-kernel`: `handle_response` mapping tests (`client/mod.rs:1431+`) and `into_error` envelope tests must pass unchanged — proof the client contract and the dominant `.error(err.into_error())` argument type are untouched.
- `cargo test -p oxidauth-api`: whole API suite unchanged (its handlers only use infallible arguments).
- Optional wire smoke: `./hurl.sh` against a local server — existing hurl assertions on `$.errors[0].debug` (per the note at `oxidauth-kernel/src/error.rs:55-57`) should pass with byte-identical envelopes, confirming zero reachable wire change.
