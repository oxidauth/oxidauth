use std::time::Duration;

pub use http::data::LoadingState;
use leptos::{logging::log, prelude::*};
use leptos_router::hooks::use_params_map;
use oxidauth::prelude::*;
use oxidauth_http::authorities::{
    create_authority::CreateAuthorityReq,
    list_all_authorities::ListAllAuthoritiesReq,
    update_authority::UpdateAuthorityReq,
};
use oxidauth_kernel::{
    JsonValue,
    authorities::{
        Authority,
        AuthoritySettings,
        AuthorityStatus,
        AuthorityStrategy,
        DEFAULT_JWT_NBF,
        NbfOffset,
        TotpSettings,
        create_authority::CreateAuthority,
        update_authority::UpdateAuthority,
    },
    jwt::EntitlementsEncoding,
};
use serde_json::Value;
use url::Url;
use uuid::Uuid;

use crate::{
    features::auth::signals::{handle_feature_error, logout_callback},
    state::AppState,
    time::format_local,
};

const DISABLED: &str = "disabled";
const ENABLED: &str = "enabled";

/// The row a pending delete confirmation refers to.
#[derive(Clone, Debug, PartialEq)]
pub struct PendingDelete {
    pub id: String,
    pub name: String,
}

/// One pre-formatted table row. The api returns every authority in a single
/// page, so the list narrows client-side and the table never re-fetches to
/// filter; `Authority` itself is not `Clone`, so rows arrive already rendered
/// to strings.
#[derive(Clone, Debug, PartialEq)]
pub struct AuthorityRow {
    pub id: String,
    pub name: String,
    pub strategy: String,
    pub status: String,
    pub client_key: String,
    pub created: String,
}

/// Signals and actions for the authorities list. Filtering is local to the
/// browser; the single fetch runs on mount and refetches after a delete.
#[derive(Clone, Copy)]
pub struct HandleAuthoritiesListResponse {
    pub authorities: ReadSignal<LoadingState<Vec<Authority>>>,
    /// The loaded rows narrowed by the local filter (empty while loading).
    pub visible: Signal<Vec<AuthorityRow>>,
    /// The narrowing in force: the empty string is "no narrowing".
    pub search: ReadSignal<String>,
    pub status: ReadSignal<String>,
    pub errors: ReadSignal<Option<Vec<String>>>,
    pub pending_delete: ReadSignal<Option<PendingDelete>>,
    pub apply_search: Callback<String>,
    pub apply_status: Callback<String>,
    pub request_delete: Callback<(String, String)>,
    pub cancel_delete: Callback<()>,
    pub confirm_delete: Action<String, ()>,
}

