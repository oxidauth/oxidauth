use std::{env, time::Duration};

use async_trait::async_trait;
use oxidauth_kernel::{
    JsonValue,
    auth::register::{RegisterParams, RegisterService},
    authorities::{
        Authority,
        AuthorityNotFoundError,
        AuthoritySettings,
        AuthorityStrategy,
        NbfOffset,
        TotpSettings,
        create_authority::{CreateAuthority, CreateAuthorityService},
        find_authority_by_strategy::{FindAuthorityByStrategy, FindAuthorityByStrategyService},
    },
    bootstrap::{BootstrapParams, BootstrapServiceTrait},
    error::BoxedError,
    jwt::EntitlementsEncoding,
    permissions::{
        Permission,
        PermissionNotFoundError,
        create_permission::{CreatePermission, CreatePermissionService},
        find_permission_by_parts::{FindPermissionByParts, FindPermissionByPartsService},
    },
    public_keys::{
        PublicKey,
        create_public_key::{CreatePublicKey, CreatePublicKeyService},
        list_all_public_keys::{ListAllPublicKeys, ListAllPublicKeysService},
    },
    role_permission_grants::{
        create_role_permission_grant::{
            CreateRolePermissionGrant,
            CreateRolePermissionGrantService,
        },
        list_role_permission_grants_by_role_id::{
            ListRolePermissionGrantsByRoleId,
            ListRolePermissionGrantsByRoleIdService,
        },
    },
    roles::{
        Role,
        create_role::{CreateRole, CreateRoleService},
        list_all_roles::{ListAllRoles, ListAllRolesService},
    },
    settings::{
        Setting,
        fetch_setting::{FetchSettingParams, FetchSettingService, SettingNotFoundError},
        save_setting::{SaveSettingParams, SaveSettingService},
    },
    user_role_grants::{
        create_user_role_grant::{CreateUserRoleGrant, CreateUserRoleGrantService},
        list_user_role_grants_by_user_id::{
            ListUserRoleGrantsByUserId,
            ListUserRoleGrantsByUserIdService,
        },
    },
    users::{
        User,
        UserNotFoundError,
        find_user_by_username::{FindUserByUsername, FindUserByUsernameService},
    },
};
use provider::Provider;
use serde_json::json;
use tracing::{error, info};

use crate::{auth::strategies::username_password::AuthorityParams, random_string};

pub struct SudoUserBootstrapUseCase {
    provider: Provider,
}

impl SudoUserBootstrapUseCase {
    pub fn new(provider: &Provider) -> Self {
        Self {
            provider: provider.clone(),
        }
    }
}

#[async_trait]
impl BootstrapServiceTrait for SudoUserBootstrapUseCase {
    #[tracing::instrument(name = "SudoUserBootstrapUseCase::bootstrap", skip(self))]
    async fn bootstrap(&self, params: &BootstrapParams) -> Result<(), BoxedError> {
        let setting = {
            let fetch_setting = self
                .provider
                .fetch_unchecked();

            check_bootstrap_setting(fetch_setting).await?
        };

        if setting.is_some() {
            info!("bootstrap already completed");

            return Ok(());
        }

        info!("no bootstrap detected -- starting bootstrap");

        {
            let list_all_public_keys = self
                .provider
                .fetch_unchecked();

            let create_public_key = self
                .provider
                .fetch_unchecked();

            first_or_create_public_key(list_all_public_keys, create_public_key).await?;
        }

        let permission = {
            let permission_by_name = self
                .provider
                .fetch_unchecked();

            let create_permission = self
                .provider
                .fetch_unchecked();

            first_or_create_permissions(permission_by_name, create_permission).await?
        };

        let role = {
            let list_all_roles = self
                .provider
                .fetch_unchecked();

            let create_role = self
                .provider
                .fetch_unchecked();

            first_or_create_role(list_all_roles, create_role).await?
        };

        {
            let list_role_permission_grants = self
                .provider
                .fetch_unchecked();
            let create_role_permission = self
                .provider
                .fetch_unchecked();

            add_admin_permission_to_admin_role(
                list_role_permission_grants,
                create_role_permission,
                &role,
                &permission,
            )
            .await?;
        }

        let authority = {
            let authority_by_strategy = self
                .provider
                .fetch_unchecked();
            let create_authority = self
                .provider
                .fetch_unchecked();

            first_or_create_authority(authority_by_strategy, create_authority).await?
        };

        let user = {
            let find_user_by_username = self
                .provider
                .fetch_unchecked();
            let register_user = self
                .provider
                .fetch_unchecked();

            first_or_register_user(find_user_by_username, register_user, &authority).await?
        };

        {
            let list_user_role = self
                .provider
                .fetch_unchecked();
            let create_user_role = self
                .provider
                .fetch_unchecked();

            add_admin_role_to_admin_user(list_user_role, create_user_role, &user, &role).await?;
        }

        {
            let save_setting = self
                .provider
                .fetch_unchecked();

            save_bootstrap_setting(save_setting).await?;
        }

        Ok(())
    }
}

pub const BOOTSTRAP_SETTING_KEY: &str = "bootstrap";

#[tracing::instrument(skip_all)]
async fn check_bootstrap_setting(
    fetch_settings_service: &FetchSettingService,
) -> Result<Option<Setting>, BoxedError> {
    let bootstrap_setting = fetch_settings_service
        .fetch_setting(&FetchSettingParams {
            key: BOOTSTRAP_SETTING_KEY.to_owned(),
        })
        .await;

    match bootstrap_setting {
        Ok(setting) => Ok(Some(setting)),
        Err(err) => {
            match err.downcast_ref::<SettingNotFoundError>() {
                Some(_) => return Ok(None),
                _ => return Err(err),
            }
        },
    }
}

#[tracing::instrument(skip_all)]
async fn first_or_create_public_key(
    list_all_public_keys: &ListAllPublicKeysService,
    create_public_key: &CreatePublicKeyService,
) -> Result<PublicKey, BoxedError> {
    let mut public_keys = list_all_public_keys
        .list_all_public_keys(&ListAllPublicKeys)
        .await?;

    if let Some(public_key) = public_keys.pop() {
        return Ok(public_key);
    }

    create_public_key
        .create_public_key(&CreatePublicKey)
        .await
}

