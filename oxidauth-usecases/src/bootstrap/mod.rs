use std::{env, time::Duration};

use async_trait::async_trait;
use oxidauth_kernel::{
    JsonValue, Password,
    auth::register::{RegisterParams, RegisterService},
    authorities::{
        Authority, AuthorityNotFoundError, AuthoritySettings, AuthorityStrategy, NbfOffset,
        TotpSettings,
        create_authority::{CreateAuthority, CreateAuthorityService},
        find_authority_by_strategy::{
            FindAuthorityByStrategy, FindAuthorityByStrategyService,
        },
    },
    bootstrap::BootstrapParams,
    error::BoxedError,
    jwt::EntitlementsEncoding,
    permissions::{
        Permission, PermissionNotFoundError,
        create_permission::{CreatePermission, CreatePermissionService},
        find_permission_by_parts::{
            FindPermissionByParts, FindPermissionByPartsService,
        },
    },
    provider::Provider,
    public_keys::{
        PublicKey,
        create_public_key::{CreatePublicKey, CreatePublicKeyService},
        list_all_public_keys::{ListAllPublicKeys, ListAllPublicKeysService},
    },
    role_permission_grants::{
        create_role_permission_grant::{
            CreateRolePermissionGrant, CreateRolePermissionGrantService,
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
    service::Service,
    settings::{
        Setting,
        fetch_setting::{
            FetchSettingParams, FetchSettingService, SettingNotFoundError,
        },
        save_setting::{SaveSettingParams, SaveSettingService},
    },
    user_role_grants::{
        create_user_role_grant::{
            CreateUserRoleGrant, CreateUserRoleGrantService,
        },
        list_user_role_grants_by_user_id::{
            ListUserRoleGrantsByUserId, ListUserRoleGrantsByUserIdService,
        },
    },
    users::{
        User, UserKind, UserNotFoundError,
        find_user_by_username::{
            FindUserByUsername, FindUserByUsernameService,
        },
    },
};
use tracing::{error, info};

use crate::{
    auth::strategies::username_password::{
        AuthorityParams, registrar::UsernamePasswordRegisterParams,
    },
    random_string,
};

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
impl<'a> Service<&'a BootstrapParams> for SudoUserBootstrapUseCase {
    type Response = ();
    type Error = BoxedError;

    #[tracing::instrument(name = "bootstrap_user_usecase", skip(self))]
    async fn call(
        &self,
        params: &'a BootstrapParams,
    ) -> Result<Self::Response, Self::Error> {
        let setting = {
            let fetch_setting = self.provider.fetch();

            check_bootstrap_setting(fetch_setting).await?
        };

        if setting.is_some() {
            info!("bootstrap already completed");

            return Ok(());
        }

        info!("no bootstrap detected -- starting bootstrap");

        {
            let list_all_public_keys = self.provider.fetch();

            let create_public_key = self.provider.fetch();

            first_or_create_public_key(
                list_all_public_keys,
                create_public_key,
            )
            .await?;
        }

        let permission = {
            let permission_by_name = self.provider.fetch();

            let create_permission = self.provider.fetch();

            first_or_create_permissions(
                permission_by_name,
                create_permission,
            )
            .await?
        };

        let role = {
            let list_all_roles = self.provider.fetch();

            let create_role = self.provider.fetch();

            first_or_create_role(list_all_roles, create_role).await?
        };

        {
            let list_role_permission_grants = self.provider.fetch();
            let create_role_permission = self.provider.fetch();

            add_admin_permission_to_admin_role(
                list_role_permission_grants,
                create_role_permission,
                &role,
                &permission,
            )
            .await?;
        }

        let authority = {
            let authority_by_strategy = self.provider.fetch();
            let create_authority = self.provider.fetch();

            first_or_create_authority(
                authority_by_strategy,
                create_authority,
            )
            .await?
        };

        let user = {
            let find_user_by_username = self.provider.fetch();
            let register_user = self.provider.fetch();

            first_or_register_user(
                find_user_by_username,
                register_user,
                &authority,
            )
            .await?
        };

        {
            let list_user_role = self.provider.fetch();
            let create_user_role = self.provider.fetch();

            add_admin_role_to_admin_user(
                list_user_role,
                create_user_role,
                &user,
                &role,
            )
            .await?;
        }

        {
            let save_setting = self.provider.fetch();

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
        .call(&FetchSettingParams {
            key: BOOTSTRAP_SETTING_KEY.to_owned(),
        })
        .await;

    match bootstrap_setting {
        Ok(setting) => Ok(Some(setting)),
        Err(err) => match err.downcast_ref::<SettingNotFoundError>() {
            Some(_) => return Ok(None),
            _ => return Err(err),
        },
    }
}

#[tracing::instrument(skip_all)]
async fn first_or_create_public_key(
    list_all_public_keys: &ListAllPublicKeysService,
    create_public_key: &CreatePublicKeyService,
) -> Result<PublicKey, BoxedError> {
    let mut public_keys = list_all_public_keys
        .call(&ListAllPublicKeys)
        .await?;

    if let Some(public_key) = public_keys.pop() {
        return Ok(public_key);
    }

    create_public_key
        .call(&CreatePublicKey)
        .await
}

pub const ADMIN_PERMISSION: &str = "oxidauth:**:**";
pub const TOTP_VALIDATE_PERMISSION: &str = "oxidauth:totp_code:validate";

#[tracing::instrument(skip_all)]
async fn first_or_create_permissions(
    permission_by_name: &FindPermissionByPartsService,
    create_permission: &CreatePermissionService,
) -> Result<Permission, BoxedError> {
    // TODO(dewey4iv): currently not returning but might want to later

    let permission_name = TOTP_VALIDATE_PERMISSION.to_owned();

    let permission = permission_by_name
        .call(&FindPermissionByParts {
            permission: permission_name.clone(),
        })
        .await;

    let _totp_permission = match permission {
        Ok(permission) => Ok(permission),
        Err(err) => match err.downcast_ref::<PermissionNotFoundError>() {
            Some(_) => {
                create_permission
                    .call(&CreatePermission {
                        permission: permission_name,
                    })
                    .await
            },
            None => return Err(err),
        },
    };

    let permission_name = ADMIN_PERMISSION.to_owned();

    let permission = permission_by_name
        .call(&FindPermissionByParts {
            permission: permission_name.clone(),
        })
        .await;

    match permission {
        Ok(permission) => Ok(permission),
        Err(err) => match err.downcast_ref::<PermissionNotFoundError>() {
            Some(_) => {
                create_permission
                    .call(&CreatePermission {
                        permission: permission_name,
                    })
                    .await
            },
            None => return Err(err),
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
        .call(&ListAllRoles)
        .await?;

    let admin_role = roles
        .into_iter()
        .find(|role| role.name == ADMIN_ROLE);

    if let Some(admin_role) = admin_role {
        return Ok(admin_role);
    }

    create_role
        .call(&CreateRole {
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
        .call(&ListRolePermissionGrantsByRoleId { role_id: role.id })
        .await?;

    let admin_role_permission = role_permissions
        .into_iter()
        .find(|rp| rp.permission.id == permission.id);

    if admin_role_permission.is_some() {
        return Ok(());
    }

    create_role_permission
        .call(&CreateRolePermissionGrant {
            role_id: role.id,
            permission: ADMIN_PERMISSION.to_owned(),
        })
        .await?;

    Ok(())
}

pub const DEFAULT_JWT_TTL: Duration = Duration::from_secs(60 * 2);
pub const DEFAULT_TOTP_TOKEN_TTL: Duration = Duration::from_secs(60 * 2);
pub const DEFAULT_CLIENT_KEY: &str = "OXIDAUTH_DEFAULT_CLIENT_KEY";
pub const DEFAULT_REFRESH_TOKEN_TTL: Duration =
    Duration::from_secs(60 * 60 * 24 * 2);
pub const DEFAULT_USERNAMEPASSWORD_NAME: &str =
    "oxidauth default username_password";

#[tracing::instrument(skip_all)]
async fn first_or_create_authority(
    authority_by_strategy: &FindAuthorityByStrategyService,
    create_authority: &CreateAuthorityService,
) -> Result<Authority, BoxedError> {
    // TODO(dewey4iv): we should swap this for something
    // that can pull the authority by the name
    let authority = authority_by_strategy
        .call(&FindAuthorityByStrategy {
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
                        AuthorityParams::new(random_string())
                            .as_json_value()?;

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
                        .call(&mut create_authority_params)
                        .await
                },
                _ => {
                    error!(
                        message = "authority could not be found or created",
                        ?err
                    );
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
        .call(&FindUserByUsername {
            username: DEFAULT_ADMIN_USERNAME.parse()?,
        })
        .await;

    match user {
        Ok(user) => return Ok(user),
        Err(err) => match err.downcast_ref::<Box<UserNotFoundError>>() {
            Some(_) => {
                let password = env::var(DEFAULT_ADMIN_PASSWORD)
                    .unwrap_or_else(|_| random_string());

                println!(
                    ":::\nDEFAULT ADMIN PASSWORD: {}\n:::",
                    password
                );

                let username_password_params = UsernamePasswordRegisterParams {
                    username: DEFAULT_ADMIN_USERNAME.to_owned(),
                    password: Password::new(password.clone()),
                    password_confirmation: Password::new(password),
                    email: None,
                    first_name: None,
                    last_name: None,
                    kind: Some(UserKind::Api),
                }
                .to_value()?;

                let register_params = RegisterParams {
                    client_key: authority.client_key,
                    params: JsonValue::new(username_password_params),
                };

                register_user
                    .call(&register_params)
                    .await?;

                let registered = find_user_by_username
                    .call(&FindUserByUsername {
                        username: DEFAULT_ADMIN_USERNAME.parse()?,
                    })
                    .await?;

                return Ok(registered);
            },

            None => return Err(err),
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
        .call(&ListUserRoleGrantsByUserId { user_id: user.id })
        .await?;

    let user_role_grant = user_role_grants
        .into_iter()
        .find(|grant| grant.role.id == role.id);

    if user_role_grant.is_some() {
        return Ok(());
    }

    create_user_role
        .call(&CreateUserRoleGrant {
            user_id: user.id,
            role_id: role.id,
        })
        .await?;

    Ok(())
}

#[tracing::instrument(skip_all)]
async fn save_bootstrap_setting(
    save_setting: &SaveSettingService,
) -> Result<(), BoxedError> {
    save_setting
        .call(&SaveSettingParams {
            key: BOOTSTRAP_SETTING_KEY.to_owned(),
            value: serde_json::Value::Bool(true),
        })
        .await?;

    Ok(())
}