pub fn handle_authorities_list_signals(state: AppState) -> HandleAuthoritiesListResponse {
    let client = state.client.get_untracked();
    let logout = logout_callback(state);

    let (authorities, set_authorities) = signal(LoadingState::<Vec<Authority>>::Pending);
    let (search, set_search) = signal(String::new());
    let (status, set_status) = signal(String::new());
    let (errors, set_errors) = signal(None::<Vec<String>>);
    let (pending_delete, set_pending_delete) = signal(None::<PendingDelete>);

    let fetch = {
        let fetch_client = client.clone();

        Action::new_unsync(move |_: &()| {
            let client = fetch_client.clone();

            set_authorities.set(LoadingState::Loading);
            set_errors.set(None);

            async move {
                match client
                    .list_all_authorities(ListAllAuthoritiesReq {})
                    .await
                {
                    Ok(res) => set_authorities.set(LoadingState::Loaded(res.authorities)),
                    Err(err) => {
                        log!("error fetching authorities: {err}");

                        // a dead session is the logout redirect's job, not a
                        // banner's — leave the spinner up while it navigates
                        if let Some(msg) = handle_feature_error(err.as_ref(), &logout) {
                            set_authorities.set(LoadingState::Error(msg));
                        }
                    },
                }
            }
        })
    };

    // the endpoint is unpaginated: one fetch on mount, plus a refetch after
    // the delete below lands
    Effect::new(move |_| {
        fetch.dispatch(());
    });

    let visible = Signal::derive(move || {
        let needle = search
            .get()
            .trim()
            .to_lowercase();
        let status_filter = status.get();

        authorities.with(|state| {
            match state {
                LoadingState::Loaded(authorities) => {
                    authorities
                        .iter()
                        .filter(|authority| {
                            (needle.is_empty()
                                || authority
                                    .name
                                    .to_lowercase()
                                    .contains(&needle))
                                && (status_filter.is_empty()
                                    || authority.status.to_string() == status_filter)
                        })
                        .map(|authority| {
                            AuthorityRow {
                                id: authority.id.to_string(),
                                name: authority.name.clone(),
                                strategy: authority.strategy.to_string(),
                                status: authority.status.to_string(),
                                client_key: authority
                                    .client_key
                                    .to_string(),
                                created: format_local(&authority.created_at),
                            }
                        })
                        .collect()
                },
                _ => Vec::new(),
            }
        })
    });

    let apply_search = Callback::new(move |text: String| {
        set_search.set(text);
    });

    let apply_status = Callback::new(move |value: String| {
        set_status.set(value);
    });

    let request_delete = Callback::new(move |(id, name): (String, String)| {
        set_pending_delete.set(Some(PendingDelete { id, name }));
    });

    let cancel_delete = Callback::new(move |_: ()| {
        set_pending_delete.set(None);
    });

    let confirm_delete = Action::new_unsync(move |id: &String| {
        let id = id.clone();
        let client = client.clone();

        async move {
            match Uuid::parse_str(&id) {
                Ok(id) => {
                    match client
                        .delete_authority(id)
                        .await
                    {
                        Ok(_) => {
                            set_pending_delete.set(None);
                            fetch.dispatch(());
                        },
                        Err(err) => {
                            log!("error deleting authority {id}: {err}");
                            set_pending_delete.set(None);

                            if let Some(msg) = handle_feature_error(err.as_ref(), &logout) {
                                set_errors.set(Some(vec![msg]));
                            }
                        },
                    }
                },
                Err(err) => {
                    set_pending_delete.set(None);
                    set_errors.set(Some(vec![err.to_string()]));
                },
            }
        }
    });

    HandleAuthoritiesListResponse {
        authorities,
        visible,
        search,
        status,
        errors,
        pending_delete,
        apply_search,
        apply_status,
        request_delete,
        cancel_delete,
        confirm_delete,
    }
}

/// The whole form as one submit carries it; numeric and json fields arrive as
/// raw text and are validated when the action runs, before any request.
#[derive(Clone, Debug, PartialEq)]
pub struct AuthorityForm {
    /// `Some` when editing (the `:id` route), `None` when creating.
    pub id: Option<String>,
    /// The edit form carries the loaded client key back: the server mints a
    /// fresh one whenever an update omits it (OXA: rotating a live authority's
    /// key signs out every flow pointed at it).
    pub client_key: Option<String>,
    pub name: String,
    pub strategy: String,
    pub status: String,
    pub jwt_ttl: String,
    pub refresh_token_ttl: String,
    /// "disabled" | "enabled" — `nbf_offset` is read only when enabled.
    pub nbf: String,
    pub nbf_offset: String,
    /// "txt" | "gz"
    pub entitlements_encoding: String,
    /// "disabled" | "enabled" — the totp fields are read only when enabled.
    pub totp: String,
    pub totp_ttl: String,
    pub webhook: String,
    pub webhook_key: String,
    /// raw json text, validated before submit
    pub params: String,
}

/// Controlled inputs: one signal per field, prefilled when an edit loads.
#[derive(Clone, Copy)]
pub struct AuthorityFields {
    pub name: RwSignal<String>,
    pub strategy: RwSignal<String>,
    pub status: RwSignal<String>,
    pub client_key: RwSignal<String>,
    pub jwt_ttl: RwSignal<String>,
    pub refresh_token_ttl: RwSignal<String>,
    pub nbf: RwSignal<String>,
    pub nbf_offset: RwSignal<String>,
    pub entitlements_encoding: RwSignal<String>,
    pub totp: RwSignal<String>,
    pub totp_ttl: RwSignal<String>,
    pub webhook: RwSignal<String>,
    pub webhook_key: RwSignal<String>,
    pub params: RwSignal<String>,
}

#[derive(Clone, Copy)]
pub struct HandleAuthorityFormResponse {
    /// True on `/authorities/:id/edit`, false on `/authorities/new`.
    pub is_edit: Signal<bool>,
    pub authority_id: Signal<String>,
    /// the preloaded authority in edit mode (pending/loaded only — the form
    /// renders once it arrives); creation keeps this at `Pending`
    pub authority: ReadSignal<LoadingState<Authority>>,
    pub fields: AuthorityFields,
    pub errors: ReadSignal<Option<Vec<String>>>,
    /// Resolves with the saved authority's id; parse and api errors land in
    /// `errors` as well.
    pub save: Action<AuthorityForm, Result<Uuid, String>>,
}