pub const ADMIN_PERMISSION: &str = "oxidauth:**:**";
pub const TOTP_VALIDATE_PERMISSION: &str = "oxidauth:totp_code:validate";
/// OXA-000005 Step 1: guards `POST /auth/username_password/forgot_password`
/// (`oxidauth-api`'s forgot_password handler `PERMISSION`). Adding a new
/// permission string always requires this seed step (or an equivalent
/// permissions migration/CRUD entry) so the string exists in the tree and is
/// grantable; the seeded admin role needs nothing extra — its
/// `oxidauth:**:**` wildcard already answers the challenge.
pub const FORGOT_PASSWORD_PERMISSION: &str = "oxidauth:auth:forgot_password";

#[tracing::instrument(skip_all)]
async fn first_or_create_permissions(
    permission_by_name: &FindPermissionByPartsService,
    create_permission: &CreatePermissionService,
) -> Result<Permission, BoxedError> {
    // TODO(dewey4iv): currently not returning but might want to later

    let permission_name = TOTP_VALIDATE_PERMISSION.to_owned();

    let permission = permission_by_name
        .find_permission_by_parts(&FindPermissionByParts {
            permission: permission_name.clone(),
        })
        .await;

    let _totp_permission = match permission {
        Ok(permission) => Ok(permission),
        Err(err) => {
            match err.downcast_ref::<PermissionNotFoundError>() {
                Some(_) => {
                    create_permission
                        .create_permission(&CreatePermission {
                            permission: permission_name,
                        })
                        .await
                },
                None => return Err(err),
            }
        },
    };

    // OXA-000005: register the forgot_password permission in the tree (the
    // route gate itself only reads jwt entitlements, so the row's job is to
    // make the string grantable; result intentionally unchecked, like totp).
    let permission_name = FORGOT_PASSWORD_PERMISSION.to_owned();

    let permission = permission_by_name
        .find_permission_by_parts(&FindPermissionByParts {
            permission: permission_name.clone(),
        })
        .await;

    let _forgot_password_permission = match permission {
        Ok(permission) => Ok(permission),
        Err(err) => {
            match err.downcast_ref::<PermissionNotFoundError>() {
                Some(_) => {
                    create_permission
                        .create_permission(&CreatePermission {
                            permission: permission_name,
                        })
                        .await
                },
                None => return Err(err),
            }
        },
    };

    let permission_name = ADMIN_PERMISSION.to_owned();

    let permission = permission_by_name
        .find_permission_by_parts(&FindPermissionByParts {
            permission: permission_name.clone(),
        })
        .await;

    match permission {
        Ok(permission) => Ok(permission),
        Err(err) => {
            match err.downcast_ref::<PermissionNotFoundError>() {
                Some(_) => {
                    create_permission
                        .create_permission(&CreatePermission {
                            permission: permission_name,
                        })
                        .await
                },
                None => return Err(err),
            }
        },
    }
}

pub const ADMIN_ROLE: &str = "oxidauth:admin";

// TODO(dewey4iv): https://www.pivotaltracker.com/story/show/186917442
// I'm using the list to search by name.
// In the interest of getting this done quickly and because
// bootstrapping will only get triggered when there's almost nothing
// in the database
#[tracing::instrument(skip_all)]
async fn first_or_create_role(
    list_all_roles: &ListAllRolesService,
    create_role: &CreateRoleService,
) -> Result<Role, BoxedError> {
    let roles = list_all_roles
        .list_all_roles(&ListAllRoles)
        .await?;

    let admin_role = roles
        .into_iter()
        .find(|role| role.name == ADMIN_ROLE);

    if let Some(admin_role) = admin_role {
        return Ok(admin_role);
    }

    create_role
        .create_role(&CreateRole {
            name: ADMIN_ROLE.to_owned(),
        })
        .await
}

#[tracing::instrument(skip_all)]
async fn add_admin_permission_to_admin_role(
    list_role_permission_grants: &ListRolePermissionGrantsByRoleIdService,
    create_role_permission: &CreateRolePermissionGrantService,
    role: &Role,
    permission: &Permission,
) -> Result<(), BoxedError> {
    let role_permissions = list_role_permission_grants
        .list_role_permission_grants_by_role_id(&ListRolePermissionGrantsByRoleId {
            role_id: role.id,
        })
        .await?;

    let admin_role_permission = role_permissions
        .into_iter()
        .find(|rp| rp.permission.id == permission.id);

    if admin_role_permission.is_some() {
        return Ok(());
    }

    create_role_permission
        .create_role_permission_grant(&CreateRolePermissionGrant {
            role_id: role.id,
            permission: ADMIN_PERMISSION.to_owned(),
        })
        .await?;

    Ok(())
}

pub const DEFAULT_JWT_TTL: Duration = Duration::from_secs(60 * 2);
pub const DEFAULT_TOTP_TOKEN_TTL: Duration = Duration::from_secs(60 * 2);
pub const DEFAULT_CLIENT_KEY: &str = "OXIDAUTH_DEFAULT_CLIENT_KEY";
pub const DEFAULT_REFRESH_TOKEN_TTL: Duration = Duration::from_secs(60 * 60 * 24 * 2);
pub const DEFAULT_USERNAMEPASSWORD_NAME: &str = "oxidauth default username_password";

#[tracing::instrument(skip_all)]
async fn first_or_create_authority(
    authority_by_strategy: &FindAuthorityByStrategyService,
    create_authority: &CreateAuthorityService,
) -> Result<Authority, BoxedError> {
    // TODO(dewey4iv): we should swap this for something
    // that can pull the authority by the name
    let authority = authority_by_strategy
        .find_authority_by_strategy(&FindAuthorityByStrategy {
            strategy: AuthorityStrategy::UsernamePassword,
        })
        .await;

    match authority {
        Ok(authority) => Ok(authority),
        Err(err) => {
            info!("authority not found -- creating authority");

            match err.downcast_ref::<Box<AuthorityNotFoundError>>() {
                Some(_) => {
                    info!("attempting to create authority");

                    let client_key = env::var(DEFAULT_CLIENT_KEY)
                        .ok()
                        .map(|key| key.parse())
                        .transpose()
                        .ok()
                        .flatten();

                    let authority_params_value =
                        AuthorityParams::new(random_string()).as_json_value()?;

                    let authority_settings = AuthoritySettings {
                        jwt_ttl: DEFAULT_JWT_TTL,
                        jwt_nbf_offset: NbfOffset::default(),
                        refresh_token_ttl: DEFAULT_REFRESH_TOKEN_TTL,
                        totp: TotpSettings::Disabled,
                        entitlements_encoding: EntitlementsEncoding::Txt,
                    };

                    let mut create_authority_params = CreateAuthority {
                        name: DEFAULT_USERNAMEPASSWORD_NAME.to_string(),
                        client_key,
                        status: None,
                        strategy: AuthorityStrategy::UsernamePassword,
                        settings: authority_settings,
                        params: authority_params_value,
                    };

                    create_authority
                        .create_authority(&mut create_authority_params)
                        .await
                },
                _ => {
                    error!(message = "authority could not be found or created", ?err);
                    return Err(err);
                },
            }
        },
    }
}

