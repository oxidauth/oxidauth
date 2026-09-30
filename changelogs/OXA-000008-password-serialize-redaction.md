- [OXA-000008](https://www.pivotaltracker.com/story/show/OXA-000008) - `Password` no longer serializes the raw secret, and `UpdatePasswordParams` masks every secret field in `Debug`
    - **`oxidauth_kernel::Password` no longer implements `serde::Serialize`.** The
      derived newtype `Serialize` emitted the credential verbatim — `to_json(password)`
      *was* the secret, with no compile-time or runtime signal — while redaction lived
      only in the hand-written `Debug`. `Deserialize` stays (every inbound parse goes
      through it), raw material remains reachable only via `inner_value()`, and the
      pin test `password_serde_carries_the_raw_value` was flipped to
      `password_serde_deserializes_but_never_serializes`, which now also carries a
      red pin that `Password: !Serialize` re-deriving the impl cannot survive.
      The rejected alternative — `Serialize` emitting `"******"` — was the trap: the
      kernel's `Deserialize` accepts any string, so the bootstrap admin handoff would
      have silently stored `******` as the admin password while printing a different
      one to stdout.
    - **The one production caller was rewritten loudly, not silently.**
      `bootstrap::first_or_register_user` builds the in-process register handoff as an
      explicit `json!({...})` from the raw `String` (precedent: `Client::auth`'s
      authenticate body), and `UsernamePasswordRegisterParams::to_value()` — whose
      only caller was bootstrap — is gone; the struct itself is now
      `Deserialize`-only. The two test-only `Serialize` derives on
      `authorities::strategies::username_password::{UsernamePasswordAuthenticateInputs,
      UsernamePasswordRegisterInputs}` were dropped as dead weight (`Debug` masks stay
      pinned by `testing_manual_password_debug_impl`). The bootstrap payload
      assertions (`register["params"]["password"]` carries the env/generated secret,
      both fields) prove the handoff still lands raw, end to end.
    - **`UpdatePasswordParams::Debug` is now hand-written and constant-masked.**
      All three `#[tracing::instrument(.., skip(self))]` layers (api handler, service
      use case, SDK wrapper) record the `params` argument through `Debug`, so the
      derived impl was persisting the user's **new password, its confirmation, and
      the TOTP recovery code in plaintext logs** on every completed password reset.
      `code`, `password`, and `password_conf` now render as `"******"` (no length,
      no prefix), matching `Password`'s mask. A regression test drives the real
      instrumented use case through a captured tracing subscriber: the sentinel
      password is gone from the recorded log while the update still runs through.
    - **Consumer-visible delta:** removal, not redaction, for serialization — any
      external `oxidauth-kernel` 0.9.0 consumer that serialized a `Password` (or a
      struct embedding one) stops compiling; none exists in-workspace (audited: the
      only production `Password::serialize` call was the in-process bootstrap
      handoff), and this is the same class of breakage as the 0.9.0
      `oxidauth-http::response` deprecation. On the `Debug`/tracing surface, log
      readers see the masked `UpdatePasswordParams { code: "******", username:
      "<kept>", client_key: <kept>, password: "******", password_conf: "******" }`
      shape where the plaintext used to appear. **No wire, DB, or API change:**
      `user_authorities.params` stores only `{password_hash}`; the SDK's
      `update_password` request body and the client authenticate body still carry
      raw passwords as plain JSON strings (wiremock contracts green; hurl
      full-stack regression deferred to the integration pass — the suite
      never touches the kernel type anyway).