pub fn handle_authority_form_signals(state: AppState) -> HandleAuthorityFormResponse {
    let client = state.client.get_untracked();
    let logout = logout_callback(state);

    let params = use_params_map();
    let authority_id = Signal::derive(move || {
        params
            .with(|params| params.get("id"))
            .unwrap_or_default()
    });
    let is_edit = Signal::derive(move || !authority_id.get().is_empty());

    let (authority, set_authority) = signal(LoadingState::<Authority>::Pending);
    let (errors, set_errors) = signal(None::<Vec<String>>);

    let fields = AuthorityFields {
        name: signal_default(""),
        strategy: signal_default("username_password"),
        status: signal_default("enabled"),
        client_key: signal_default(""),
        jwt_ttl: signal_default("3600"),
        refresh_token_ttl: signal_default("2592000"),
        nbf: signal_default(ENABLED),
        nbf_offset: signal_default(
            &DEFAULT_JWT_NBF
                .as_secs()
                .to_string(),
        ),
        entitlements_encoding: signal_default("txt"),
        totp: signal_default(DISABLED),
        totp_ttl: signal_default("30"),
        webhook: signal_default(""),
        webhook_key: signal_default(""),
        params: signal_default("{}"),
    };

    let store_error = Callback::new(move |error: String| {
        set_errors.set(Some(vec![error]));
    });

    // edit mode preloads through find_authority_by_id and seeds every field
    // from the loaded authority before the form renders
    let fetch = {
        let preload_client = client.clone();

        Action::new_unsync(move |id: &String| {
            let id = id.clone();
            let client = preload_client.clone();

            set_authority.set(LoadingState::Loading);
            set_errors.set(None);

            async move {
                match Uuid::parse_str(&id) {
                    Ok(id) => {
                        match client
                            .find_authority_by_id(id)
                            .await
                        {
                            Ok(res) => {
                                let loaded = &res.authority;

                                fields
                                    .name
                                    .set(loaded.name.clone());
                                fields
                                    .strategy
                                    .set(loaded.strategy.to_string());
                                fields
                                    .status
                                    .set(loaded.status.to_string());
                                fields
                                    .client_key
                                    .set(loaded.client_key.to_string());
                                fields.jwt_ttl.set(
                                    loaded
                                        .settings
                                        .jwt_ttl
                                        .as_secs()
                                        .to_string(),
                                );
                                fields.refresh_token_ttl.set(
                                    loaded
                                        .settings
                                        .refresh_token_ttl
                                        .as_secs()
                                        .to_string(),
                                );

                                match &loaded.settings.jwt_nbf_offset {
                                    NbfOffset::Disabled => {
                                        fields
                                            .nbf
                                            .set(DISABLED.to_string());
                                    },
                                    NbfOffset::Enabled(offset) => {
                                        fields
                                            .nbf
                                            .set(ENABLED.to_string());
                                        fields
                                            .nbf_offset
                                            .set(offset.as_secs().to_string());
                                    },
                                }

                                fields
                                    .entitlements_encoding
                                    .set(
                                        match loaded
                                            .settings
                                            .entitlements_encoding
                                        {
                                            EntitlementsEncoding::Txt => "txt".to_string(),
                                            EntitlementsEncoding::Gz => "gz".to_string(),
                                        },
                                    );

                                match &loaded.settings.totp {
                                    TotpSettings::Disabled => {
                                        fields
                                            .totp
                                            .set(DISABLED.to_string());
                                    },
                                    TotpSettings::Enabled {
                                        totp_ttl,
                                        webhook,
                                        webhook_key,
                                    } => {
                                        fields
                                            .totp
                                            .set(ENABLED.to_string());
                                        fields
                                            .totp_ttl
                                            .set(totp_ttl.as_secs().to_string());
                                        fields
                                            .webhook
                                            .set(webhook.to_string());
                                        fields
                                            .webhook_key
                                            .set(webhook_key.clone());
                                    },
                                }

                                fields.params.set(
                                    serde_json::to_string_pretty(&*loaded.params)
                                        .unwrap_or_else(|_| "{}".to_string()),
                                );

                                set_authority.set(LoadingState::Loaded(res.authority));
                            },
                            Err(err) => {
                                log!("error fetching authority {id}: {err}");

                                if let Some(msg) = handle_feature_error(err.as_ref(), &logout) {
                                    set_authority.set(LoadingState::Error(msg));
                                }
                            },
                        }
                    },
                    Err(_) => {
                        set_authority
                            .set(LoadingState::Error(format!("invalid authority id: {id}")));
                    },
                }
            }
        })
    };

    Effect::new(move |_| {
        let id = authority_id.get();

        if !id.is_empty() {
            fetch.dispatch(id);
        }
    });

    let save = Action::new_unsync(move |form: &AuthorityForm| {
        let form = form.clone();
        let client = client.clone();

        async move {
            let outcome: Result<Uuid, String> = async {
                let name = form.name.trim().to_string();

                if name.is_empty() {
                    return Err("name is required".to_string());
                }

                // everything below validates client-side before any request
                let strategy: AuthorityStrategy = form
                    .strategy
                    .parse()
                    .map_err(
                        |err: oxidauth_kernel::authorities::ParseAuthorityStrategyError| {
                            err.to_string()
                        },
                    )?;
                let status: AuthorityStatus = form
                    .status
                    .parse()
                    .map_err(|err: BoxedError| err.to_string())?;
                let settings = build_settings(&form)?;
                let params: Value = serde_json::from_str(form.params.trim())
                    .map_err(|err| format!("params must be valid json: {err}"))?;

                match &form.id {
                    None => {
                        client
                            .create_authority(CreateAuthorityReq {
                                authority: CreateAuthority {
                                    name,
                                    client_key: None,
                                    status: Some(status),
                                    strategy,
                                    settings,
                                    params: JsonValue::new(params),
                                },
                            })
                            .await
                            .map(|res| res.authority.id)
                            .map_err(|err| err.to_string())
                    },
                    Some(id) => {
                        let authority_id = Uuid::parse_str(id).map_err(|err| err.to_string())?;
                        let client_key = Uuid::parse_str(
                            form.client_key
                                .as_deref()
                                .unwrap_or_default(),
                        )
                        .map_err(|err| format!("client key must be a uuid: {err}"))?;

                        client
                            .update_authority(
                                authority_id,
                                UpdateAuthorityReq {
                                    authority: UpdateAuthority {
                                        id: Some(authority_id),
                                        name,
                                        client_key: Some(client_key),
                                        status: Some(status),
                                        strategy,
                                        settings,
                                        params,
                                    },
                                },
                            )
                            .await
                            .map(|res| res.authority.id)
                            .map_err(|err| err.to_string())
                    },
                }
            }
            .await;

            if let Err(msg) = &outcome {
                store_error.run(msg.clone());
            }

            outcome
        }
    });

    HandleAuthorityFormResponse {
        is_edit,
        authority_id,
        authority,
        fields,
        errors,
        save,
    }
}