pub const DEFAULT_ADMIN_USERNAME: &str = "oxidauth:admin";
pub const DEFAULT_ADMIN_PASSWORD: &str = "OXIDAUTH_DEFAULT_ADMIN_PASSWORD";

#[tracing::instrument(skip_all)]
async fn first_or_register_user(
    find_user_by_username: &FindUserByUsernameService,
    register_user: &RegisterService,
    authority: &Authority,
) -> Result<User, BoxedError> {
    let user = find_user_by_username
        .find_user_by_username(&FindUserByUsername {
            username: DEFAULT_ADMIN_USERNAME.parse()?,
        })
        .await;

    match user {
        Ok(user) => return Ok(user),
        Err(err) => {
            match err.downcast_ref::<Box<UserNotFoundError>>() {
                Some(_) => {
                    let password =
                        env::var(DEFAULT_ADMIN_PASSWORD).unwrap_or_else(|_| random_string());

                    println!(":::\nDEFAULT ADMIN PASSWORD: {}\n:::", password);

                    // `Password` deliberately does not implement `Serialize`
                    // (OXA-000008); this in-process handoff is re-parsed by
                    // the registrar's `TryFrom<JsonValue>`, so the raw admin
                    // password is written as an explicit `String` here —
                    // mirroring `Client::auth`'s `json!` authenticate body.
                    let username_password_params = json!({
                        "username": DEFAULT_ADMIN_USERNAME,
                        "password": &password,
                        "password_confirmation": &password,
                        "kind": "api",
                    });

                    let register_params = RegisterParams {
                        client_key: authority.client_key,
                        params: JsonValue::new(username_password_params),
                    };

                    register_user
                        .register(&register_params)
                        .await?;

                    let registered = find_user_by_username
                        .find_user_by_username(&FindUserByUsername {
                            username: DEFAULT_ADMIN_USERNAME.parse()?,
                        })
                        .await?;

                    return Ok(registered);
                },

                None => return Err(err),
            }
        },
    }
}

#[tracing::instrument(skip_all)]
async fn add_admin_role_to_admin_user(
    list_user_role: &ListUserRoleGrantsByUserIdService,
    create_user_role: &CreateUserRoleGrantService,
    user: &User,
    role: &Role,
) -> Result<(), BoxedError> {
    let user_role_grants = list_user_role
        .list_user_role_grants_by_user_id(&ListUserRoleGrantsByUserId { user_id: user.id })
        .await?;

    let user_role_grant = user_role_grants
        .into_iter()
        .find(|grant| grant.role.id == role.id);

    if user_role_grant.is_some() {
        return Ok(());
    }

    create_user_role
        .create_user_role_grant(&CreateUserRoleGrant {
            user_id: user.id,
            role_id: role.id,
        })
        .await?;

    Ok(())
}

#[tracing::instrument(skip_all)]
async fn save_bootstrap_setting(save_setting: &SaveSettingService) -> Result<(), BoxedError> {
    save_setting
        .save_setting(&SaveSettingParams {
            key: BOOTSTRAP_SETTING_KEY.to_owned(),
            value: serde_json::Value::Bool(true),
        })
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::{
        Arc,
        Mutex,
        MutexGuard,
        atomic::{AtomicUsize, Ordering},
    };

    use chrono::Utc;
    use oxidauth_kernel::{
        JsonValue,
        auth::register::{RegisterParams, RegisterResponse},
        authorities::{
            Authority,
            AuthorityNotFoundError,
            AuthoritySettings,
            AuthorityStatus,
            AuthorityStrategy,
            TotpSettings,
            create_authority::{CreateAuthority, CreateAuthorityServiceTrait},
            find_authority_by_strategy::{
                FindAuthorityByStrategy,
                FindAuthorityByStrategyServiceTrait,
            },
        },
        bootstrap::{BootstrapParams, BootstrapServiceTrait},
        jwt::EntitlementsEncoding,
        permissions::{
            Permission,
            PermissionNotFoundError,
            create_permission::{CreatePermission, CreatePermissionServiceTrait},
            find_permission_by_parts::{FindPermissionByParts, FindPermissionByPartsServiceTrait},
        },
        public_keys::{
            PublicKey,
            create_public_key::{CreatePublicKey, CreatePublicKeyServiceTrait},
            list_all_public_keys::{ListAllPublicKeys, ListAllPublicKeysServiceTrait},
        },
        role_permission_grants::{
            RolePermission,
            RolePermissionGrant,
            create_role_permission_grant::{
                CreateRolePermissionGrant,
                CreateRolePermissionGrantServiceTrait,
            },
            list_role_permission_grants_by_role_id::{
                ListRolePermissionGrantsByRoleId,
                ListRolePermissionGrantsByRoleIdServiceTrait,
            },
        },
        roles::{
            Role,
            create_role::{CreateRole, CreateRoleServiceTrait},
            list_all_roles::{ListAllRoles, ListAllRolesServiceTrait},
        },
        settings::{
            Setting,
            fetch_setting::{FetchSettingParams, FetchSettingServiceTrait, SettingNotFoundError},
            save_setting::{SaveSettingParams, SaveSettingServiceTrait},
        },
        user_role_grants::{
            UserRole,
            UserRoleGrant,
            create_user_role_grant::{CreateUserRoleGrant, CreateUserRoleGrantServiceTrait},
            list_user_role_grants_by_user_id::{
                ListUserRoleGrantsByUserId,
                ListUserRoleGrantsByUserIdServiceTrait,
            },
        },
        users::{
            User,
            UserKind,
            UserNotFoundError,
            UserStatus,
            find_user_by_username::{FindUserByUsername, FindUserByUsernameServiceTrait},
        },
    };
    use provider::Provider;
    use serde_json::{Value, json};
    use uuid::Uuid;

    use super::*;

    const PUBKEY_ID: Uuid = uuid::uuid!("6f1b2c3d-6666-4000-8000-000000000001");
    const TOTP_PERM_ID: Uuid = uuid::uuid!("6f1b2c3d-6666-4000-8000-000000000002");
    const ADMIN_PERM_ID: Uuid = uuid::uuid!("6f1b2c3d-6666-4000-8000-000000000003");
    const ROLE_ID: Uuid = uuid::uuid!("6f1b2c3d-6666-4000-8000-000000000004");
    const AUTHORITY_ID: Uuid = uuid::uuid!("6f1b2c3d-6666-4000-8000-000000000005");
    const ADMIN_USER_ID: Uuid = uuid::uuid!("6f1b2c3d-6666-4000-8000-000000000006");
    const CLIENT_KEY: Uuid = uuid::uuid!("6f1b2c3d-6666-4000-8000-000000000007");
    const ENV_CLIENT_KEY: Uuid = uuid::uuid!("6f1b2c3d-6666-4000-8000-000000000008");

    /// Sets/removes multiple env vars under ONE acquisition of the
    /// non-reentrant `ENV_LOCK`; previous values are restored on drop.
    /// (per-var guards deadlock when a test needs two vars at once)
    struct EnvBatch {
        restores: Vec<(&'static str, Option<String>)>,
        _lock: MutexGuard<'static, ()>,
    }

    impl EnvBatch {
        fn apply(batch: &[(&'static str, Option<&str>)]) -> Self {
            let _lock = crate::ENV_LOCK
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let mut restores = Vec::new();

            for (key, value) in batch {
                restores.push((*key, std::env::var(*key).ok()));

                // SAFETY: every test that mutates or reads the process environment
                // does so while holding `ENV_LOCK`.
                match value {
                    Some(value) => unsafe { std::env::set_var(*key, *value) },
                    None => unsafe { std::env::remove_var(*key) },
                }
            }

            Self { restores, _lock }
        }
    }

    impl Drop for EnvBatch {
        fn drop(&mut self) {
            for (key, value) in self.restores.iter().rev() {
                match value {
                    Some(value) => unsafe { std::env::set_var(key, value) },
                    None => unsafe { std::env::remove_var(key) },
                }
            }
        }
    }

    #[derive(Clone, Default)]
    struct Log {
        ops: Arc<Mutex<Vec<String>>>,
    }

    impl Log {
        fn push(&self, op: impl Into<String>) {
            self.ops
                .lock()
                .expect("log")
                .push(op.into());
        }

        fn ops(&self) -> Vec<String> {
            self.ops
                .lock()
                .expect("log")
                .clone()
        }
    }

    #[derive(Clone, Copy, PartialEq)]
    enum State {
        Hit,
        Miss,
        Fail,
    }

    #[derive(Clone, Copy)]
    struct Cfg {
        setting: State,
        public_key: State,
        permission: State,
        role: State,
        role_grant: State,
        authority: State,
        user: State,
        user_role: State,
        create_fails: bool,
    }

    const ALL_PRESENT: Cfg = Cfg {
        setting: State::Hit,
        public_key: State::Hit,
        permission: State::Hit,
        role: State::Hit,
        role_grant: State::Hit,
        authority: State::Hit,
        user: State::Hit,
        user_role: State::Hit,
        create_fails: false,
    };

    const ALL_ABSENT: Cfg = Cfg {
        setting: State::Miss,
        public_key: State::Miss,
        permission: State::Miss,
        role: State::Miss,
        role_grant: State::Miss,
        authority: State::Miss,
        user: State::Miss,
        user_role: State::Miss,
        create_fails: false,
    };

    // ---------------------------------------------------------------- fixtures

    fn admin_permission() -> Permission {
        Permission {
            id: ADMIN_PERM_ID,
            realm: "oxidauth".to_owned(),
            resource: "**".to_owned(),
            action: "**".to_owned(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn totp_permission() -> Permission {
        Permission {
            id: TOTP_PERM_ID,
            realm: "oxidauth".to_owned(),
            resource: "totp_code".to_owned(),
            action: "validate".to_owned(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn admin_role() -> Role {
        Role {
            id: ROLE_ID,
            name: ADMIN_ROLE.to_owned(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn default_authority() -> Authority {
        Authority {
            id: AUTHORITY_ID,
            name: DEFAULT_USERNAMEPASSWORD_NAME.to_owned(),
            client_key: CLIENT_KEY,
            status: AuthorityStatus::Enabled,
            strategy: AuthorityStrategy::UsernamePassword,
            settings: AuthoritySettings {
                jwt_ttl: DEFAULT_JWT_TTL,
                jwt_nbf_offset: NbfOffset::default(),
                refresh_token_ttl: DEFAULT_REFRESH_TOKEN_TTL,
                totp: TotpSettings::Disabled,
                entitlements_encoding: EntitlementsEncoding::Txt,
            },
            params: JsonValue::new(json!({"password_salt": "x"})),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn admin_user() -> User {
        User {
            id: ADMIN_USER_ID,
            kind: UserKind::Api,
            status: UserStatus::Enabled,
            username: DEFAULT_ADMIN_USERNAME.to_owned(),
            email: None,
            first_name: None,
            last_name: None,
            profile: json!({}),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    // ------------------------------------------------------------------ mocks

    struct MockFetchSetting {
        state: State,
        log: Log,
    }

    #[async_trait]
    impl FetchSettingServiceTrait for MockFetchSetting {
        async fn fetch_setting(&self, params: &FetchSettingParams) -> Result<Setting, BoxedError> {
            self.log
                .push(format!("fetch_setting:{}", params.key));

            match self.state {
                State::Hit => {
                    Ok(Setting {
                        key: params.key.clone(),
                        value: Value::Bool(true),
                        created_at: Utc::now(),
                        updated_at: Utc::now(),
                    })
                },
                // production shape: `Err(SettingNotFoundError::new(..))` handed
                // over directly, which is what `downcast_ref::<SettingNotFoundError>`
                // in `check_bootstrap_setting` matches
                State::Miss => Err(SettingNotFoundError::new(&params.key)),
                State::Fail => Err("simulated fetch failure".into()),
            }
        }
    }

    struct MockPublicKeys {
        state: State,
        log: Log,
    }

    #[async_trait]
    impl ListAllPublicKeysServiceTrait for MockPublicKeys {
        async fn list_all_public_keys(
            &self,
            _params: &ListAllPublicKeys,
        ) -> Result<Vec<PublicKey>, BoxedError> {
            self.log
                .push("list_public_keys".to_owned());

            match self.state {
                State::Hit => {
                    Ok(vec![PublicKey {
                        id: PUBKEY_ID,
                        public_key: "pem".to_owned(),
                        created_at: Utc::now(),
                        updated_at: Utc::now(),
                    }])
                },
                State::Miss => Ok(vec![]),
                State::Fail => Err("simulated list_public_keys failure".into()),
            }
        }
    }

    #[async_trait]
    impl CreatePublicKeyServiceTrait for MockPublicKeys {
        async fn create_public_key(
            &self,
            _params: &CreatePublicKey,
        ) -> Result<PublicKey, BoxedError> {
            self.log
                .push("create_public_key".to_owned());

            Ok(PublicKey {
                id: PUBKEY_ID,
                public_key: "pem".to_owned(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })
        }
    }

    struct MockPermissions {
        find: State,
        create_fails: bool,
        log: Log,
    }

    #[async_trait]
    impl FindPermissionByPartsServiceTrait for MockPermissions {
        async fn find_permission_by_parts(
            &self,
            params: &FindPermissionByParts,
        ) -> Result<Permission, BoxedError> {
            self.log
                .push(format!("find_permission:{}", params.permission));

            match self.find {
                State::Hit => {
                    Ok(if params.permission == TOTP_VALIDATE_PERMISSION {
                        totp_permission()
                    } else {
                        admin_permission()
                    })
                },
                // production shape: direct `Err(PermissionNotFoundError::new(..))`
                State::Miss => Err(PermissionNotFoundError::new(&params.permission)),
                State::Fail => Err("simulated find_permission failure".into()),
            }
        }
    }

    #[async_trait]
    impl CreatePermissionServiceTrait for MockPermissions {
        async fn create_permission(
            &self,
            params: &CreatePermission,
        ) -> Result<Permission, BoxedError> {
            self.log
                .push(format!("create_permission:{}", params.permission));

            if self.create_fails {
                return Err("simulated create_permission failure".into());
            }

            Ok(if params.permission == TOTP_VALIDATE_PERMISSION {
                totp_permission()
            } else {
                admin_permission()
            })
        }
    }

    struct MockRoles {
        state: State,
        log: Log,
    }

    #[async_trait]
    impl ListAllRolesServiceTrait for MockRoles {
        async fn list_all_roles(&self, _params: &ListAllRoles) -> Result<Vec<Role>, BoxedError> {
            self.log
                .push("list_roles".to_owned());

            match self.state {
                State::Hit => Ok(vec![admin_role()]),
                State::Miss => Ok(vec![]),
                State::Fail => Err("simulated list_roles failure".into()),
            }
        }
    }

    #[async_trait]
    impl CreateRoleServiceTrait for MockRoles {
        async fn create_role(&self, params: &CreateRole) -> Result<Role, BoxedError> {
            self.log
                .push(format!("create_role:{}", params.name));

            Ok(admin_role())
        }
    }

    struct MockRoleGrants {
        state: State,
        log: Log,
    }

    #[async_trait]
    impl ListRolePermissionGrantsByRoleIdServiceTrait for MockRoleGrants {
        async fn list_role_permission_grants_by_role_id(
            &self,
            params: &ListRolePermissionGrantsByRoleId,
        ) -> Result<Vec<RolePermission>, BoxedError> {
            self.log
                .push(format!("list_role_permission_grants:{}", params.role_id));

            match self.state {
                State::Hit => {
                    Ok(vec![RolePermission {
                        permission: admin_permission(),
                        grant: RolePermissionGrant {
                            role_id: params.role_id,
                            permission_id: ADMIN_PERM_ID,
                            created_at: Utc::now(),
                            updated_at: Utc::now(),
                        },
                    }])
                },
                State::Miss => Ok(vec![]),
                State::Fail => Err("simulated list_role_grants failure".into()),
            }
        }
    }

    #[async_trait]
    impl CreateRolePermissionGrantServiceTrait for MockRoleGrants {
        async fn create_role_permission_grant(
            &self,
            params: &CreateRolePermissionGrant,
        ) -> Result<RolePermission, BoxedError> {
            self.log.push(format!(
                "create_role_permission_grant:{}.{}",
                params.role_id, params.permission
            ));

            Ok(RolePermission {
                permission: admin_permission(),
                grant: RolePermissionGrant {
                    role_id: params.role_id,
                    permission_id: ADMIN_PERM_ID,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                },
            })
        }
    }

    struct MockAuthorityService {
        state: State,
        log: Log,
    }

    #[async_trait]
    impl FindAuthorityByStrategyServiceTrait for MockAuthorityService {
        async fn find_authority_by_strategy(
            &self,
            params: &FindAuthorityByStrategy,
        ) -> Result<Authority, BoxedError> {
            self.log
                .push(format!("find_authority_by_strategy:{}", params.strategy));

            match self.state {
                State::Hit => Ok(default_authority()),
                // production shape (find_authority_by_strategy.rs): the
                // `Box<AuthorityNotFoundError>` produced by the constructor is
                // moved through `?`, whose From impl stores the *boxed* error —
                // so bootstrap's `downcast_ref::<Box<AuthorityNotFoundError>>`
                // only matches this shape, unlike the direct-Err cases
                State::Miss => {
                    authority_missing(&params.strategy)?;
                    unreachable!("the helper always errors")
                },
                State::Fail => Err("simulated find_authority failure".into()),
            }
        }
    }

    fn authority_missing(
        strategy: &AuthorityStrategy,
    ) -> Result<Authority, Box<AuthorityNotFoundError>> {
        Err(AuthorityNotFoundError::strategy(*strategy))
    }

    fn admin_user_missing() -> Result<User, Box<UserNotFoundError>> {
        Err(UserNotFoundError::username(
            &DEFAULT_ADMIN_USERNAME
                .parse()
                .expect("valid username"),
        ))
    }

    #[async_trait]
    impl CreateAuthorityServiceTrait for MockAuthorityService {
        async fn create_authority(
            &self,
            params: &mut CreateAuthority,
        ) -> Result<Authority, BoxedError> {
            let payload = serde_json::to_value(&*params).expect("serializable");
            params.client_key = Some(CLIENT_KEY);

            self.log
                .push(format!("create_authority:{payload}"));

            Ok(default_authority())
        }
    }

    struct MockUserService {
        state: State,
        calls: AtomicUsize,
        log: Log,
    }

    #[async_trait]
    impl FindUserByUsernameServiceTrait for MockUserService {
        async fn find_user_by_username(
            &self,
            params: &FindUserByUsername,
        ) -> Result<User, BoxedError> {
            self.log
                .push(format!("find_user_by_username:{}", params.username));

            match self.state {
                State::Hit => Ok(admin_user()),
                // production shape (users/find_user_by_username.rs): `?`-moved
                State::Miss => {
                    // the first lookup misses (register is triggered), the
                    // re-find after register hits
                    if self
                        .calls
                        .fetch_add(1, Ordering::SeqCst)
                        == 0
                    {
                        admin_user_missing()?;
                    }

                    Ok(admin_user())
                },
                State::Fail => Err("simulated find_user failure".into()),
            }
        }
    }

    #[async_trait]
    impl oxidauth_kernel::auth::register::RegisterServiceTrait for MockUserService {
        async fn register(&self, params: &RegisterParams) -> Result<RegisterResponse, BoxedError> {
            let payload = serde_json::to_value(params).expect("serializable");

            self.log
                .push(format!("register:{payload}"));

            Ok(RegisterResponse {
                jwt: "admin-jwt".to_owned(),
                refresh_token: Uuid::new_v4(),
                user_id: ADMIN_USER_ID,
            })
        }
    }

    struct MockUserRoleGrants {
        state: State,
        log: Log,
    }

    #[async_trait]
    impl ListUserRoleGrantsByUserIdServiceTrait for MockUserRoleGrants {
        async fn list_user_role_grants_by_user_id(
            &self,
            params: &ListUserRoleGrantsByUserId,
        ) -> Result<Vec<UserRole>, BoxedError> {
            self.log
                .push(format!("list_user_role_grants:{}", params.user_id));

            match self.state {
                State::Hit => {
                    Ok(vec![UserRole {
                        role: admin_role(),
                        grant: UserRoleGrant {
                            user_id: params.user_id,
                            role_id: ROLE_ID,
                            created_at: Utc::now(),
                            updated_at: Utc::now(),
                        },
                    }])
                },
                State::Miss => Ok(vec![]),
                State::Fail => Err("simulated list_user_roles failure".into()),
            }
        }
    }

    #[async_trait]
    impl CreateUserRoleGrantServiceTrait for MockUserRoleGrants {
        async fn create_user_role_grant(
            &self,
            params: &CreateUserRoleGrant,
        ) -> Result<UserRole, BoxedError> {
            self.log.push(format!(
                "create_user_role_grant:{}.{}",
                params.user_id, params.role_id
            ));

            Ok(UserRole {
                role: admin_role(),
                grant: UserRoleGrant {
                    user_id: params.user_id,
                    role_id: params.role_id,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                },
            })
        }
    }

    struct MockSaveSetting {
        log: Log,
    }

    #[async_trait]
    impl SaveSettingServiceTrait for MockSaveSetting {
        async fn save_setting(&self, params: &SaveSettingParams) -> Result<Setting, BoxedError> {
            self.log
                .push(format!("save_setting:{}={}", params.key, params.value));

            Ok(Setting {
                key: params.key.clone(),
                value: params.value.clone(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })
        }
    }

    fn provider(cfg: Cfg, log: &Log) -> Provider {
        let mut provider = Provider::new();

        provider.store::<FetchSettingService>(Arc::new(MockFetchSetting {
            state: cfg.setting,
            log: log.clone(),
        }));
        provider.store::<ListAllPublicKeysService>(Arc::new(MockPublicKeys {
            state: cfg.public_key,
            log: log.clone(),
        }));
        provider.store::<CreatePublicKeyService>(Arc::new(MockPublicKeys {
            state: cfg.public_key,
            log: log.clone(),
        }));
        provider.store::<FindPermissionByPartsService>(Arc::new(MockPermissions {
            find: cfg.permission,
            create_fails: cfg.create_fails,
            log: log.clone(),
        }));
        provider.store::<CreatePermissionService>(Arc::new(MockPermissions {
            find: cfg.permission,
            create_fails: cfg.create_fails,
            log: log.clone(),
        }));
        provider.store::<ListAllRolesService>(Arc::new(MockRoles {
            state: cfg.role,
            log: log.clone(),
        }));
        provider.store::<CreateRoleService>(Arc::new(MockRoles {
            state: cfg.role,
            log: log.clone(),
        }));
        provider.store::<ListRolePermissionGrantsByRoleIdService>(Arc::new(MockRoleGrants {
            state: cfg.role_grant,
            log: log.clone(),
        }));
        provider.store::<CreateRolePermissionGrantService>(Arc::new(MockRoleGrants {
            state: cfg.role_grant,
            log: log.clone(),
        }));
        provider.store::<FindAuthorityByStrategyService>(Arc::new(MockAuthorityService {
            state: cfg.authority,
            log: log.clone(),
        }));
        provider.store::<CreateAuthorityService>(Arc::new(MockAuthorityService {
            state: cfg.authority,
            log: log.clone(),
        }));
        provider.store::<FindUserByUsernameService>(Arc::new(MockUserService {
            state: cfg.user,
            calls: AtomicUsize::new(0),
            log: log.clone(),
        }));
        provider.store::<oxidauth_kernel::auth::register::RegisterService>(Arc::new(
            MockUserService {
                state: cfg.user,
                calls: AtomicUsize::new(0),
                log: log.clone(),
            },
        ));
        provider.store::<ListUserRoleGrantsByUserIdService>(Arc::new(MockUserRoleGrants {
            state: cfg.user_role,
            log: log.clone(),
        }));
        provider.store::<CreateUserRoleGrantService>(Arc::new(MockUserRoleGrants {
            state: cfg.user_role,
            log: log.clone(),
        }));
        provider.store::<SaveSettingService>(Arc::new(MockSaveSetting { log: log.clone() }));

        provider
    }

    fn payload_op<'a>(ops: &'a [String], prefix: &str) -> &'a str {
        ops.iter()
            .find(|op| op.starts_with(prefix))
            .map(String::as_str)
            .unwrap_or_else(|| panic!("no op starting with {prefix:?} in {ops:?}"))
    }

    #[tokio::test]
    async fn second_run_short_circuits_with_zero_creation_calls() {
        let log = Log::default();
        let case = SudoUserBootstrapUseCase::new(&provider(ALL_PRESENT, &log));

        case.bootstrap(&BootstrapParams)
            .await
            .expect("a completed bootstrap is a no-op");

        assert_eq!(
            log.ops(),
            vec![format!("fetch_setting:{BOOTSTRAP_SETTING_KEY}")],
            "the setting row is the ONLY gate: nothing else may be touched"
        );
    }

    #[tokio::test]
    async fn setting_probe_error_propagates_without_starting_the_sequence() {
        let log = Log::default();
        let case = SudoUserBootstrapUseCase::new(&provider(
            Cfg {
                setting: State::Fail,
                ..ALL_PRESENT
            },
            &log,
        ));

        let err = case
            .bootstrap(&BootstrapParams)
            .await
            .expect_err("a real (non-not-found) fetch error must propagate");

        assert!(
            err.to_string()
                .contains("simulated fetch failure")
        );
        assert_eq!(log.ops().len(), 1);
    }

    #[tokio::test]
    async fn every_first_or_create_step_skips_creation_when_rows_exist() {
        let log = Log::default();
        let case = SudoUserBootstrapUseCase::new(&provider(
            Cfg {
                setting: State::Miss,
                ..ALL_PRESENT
            },
            &log,
        ));

        case.bootstrap(&BootstrapParams)
            .await
            .expect("a fully-populated database still completes bootstrap");
        assert_eq!(
            log.ops(),
            vec![
                format!("fetch_setting:{BOOTSTRAP_SETTING_KEY}"),
                "list_public_keys".to_owned(),
                format!("find_permission:{TOTP_VALIDATE_PERMISSION}"),
                format!("find_permission:{FORGOT_PASSWORD_PERMISSION}"),
                format!("find_permission:{ADMIN_PERMISSION}"),
                "list_roles".to_owned(),
                format!("list_role_permission_grants:{ROLE_ID}"),
                format!(
                    "find_authority_by_strategy:{username_password}",
                    username_password = AuthorityStrategy::UsernamePassword
                ),
                format!("find_user_by_username:{DEFAULT_ADMIN_USERNAME}"),
                format!("list_user_role_grants:{ADMIN_USER_ID}"),
                format!("save_setting:{BOOTSTRAP_SETTING_KEY}=true"),
            ],
            "hit-paths only: no create/register call anywhere, and the admin is \
             found on the first lookup so the re-find never runs"
        );
    }

    #[tokio::test]
    async fn empty_database_runs_every_creation_step_in_order_with_exact_params() {
        let client_key_env = ENV_CLIENT_KEY.to_string();
        let _env = EnvBatch::apply(&[
            (DEFAULT_CLIENT_KEY, Some(client_key_env.as_str())),
            (DEFAULT_ADMIN_PASSWORD, Some("s3cret-admin-pw")),
        ]);

        let log = Log::default();
        let case = SudoUserBootstrapUseCase::new(&provider(ALL_ABSENT, &log));

        case.bootstrap(&BootstrapParams)
            .await
            .expect("a fresh database must complete bootstrap");

        let ops = log.ops();
        let op_names: Vec<&str> = ops
            .iter()
            .map(|op| {
                op.split(':')
                    .next()
                    .expect("op name")
            })
            .collect();
        assert_eq!(
            op_names,
            vec![
                "fetch_setting",
                "list_public_keys",
                "create_public_key",
                "find_permission",
                "create_permission",
                "find_permission",
                "create_permission",
                "find_permission",
                "create_permission",
                "list_roles",
                "create_role",
                "list_role_permission_grants",
                "create_role_permission_grant",
                "find_authority_by_strategy",
                "create_authority",
                "find_user_by_username",
                "register",
                "find_user_by_username",
                "list_user_role_grants",
                "create_user_role_grant",
                "save_setting",
            ],
            "the full 11-step sequence, including the post-register re-find"
        );

        assert_eq!(
            ops[3],
            format!("find_permission:{TOTP_VALIDATE_PERMISSION}"),
            "the totp permission is resolved (and created) before the admin one"
        );
        assert_eq!(
            ops[4],
            format!("create_permission:{TOTP_VALIDATE_PERMISSION}")
        );
        assert_eq!(
            ops[5],
            format!("find_permission:{FORGOT_PASSWORD_PERMISSION}"),
            "the forgot_password permission is seeded between totp and admin"
        );
        assert_eq!(
            ops[6],
            format!("create_permission:{FORGOT_PASSWORD_PERMISSION}")
        );
        assert_eq!(ops[7], format!("find_permission:{ADMIN_PERMISSION}"));
        assert_eq!(ops[8], format!("create_permission:{ADMIN_PERMISSION}"));
        assert_eq!(ops[10], format!("create_role:{ADMIN_ROLE}"));
        assert_eq!(
            ops[12],
            format!("create_role_permission_grant:{ROLE_ID}.{ADMIN_PERMISSION}"),
            "the admin role is granted the whole-system permission by name"
        );
        assert_eq!(
            ops[19],
            format!("create_user_role_grant:{ADMIN_USER_ID}.{ROLE_ID}"),
            "the registered admin receives the admin role"
        );
        assert_eq!(
            ops[20],
            format!("save_setting:{BOOTSTRAP_SETTING_KEY}=true"),
            "the setting that gates every future boot is saved last"
        );

        let authority: Value = serde_json::from_str(
            payload_op(&ops, "create_authority:").trim_start_matches("create_authority:"),
        )
        .expect("create_authority payload");
        assert_eq!(authority["name"], json!(DEFAULT_USERNAMEPASSWORD_NAME));
        assert_eq!(
            authority["strategy"],
            json!(AuthorityStrategy::UsernamePassword)
        );
        assert_eq!(authority["status"], Value::Null);
        assert_eq!(
            authority["client_key"],
            json!(ENV_CLIENT_KEY.to_string()),
            "OXIDAUTH_DEFAULT_CLIENT_KEY pins the default authority's client key"
        );
        assert_eq!(authority["settings"]["jwt_ttl"]["secs"], json!(120));
        assert_eq!(
            authority["settings"]["refresh_token_ttl"]["secs"],
            json!(172_800)
        );
        assert_eq!(authority["settings"]["totp"], json!("disabled"));
        let salt = authority["params"]["password_salt"]
            .as_str()
            .expect("generated salt");
        assert!(
            salt.len() == 32
                && salt
                    .chars()
                    .all(char::is_alphanumeric),
            "the authority salt is a random_string(), got {salt:?}"
        );

        let register: Value =
            serde_json::from_str(payload_op(&ops, "register:").trim_start_matches("register:"))
                .expect("register payload");
        assert_eq!(
            register["client_key"],
            json!(CLIENT_KEY.to_string()),
            "registration targets the just-created default authority (its client_key)"
        );
        assert_eq!(
            register["params"]["username"],
            json!(DEFAULT_ADMIN_USERNAME)
        );
        assert_eq!(register["params"]["kind"], json!("api"));
        assert_eq!(register["params"]["password"], json!("s3cret-admin-pw"));
        assert_eq!(
            register["params"]["password_confirmation"],
            json!("s3cret-admin-pw")
        );
        assert_eq!(register["params"]["email"], Value::Null);
    }

    #[tokio::test]
    async fn missing_env_vars_fall_back_to_generated_material() {
        let _env = EnvBatch::apply(&[(DEFAULT_CLIENT_KEY, None), (DEFAULT_ADMIN_PASSWORD, None)]);

        let log = Log::default();
        let case = SudoUserBootstrapUseCase::new(&provider(ALL_ABSENT, &log));

        case.bootstrap(&BootstrapParams)
            .await
            .expect("bootstrap must work without the default env vars");

        let ops = log.ops();
        let authority: Value = serde_json::from_str(
            payload_op(&ops, "create_authority:").trim_start_matches("create_authority:"),
        )
        .expect("create_authority payload");
        assert_eq!(
            authority["client_key"],
            Value::Null,
            "an unset OXIDAUTH_DEFAULT_CLIENT_KEY leaves client_key to the create use case"
        );

        let register: Value =
            serde_json::from_str(payload_op(&ops, "register:").trim_start_matches("register:"))
                .expect("register payload");
        let generated = register["params"]["password"]
            .as_str()
            .expect("generated admin password");
        assert!(
            generated.len() == 32
                && generated
                    .chars()
                    .all(char::is_alphanumeric),
            "with the env unset the admin password is a random_string(), got {generated:?}"
        );
        assert_eq!(
            register["params"]["password"],
            json!(generated),
            "the explicit `json!` handoff must keep carrying the raw secret"
        );
        assert_eq!(
            register["params"]["password_confirmation"],
            json!(generated),
            "OXA-000008 guard: a masked handoff would store ****** as the admin password"
        );
    }

    #[tokio::test]
    async fn unparseable_client_key_env_is_silently_dropped() {
        let _env = EnvBatch::apply(&[
            (DEFAULT_CLIENT_KEY, Some("not-a-uuid")),
            (DEFAULT_ADMIN_PASSWORD, Some("pw")),
        ]);

        let log = Log::default();
        let case = SudoUserBootstrapUseCase::new(&provider(ALL_ABSENT, &log));

        case.bootstrap(&BootstrapParams)
            .await
            .expect("a malformed client key does not abort bootstrap");

        let authority: Value = serde_json::from_str(
            payload_op(&log.ops(), "create_authority:").trim_start_matches("create_authority:"),
        )
        .expect("create_authority payload");
        assert_eq!(
            authority["client_key"],
            Value::Null,
            "BUG(pinned): a non-uuid OXIDAUTH_DEFAULT_CLIENT_KEY is swallowed by \
             .ok().flatten() instead of failing the boot"
        );
    }

    #[tokio::test]
    async fn a_single_non_not_found_step_failure_aborts_the_sequence() {
        let _env = EnvBatch::apply(&[
            (DEFAULT_CLIENT_KEY, Some("invalid")),
            (DEFAULT_ADMIN_PASSWORD, Some("pw")),
        ]);

        let cases: Vec<(Cfg, &str, &str)> = vec![
            (
                Cfg {
                    public_key: State::Fail,
                    ..ALL_ABSENT
                },
                "simulated list_public_keys failure",
                "list_public_keys",
            ),
            (
                Cfg {
                    permission: State::Fail,
                    ..ALL_ABSENT
                },
                "simulated find_permission failure",
                "find_permission",
            ),
            (
                Cfg {
                    role: State::Fail,
                    ..ALL_ABSENT
                },
                "simulated list_roles failure",
                "list_roles",
            ),
            (
                Cfg {
                    role_grant: State::Fail,
                    ..ALL_ABSENT
                },
                "simulated list_role_grants failure",
                "list_role_permission_grants",
            ),
            (
                Cfg {
                    authority: State::Fail,
                    ..ALL_ABSENT
                },
                "simulated find_authority failure",
                "find_authority_by_strategy",
            ),
            (
                Cfg {
                    user: State::Fail,
                    ..ALL_ABSENT
                },
                "simulated find_user failure",
                "find_user_by_username",
            ),
            (
                Cfg {
                    user_role: State::Fail,
                    ..ALL_ABSENT
                },
                "simulated list_user_roles failure",
                "list_user_role_grants",
            ),
        ];

        for (cfg, message, stopped_at) in cases {
            let log = Log::default();
            let case = SudoUserBootstrapUseCase::new(&provider(cfg, &log));

            let err = case
                .bootstrap(&BootstrapParams)
                .await
                .expect_err("mid-sequence failures must abort bootstrap");

            assert!(
                err.to_string()
                    .contains(message),
                "got {err}"
            );

            let ops = log.ops();
            assert!(
                ops.last()
                    .is_some_and(|op| op.starts_with(stopped_at)),
                "the sequence must stop AT {stopped_at:?} for {message:?}, got {ops:?}"
            );
            assert!(
                !ops.iter()
                    .any(|op| op.starts_with("save_setting")),
                "a failed sequence must NEVER save the bootstrap-complete setting"
            );
        }
    }

    #[tokio::test]
    async fn a_failing_create_step_aborts_before_later_steps_and_saves_nothing() {
        let log = Log::default();
        let case = SudoUserBootstrapUseCase::new(&provider(
            Cfg {
                create_fails: true,
                ..ALL_ABSENT
            },
            &log,
        ));

        let err = case
            .bootstrap(&BootstrapParams)
            .await
            .expect_err("a failing create_permission must abort the sequence");

        assert!(
            err.to_string()
                .contains("simulated create_permission failure"),
            "got {err}"
        );

        let ops = log.ops();
        // BUG(pinned): first_or_create_permissions stores the totp create result
        // in `_totp_permission` without checking it, so the FIRST create failure
        // is swallowed and only the admin-permission create aborts the sequence
        assert_eq!(
            ops.last().map(String::as_str),
            Some(format!("create_permission:{ADMIN_PERMISSION}").as_str()),
            "the abort lands on the admin-permission create, before any role or \
             user work: got {ops:?}"
        );
        assert!(
            !ops.iter()
                .any(|op| op.starts_with("save_setting")),
            "a failed sequence must NEVER save the bootstrap-complete setting"
        );
    }
}