fn signal_default(value: &str) -> RwSignal<String> {
    RwSignal::new(value.to_string())
}

fn parse_secs(field: &str, text: &str) -> Result<Duration, String> {
    text.trim()
        .parse::<u64>()
        .map(Duration::from_secs)
        .map_err(|err| format!("{field} must be a whole number of seconds: {err}"))
}

/// the settings envelope: every ttl in whole seconds, the nbf and totp
/// discriminants from their mode selects, the encoding from its select
fn build_settings(form: &AuthorityForm) -> Result<AuthoritySettings, String> {
    let jwt_ttl = parse_secs("jwt ttl", &form.jwt_ttl)?;
    let refresh_token_ttl = parse_secs("refresh token ttl", &form.refresh_token_ttl)?;

    let jwt_nbf_offset = match form.nbf.as_str() {
        DISABLED => NbfOffset::Disabled,
        ENABLED => NbfOffset::Enabled(parse_secs("jwt nbf offset", &form.nbf_offset)?),
        mode => return Err(format!("unknown jwt nbf offset mode: {mode}")),
    };

    let entitlements_encoding = match form
        .entitlements_encoding
        .as_str()
    {
        "txt" => EntitlementsEncoding::Txt,
        "gz" => EntitlementsEncoding::Gz,
        encoding => return Err(format!("unknown entitlements encoding: {encoding}")),
    };

    let totp = match form.totp.as_str() {
        DISABLED => TotpSettings::Disabled,
        ENABLED => {
            TotpSettings::Enabled {
                totp_ttl: parse_secs("totp ttl", &form.totp_ttl)?,
                webhook: Url::parse(form.webhook.trim())
                    .map_err(|err| format!("totp webhook must be a valid url: {err}"))?,
                webhook_key: form.webhook_key.clone(),
            }
        },
        mode => return Err(format!("unknown totp mode: {mode}")),
    };

    Ok(AuthoritySettings {
        jwt_ttl,
        jwt_nbf_offset,
        refresh_token_ttl,
        totp,
        entitlements_encoding,
    })
}
