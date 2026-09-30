use std::sync::Arc;

use oxidauth_kernel::error::BoxedError;
use oxidauth_postgres::{
    Database,
    auth::PgAuthRepository,
    authorities::PgAuthorityRepository,
    invitations::PgInvitationRepository,
    permissions::PgPermissionRepository,
    private_keys::PgPrivateKeyRepository,
    public_keys::PgPublicKeyRepository,
    refresh_tokens::PgRefreshTokenRepository,
    role_permission_grants::PgRolePermissionGrantRepository,
    role_role_grants::PgRoleRoleGrantRepository,
    roles::PgRoleRepository,
    settings::PgSettingRepository,
    totp_secrets::PgTotpSecretRepository,
    user_authorities::PgUserAuthorityRepository,
    user_permission_grants::PgUserPermissionGrantRepository,
    user_role_grants::PgUserRoleGrantRepository,
    users::PgUserRepository,
};
use provider::Provider;

pub async fn init(provider: &mut Provider) -> Result<(), BoxedError> {
    let db = provider
        .fetch::<Database>()?
        .clone();

    // region auth
    let register_service = {
        use oxidauth_kernel::auth::register::RegisterService;
        use oxidauth_services::auth::register::RegisterUseCase;

        let register_service = Arc::new(RegisterUseCase::new(
            PgAuthorityRepository::new(db.clone()),
            PgUserRepository::new(db.clone()),
            PgUserAuthorityRepository::new(db.clone()),
            PgAuthRepository::new(db.clone()),
            PgPrivateKeyRepository::new(db.clone()),
            PgRefreshTokenRepository::new(db.clone()),
        ));
        provider.store::<RegisterService>(register_service.clone());

        register_service
    };

    let authenticate_service = {
        use oxidauth_kernel::auth::authenticate::AuthenticateService;
        use oxidauth_services::auth::authenticate::AuthenticateUseCase;

        let authenticate_service = Arc::new(AuthenticateUseCase::new(
            PgAuthorityRepository::new(db.clone()),
            PgUserAuthorityRepository::new(db.clone()),
            PgAuthRepository::new(db.clone()),
            PgPrivateKeyRepository::new(db.clone()),
            PgRefreshTokenRepository::new(db.clone()),
            PgTotpSecretRepository::new(db.clone()),
            PgUserRepository::new(db.clone()),
        ));

        provider.store::<AuthenticateService>(authenticate_service.clone());

        authenticate_service
    };

    {
        use oxidauth_kernel::auth::authenticate_or_register::AuthenticateOrRegisterService;
        use oxidauth_services::auth::authenticate_or_register::AuthenticateOrRegisterUseCase;

        let authenticate_or_register_service = Arc::new(AuthenticateOrRegisterUseCase::new(
            authenticate_service,
            register_service,
            PgAuthorityRepository::new(db.clone()),
        ));

        provider.store::<AuthenticateOrRegisterService>(authenticate_or_register_service);
    }

    {
        use oxidauth_kernel::auth::oauth2::redirect::Oauth2RedirectService;
        use oxidauth_services::auth::strategies::oauth2::redirect::Oauth2RedirectUseCase;

        let oauth2_redirect_service = Arc::new(Oauth2RedirectUseCase::new(
            PgAuthorityRepository::new(db.clone()),
        ));

        provider.store::<Oauth2RedirectService>(oauth2_redirect_service);
    }

    {
        use oxidauth_kernel::auth::username_password::forgot_password::ForgotPasswordService;
        use oxidauth_services::auth::strategies::username_password::forgot_password::ForgotPasswordUseCase;

        let forgot_password_service = Arc::new(ForgotPasswordUseCase::new(
            PgRefreshTokenRepository::new(db.clone()),
            PgTotpSecretRepository::new(db.clone()),
        ));

        provider.store::<ForgotPasswordService>(forgot_password_service);
    }

    {
        use oxidauth_kernel::auth::username_password::update_password::UpdatePasswordService;
        use oxidauth_services::auth::strategies::username_password::update_password::UpdatePasswordUseCase;

        let update_password_service = Arc::new(UpdatePasswordUseCase::new(
            PgTotpSecretRepository::new(db.clone()),
            PgAuthorityRepository::new(db.clone()),
            PgUserAuthorityRepository::new(db.clone()),
            PgUserAuthorityRepository::new(db.clone()),
            // OXA-000009: recovery status gate
            PgUserRepository::new(db.clone()),
        ));

        provider.store::<UpdatePasswordService>(update_password_service);
    }

    // region totp
    let create_totp_secret_service = {
        use oxidauth_kernel::totp_secrets::create_totp_secret::CreateTotpSecretService;
        use oxidauth_services::totp_secrets::create_totp_secret::CreateTotpSecretUseCase;

        let create_totp_secret_service = Arc::new(CreateTotpSecretUseCase::new(
            PgTotpSecretRepository::new(db.clone()),
        ));

        provider.store::<CreateTotpSecretService>(create_totp_secret_service.clone());

        create_totp_secret_service
    };

    {
        use oxidauth_kernel::totp::validate::ValidateTOTPService;
        use oxidauth_services::totp::validate::ValidateTOTPUseCase;

        let validate_totp_service = Arc::new(ValidateTOTPUseCase::new(
            PgTotpSecretRepository::new(db.clone()),
            PgPrivateKeyRepository::new(db.clone()),
            PgAuthRepository::new(db.clone()),
            PgAuthorityRepository::new(db.clone()),
            PgRefreshTokenRepository::new(db.clone()),
            // OXA-000009: post-2FA status gate
            PgUserRepository::new(db.clone()),
        ));
        provider.store::<ValidateTOTPService>(validate_totp_service);
    }

    // region users
    let create_user_service = {
        use oxidauth_kernel::users::create_user::CreateUserService;
        use oxidauth_services::users::create_user::CreateUserUseCase;

        let create_user_service = CreateUserUseCase::new(PgUserRepository::new(db.clone()));

        provider.store::<CreateUserService>(Arc::new(create_user_service.clone()));

        create_user_service
    };

    let update_user_service = {
        use oxidauth_kernel::users::update_user::UpdateUserService;
        use oxidauth_services::users::update_user::UpdateUserUseCase;

        let update_user_service = Arc::new(UpdateUserUseCase::new(
            PgUserRepository::new(db.clone()),
            PgUserRepository::new(db.clone()),
        ));
        provider.store::<UpdateUserService>(update_user_service.clone());

        update_user_service
    };

    {
        use oxidauth_kernel::users::find_user_by_id::FindUserByIdService;
        use oxidauth_services::users::find_user_by_id::FindUserByIdUseCase;

        let find_user_by_id_service =
            Arc::new(FindUserByIdUseCase::new(PgUserRepository::new(db.clone())));
        provider.store::<FindUserByIdService>(find_user_by_id_service);
    }

    {
        use oxidauth_kernel::users::find_users_by_ids::FindUsersByIdsService;
        use oxidauth_services::users::find_users_by_ids::FindUsersByIdsUseCase;

        let find_users_by_ids_service = Arc::new(FindUsersByIdsUseCase::new(
            PgUserRepository::new(db.clone()),
        ));
        provider.store::<FindUsersByIdsService>(find_users_by_ids_service);
    }

    {
        use oxidauth_kernel::users::delete_user_by_id::DeleteUserByIdService;
        use oxidauth_services::users::delete_user_by_id::DeleteUserByIdUseCase;

        let delete_user_by_id_service = Arc::new(DeleteUserByIdUseCase::new(
            PgUserRepository::new(db.clone()),
        ));
        provider.store::<DeleteUserByIdService>(delete_user_by_id_service);
    }

    {
        use oxidauth_kernel::users::find_user_by_username::FindUserByUsernameService;
        use oxidauth_services::users::find_user_by_username::FindUserByUsernameUseCase;

        let find_user_by_username_service = Arc::new(FindUserByUsernameUseCase::new(
            PgUserRepository::new(db.clone()),
        ));
        provider.store::<FindUserByUsernameService>(find_user_by_username_service);
    }

    {
        use oxidauth_kernel::users::list_all_users::ListAllUsersService;
        use oxidauth_services::users::list_all_users::ListAllUsersUseCase;

        let list_all_users_service =
            Arc::new(ListAllUsersUseCase::new(PgUserRepository::new(db.clone())));
        provider.store::<ListAllUsersService>(list_all_users_service);
    }

    // region user authorities
    let create_user_authority_service = {
        use oxidauth_kernel::user_authorities::create_user_authority::CreateUserAuthorityService;
        use oxidauth_services::user_authorities::create_user_authority::CreateUserAuthorityUseCase;

        let create_user_authority_service = Arc::new(CreateUserAuthorityUseCase::new(
            PgAuthorityRepository::new(db.clone()),
            PgUserAuthorityRepository::new(db.clone()),
            create_totp_secret_service,
        ));
        provider.store::<CreateUserAuthorityService>(create_user_authority_service.clone());

        create_user_authority_service
    };

    // region permissions
    {
        use oxidauth_kernel::permissions::create_permission::CreatePermissionService;
        use oxidauth_services::permissions::create_permission::CreatePermissionUseCase;

        let create_permission_service = Arc::new(CreatePermissionUseCase::new(
            PgPermissionRepository::new(db.clone()),
        ));
        provider.store::<CreatePermissionService>(create_permission_service);
    }

    {
        use oxidauth_kernel::permissions::find_permission_by_parts::FindPermissionByPartsService;
        use oxidauth_services::permissions::find_permission_by_parts::FindPermissionByPartsUseCase;

        let find_permission_by_parts_service = Arc::new(FindPermissionByPartsUseCase::new(
            PgPermissionRepository::new(db.clone()),
        ));
        provider.store::<FindPermissionByPartsService>(find_permission_by_parts_service);
    }

    {
        use oxidauth_kernel::permissions::list_all_permissions::ListAllPermissionsService;
        use oxidauth_services::permissions::list_all_permissions::ListAllPermissionsUseCase;

        let list_all_permissions_service = Arc::new(ListAllPermissionsUseCase::new(
            PgPermissionRepository::new(db.clone()),
        ));
        provider.store::<ListAllPermissionsService>(list_all_permissions_service);
    }

    {
        use oxidauth_kernel::permissions::delete_permission::DeletePermissionService;
        use oxidauth_services::permissions::delete_permission::DeletePermissionUseCase;

        let delete_permission_service = Arc::new(DeletePermissionUseCase::new(
            PgPermissionRepository::new(db.clone()),
        ));
        provider.store::<DeletePermissionService>(delete_permission_service);
    }

    // region user permission grants
    {
        use oxidauth_kernel::user_permission_grants::create_user_permission_grant::CreateUserPermissionGrantService;
        use oxidauth_services::user_permission_grants::create_user_permission_grant::CreateUserPermissionGrantUseCase;

        let create_user_permission_grant_service = Arc::new(CreateUserPermissionGrantUseCase::new(
            PgUserRepository::new(db.clone()),
            PgPermissionRepository::new(db.clone()),
            PgUserPermissionGrantRepository::new(db.clone()),
        ));
        provider.store::<CreateUserPermissionGrantService>(create_user_permission_grant_service);
    }

    {
        use oxidauth_kernel::user_permission_grants::list_user_permission_grants_by_user_id::ListUserPermissionGrantsByUserIdService;
        use oxidauth_services::user_permission_grants::list_user_permission_grants_by_user_id::ListUserPermissionGrantsByUserIdUseCase;

        let list_user_permission_grants_by_user_id_service =
            Arc::new(ListUserPermissionGrantsByUserIdUseCase::new(
                PgUserPermissionGrantRepository::new(db.clone()),
            ));
        provider.store::<ListUserPermissionGrantsByUserIdService>(
            list_user_permission_grants_by_user_id_service,
        );
    }

    {
        use oxidauth_kernel::user_permission_grants::delete_user_permission_grant::DeleteUserPermissionGrantService;
        use oxidauth_services::user_permission_grants::delete_user_permission_grant::DeleteUserPermissionGrantUseCase;

        let delete_user_permission_grant_service = Arc::new(DeleteUserPermissionGrantUseCase::new(
            PgUserRepository::new(db.clone()),
            PgPermissionRepository::new(db.clone()),
            PgUserPermissionGrantRepository::new(db.clone()),
        ));
        provider.store::<DeleteUserPermissionGrantService>(delete_user_permission_grant_service);
    }

    // region user authorities (cont.)
    {
        use oxidauth_kernel::user_authorities::update_user_authority::UpdateUserAuthorityService;
        use oxidauth_services::user_authorities::update_user_authority::UpdateUserAuthorityUseCase;

        let update_user_authority_service = Arc::new(UpdateUserAuthorityUseCase::new(
            PgUserAuthorityRepository::new(db.clone()),
        ));
        provider.store::<UpdateUserAuthorityService>(update_user_authority_service);
    }

    {
        use oxidauth_kernel::user_authorities::delete_user_authority::DeleteUserAuthorityService;
        use oxidauth_services::user_authorities::delete_user_authority::DeleteUserAuthorityUseCase;

        let delete_user_authority_service = Arc::new(DeleteUserAuthorityUseCase::new(
            PgUserAuthorityRepository::new(db.clone()),
        ));
        provider.store::<DeleteUserAuthorityService>(delete_user_authority_service);
    }

    {
        use oxidauth_kernel::user_authorities::find_user_authority_by_user_id_and_authority_id::FindUserAuthorityByUserIdAndAuthorityIdService;
        use oxidauth_services::user_authorities::find_user_authority_by_user_id_and_authority_id::FindUserAuthorityByUserIdAndAuthorityIdUseCase;

        let find_user_authority_by_user_id_and_authority_id_service =
            Arc::new(FindUserAuthorityByUserIdAndAuthorityIdUseCase::new(
                PgUserAuthorityRepository::new(db.clone()),
            ));
        provider.store::<FindUserAuthorityByUserIdAndAuthorityIdService>(
            find_user_authority_by_user_id_and_authority_id_service,
        );
    }

    {
        use oxidauth_kernel::user_authorities::list_user_authorities_by_user_id::ListUserAuthoritiesByUserIdService;
        use oxidauth_services::user_authorities::list_user_authorities_by_user_id::ListUserAuthoritiesByUserIdUseCase;

        let list_user_authorities_by_user_id_service = Arc::new(
            ListUserAuthoritiesByUserIdUseCase::new(PgUserAuthorityRepository::new(db.clone())),
        );
        provider
            .store::<ListUserAuthoritiesByUserIdService>(list_user_authorities_by_user_id_service);
    }

    // region roles
    {
        use oxidauth_kernel::roles::create_role::CreateRoleService;
        use oxidauth_services::roles::create_role::CreateRoleUseCase;

        let create_role_service =
            Arc::new(CreateRoleUseCase::new(PgRoleRepository::new(db.clone())));
        provider.store::<CreateRoleService>(create_role_service);
    }

    {
        use oxidauth_kernel::roles::find_role_by_id::FindRoleByIdService;
        use oxidauth_services::roles::find_role_by_id::FindRoleByIdUseCase;

        let find_role_by_id_service =
            Arc::new(FindRoleByIdUseCase::new(PgRoleRepository::new(db.clone())));
        provider.store::<FindRoleByIdService>(find_role_by_id_service);
    }

    {
        use oxidauth_kernel::roles::find_role_by_name::FindRoleByNameService;
        use oxidauth_services::roles::find_role_by_name::FindRoleByNameUseCase;

        let find_role_by_name_service = Arc::new(FindRoleByNameUseCase::new(
            PgRoleRepository::new(db.clone()),
        ));
        provider.store::<FindRoleByNameService>(find_role_by_name_service);
    }

    {
        use oxidauth_kernel::roles::list_all_roles::ListAllRolesService;
        use oxidauth_services::roles::list_all_roles::ListAllRolesUseCase;

        let list_all_roles_service =
            Arc::new(ListAllRolesUseCase::new(PgRoleRepository::new(db.clone())));
        provider.store::<ListAllRolesService>(list_all_roles_service);
    }

    {
        use oxidauth_kernel::roles::update_role::UpdateRoleService;
        use oxidauth_services::roles::update_role::UpdateRoleUseCase;

        let update_role_service =
            Arc::new(UpdateRoleUseCase::new(PgRoleRepository::new(db.clone())));
        provider.store::<UpdateRoleService>(update_role_service);
    }

    {
        use oxidauth_kernel::roles::delete_role::DeleteRoleService;
        use oxidauth_services::roles::delete_role::DeleteRoleUseCase;

        let delete_role_service =
            Arc::new(DeleteRoleUseCase::new(PgRoleRepository::new(db.clone())));
        provider.store::<DeleteRoleService>(delete_role_service);
    }

    // region user role grants
    {
        use oxidauth_kernel::user_role_grants::create_user_role_grant::CreateUserRoleGrantService;
        use oxidauth_services::user_role_grants::create_user_role_grant::CreateUserRoleGrantUseCase;

        let create_user_role_service = Arc::new(CreateUserRoleGrantUseCase::new(
            PgUserRepository::new(db.clone()),
            PgRoleRepository::new(db.clone()),
            PgUserRoleGrantRepository::new(db.clone()),
        ));
        provider.store::<CreateUserRoleGrantService>(create_user_role_service);
    }

    {
        use oxidauth_kernel::user_role_grants::delete_user_role_grant::DeleteUserRoleGrantService;
        use oxidauth_services::user_role_grants::delete_user_role_grant::DeleteUserRoleGrantUseCase;

        let delete_user_role_service = Arc::new(DeleteUserRoleGrantUseCase::new(
            PgUserRepository::new(db.clone()),
            PgRoleRepository::new(db.clone()),
            PgUserRoleGrantRepository::new(db.clone()),
        ));
        provider.store::<DeleteUserRoleGrantService>(delete_user_role_service);
    }

    {
        use oxidauth_kernel::user_role_grants::list_user_role_grants_by_user_id::ListUserRoleGrantsByUserIdService;
        use oxidauth_services::user_role_grants::list_user_role_grants_by_user_id::ListUserRoleGrantsByUserIdUseCase;

        let list_user_role_grants_by_user_id_service = Arc::new(
            ListUserRoleGrantsByUserIdUseCase::new(PgUserRoleGrantRepository::new(db.clone())),
        );
        provider
            .store::<ListUserRoleGrantsByUserIdService>(list_user_role_grants_by_user_id_service);
    }

    // region role role grants
    {
        use oxidauth_kernel::role_role_grants::create_role_role_grant::CreateRoleRoleGrantService;
        use oxidauth_services::role_role_grants::create_role_role_grant::CreateRoleRoleGrantUseCase;

        let create_role_role_grant_service = Arc::new(CreateRoleRoleGrantUseCase::new(
            PgRoleRoleGrantRepository::new(db.clone()),
            PgRoleRepository::new(db.clone()),
        ));
        provider.store::<CreateRoleRoleGrantService>(create_role_role_grant_service);
    }

    {
        use oxidauth_kernel::role_role_grants::delete_role_role_grant::DeleteRoleRoleGrantService;
        use oxidauth_services::role_role_grants::delete_role_role_grant::DeleteRoleRoleGrantUseCase;

        let delete_role_role_grant_service = Arc::new(DeleteRoleRoleGrantUseCase::new(
            PgRoleRoleGrantRepository::new(db.clone()),
        ));
        provider.store::<DeleteRoleRoleGrantService>(delete_role_role_grant_service);
    }
    {
        use oxidauth_kernel::role_role_grants::list_role_role_grants_by_parent_id::ListRoleRoleGrantsByParentIdService;
        use oxidauth_services::role_role_grants::list_role_role_grants_by_parent_id::ListRoleRoleGrantsByParentIdUseCase;

        let list_role_role_grants_by_parent_id_service = Arc::new(
            ListRoleRoleGrantsByParentIdUseCase::new(PgRoleRoleGrantRepository::new(db.clone())),
        );
        provider.store::<ListRoleRoleGrantsByParentIdService>(
            list_role_role_grants_by_parent_id_service,
        );
    }

    // region role permission grants
    {
        use oxidauth_kernel::role_permission_grants::create_role_permission_grant::CreateRolePermissionGrantService;
        use oxidauth_services::role_permission_grants::create_role_permission_grant::CreateRolePermissionGrantUseCase;

        let create_role_permission_grant_service = Arc::new(CreateRolePermissionGrantUseCase::new(
            PgRolePermissionGrantRepository::new(db.clone()),
            PgRoleRepository::new(db.clone()),
            PgPermissionRepository::new(db.clone()),
        ));
        provider.store::<CreateRolePermissionGrantService>(create_role_permission_grant_service);
    }

    {
        use oxidauth_kernel::role_permission_grants::list_role_permission_grants_by_role_id::ListRolePermissionGrantsByRoleIdService;
        use oxidauth_services::role_permission_grants::list_role_permission_grants_by_role_id::ListRolePermissionGrantsByRoleIdUseCase;

        let list_role_permission_grants_by_role_id_service =
            Arc::new(ListRolePermissionGrantsByRoleIdUseCase::new(
                PgRolePermissionGrantRepository::new(db.clone()),
            ));
        provider.store::<ListRolePermissionGrantsByRoleIdService>(
            list_role_permission_grants_by_role_id_service,
        );
    }

    {
        use oxidauth_kernel::role_permission_grants::delete_role_permission_grant::DeleteRolePermissionGrantService;
        use oxidauth_services::role_permission_grants::delete_role_permission_grant::DeleteRolePermissionGrantUseCase;

        let delete_role_permission_grant_service = Arc::new(DeleteRolePermissionGrantUseCase::new(
            PgRolePermissionGrantRepository::new(db.clone()),
            PgRoleRepository::new(db.clone()),
            PgPermissionRepository::new(db.clone()),
        ));
        provider.store::<DeleteRolePermissionGrantService>(delete_role_permission_grant_service);
    }

    // region authorities
    {
        use oxidauth_kernel::authorities::create_authority::CreateAuthorityService;
        use oxidauth_services::authorities::create_authority::CreateAuthorityUseCase;

        let create_authority_service = Arc::new(CreateAuthorityUseCase::new(
            PgAuthorityRepository::new(db.clone()),
        ));
        provider.store::<CreateAuthorityService>(create_authority_service);
    }

    {
        use oxidauth_kernel::authorities::find_authority_by_id::FindAuthorityByIdService;
        use oxidauth_services::authorities::find_authority_by_id::FindAuthorityByIdUseCase;

        let find_authority_by_id_service = Arc::new(FindAuthorityByIdUseCase::new(
            PgAuthorityRepository::new(db.clone()),
        ));
        provider.store::<FindAuthorityByIdService>(find_authority_by_id_service);
    }

    {
        use oxidauth_kernel::authorities::delete_authority::DeleteAuthorityService;
        use oxidauth_services::authorities::delete_authority::DeleteAuthorityUseCase;

        let delete_authority_service = Arc::new(DeleteAuthorityUseCase::new(
            PgAuthorityRepository::new(db.clone()),
        ));
        provider.store::<DeleteAuthorityService>(delete_authority_service);
    }

    {
        use oxidauth_kernel::authorities::list_all_authorities::ListAllAuthoritiesService;
        use oxidauth_services::authorities::list_all_authorities::ListAllAuthoritiesUseCase;

        let list_all_authorities_service = Arc::new(ListAllAuthoritiesUseCase::new(
            PgAuthorityRepository::new(db.clone()),
        ));
        provider.store::<ListAllAuthoritiesService>(list_all_authorities_service);
    }

    // region public keys
    {
        use oxidauth_kernel::public_keys::find_public_key_by_id::FindPublicKeyByIdService;
        use oxidauth_services::public_keys::find_public_key_by_id::FindPublicKeyByIdUseCase;

        let find_public_key_by_id_service = Arc::new(FindPublicKeyByIdUseCase::new(
            PgPublicKeyRepository::new(db.clone()),
        ));
        provider.store::<FindPublicKeyByIdService>(find_public_key_by_id_service);
    }

    {
        use oxidauth_kernel::public_keys::list_all_public_keys::ListAllPublicKeysService;
        use oxidauth_services::public_keys::list_all_public_keys::ListAllPublicKeysUseCase;

        let list_all_public_keys_service = Arc::new(ListAllPublicKeysUseCase::new(
            PgPublicKeyRepository::new(db.clone()),
        ));
        provider.store::<ListAllPublicKeysService>(list_all_public_keys_service);
    }

    // region totp secrets
    let create_totp_secrets_service = {
        use oxidauth_kernel::totp_secrets::create_totp_secrets_by_authority_id::CreateTotpSecretsService;
        use oxidauth_services::totp_secrets::create_totp_secrets_by_authority_id::CreateTotpSecretsByAuthorityIdUseCase;

        let create_totp_secrets_service = Arc::new(CreateTotpSecretsByAuthorityIdUseCase::new(
            PgTotpSecretRepository::new(db.clone()),
            PgTotpSecretRepository::new(db.clone()),
        ));

        provider.store::<CreateTotpSecretsService>(create_totp_secrets_service.clone());

        create_totp_secrets_service
    };

    // region authorities (cont.)
    {
        use oxidauth_kernel::authorities::update_authority::UpdateAuthorityService;
        use oxidauth_services::authorities::update_authority::UpdateAuthorityUseCase;

        let update_authority_service = Arc::new(UpdateAuthorityUseCase::new(
            PgAuthorityRepository::new(db.clone()),
            PgAuthorityRepository::new(db.clone()),
            create_totp_secrets_service,
        ));
        provider.store::<UpdateAuthorityService>(update_authority_service);
    }

    {
        use oxidauth_kernel::authorities::find_authority_by_strategy::FindAuthorityByStrategyService;
        use oxidauth_services::authorities::find_authority_by_strategy::FindAuthorityByStrategyUseCase;

        let find_authority_by_strategy_service = Arc::new(FindAuthorityByStrategyUseCase::new(
            PgAuthorityRepository::new(db.clone()),
        ));
        provider.store::<FindAuthorityByStrategyService>(find_authority_by_strategy_service);
    }

    // region public keys (cont.)
    {
        use oxidauth_kernel::public_keys::create_public_key::CreatePublicKeyService;
        use oxidauth_services::public_keys::create_public_key::CreatePublicKeyUseCase;

        let create_public_key_service = Arc::new(CreatePublicKeyUseCase::new(
            PgPublicKeyRepository::new(db.clone()),
        ));
        provider.store::<CreatePublicKeyService>(create_public_key_service);
    }

    {
        use oxidauth_kernel::public_keys::delete_public_key::DeletePublicKeyService;
        use oxidauth_services::public_keys::delete_public_key::DeletePublicKeyUseCase;

        let delete_public_key_service = Arc::new(DeletePublicKeyUseCase::new(
            PgPublicKeyRepository::new(db.clone()),
        ));
        provider.store::<DeletePublicKeyService>(delete_public_key_service);
    }

    // region refresh tokens
    {
        use oxidauth_kernel::refresh_tokens::exchange_refresh_token::ExchangeRefreshTokenService;
        use oxidauth_services::refresh_tokens::exchange_refresh_token::ExchangeRefreshTokenUseCase;

        let exchange_refresh_token_service = Arc::new(ExchangeRefreshTokenUseCase::new(
            PgRefreshTokenRepository::new(db.clone()),
            PgRefreshTokenRepository::new(db.clone()),
            PgUserAuthorityRepository::new(db.clone()),
            PgAuthorityRepository::new(db.clone()),
            PgAuthRepository::new(db.clone()),
            PgPrivateKeyRepository::new(db.clone()),
            PgRefreshTokenRepository::new(db.clone()),
            // OXA-000009: refresh status gate
            PgUserRepository::new(db.clone()),
        ));
        provider.store::<ExchangeRefreshTokenService>(exchange_refresh_token_service);
    }

    // region settings
    {
        use oxidauth_kernel::settings::save_setting::SaveSettingService;
        use oxidauth_services::settings::save_setting::SaveSettingUseCase;

        let save_setting_service = Arc::new(SaveSettingUseCase::new(PgSettingRepository::new(
            db.clone(),
        )));
        provider.store::<SaveSettingService>(save_setting_service);
    }

    {
        use oxidauth_kernel::settings::fetch_setting::FetchSettingService;
        use oxidauth_services::settings::fetch_setting::FetchSettingUseCase;

        let fetch_setting_service = Arc::new(FetchSettingUseCase::new(PgSettingRepository::new(
            db.clone(),
        )));
        provider.store::<FetchSettingService>(fetch_setting_service);
    }

    // region invitations
    {
        use oxidauth_kernel::invitations::create_invitation::CreateInvitationService;
        use oxidauth_services::invitations::create_invitation::CreateInvitationUseCase;

        let create_invitation_service = Arc::new(CreateInvitationUseCase::new(
            PgInvitationRepository::new(db.clone()),
            create_user_service,
        ));
        provider.store::<CreateInvitationService>(create_invitation_service);
    }

    {
        use oxidauth_kernel::invitations::find_invitation::FindInvitationService;
        use oxidauth_services::invitations::find_invitation::FindInvitationUseCase;

        let find_invitation_service = Arc::new(FindInvitationUseCase::new(
            PgInvitationRepository::new(db.clone()),
        ));
        provider.store::<FindInvitationService>(find_invitation_service);
    }

    {
        use oxidauth_kernel::invitations::delete_invitation::DeleteInvitationService;
        use oxidauth_services::invitations::delete_invitation::DeleteInvitationUseCase;

        let delete_invitation_service = Arc::new(DeleteInvitationUseCase::new(
            PgInvitationRepository::new(db.clone()),
        ));
        provider.store::<DeleteInvitationService>(delete_invitation_service);
    }

    {
        use oxidauth_kernel::invitations::accept_invitation::AcceptInvitationService;
        use oxidauth_services::invitations::accept_invitation::AcceptInvitationUseCase;

        let accept_invitation_service = Arc::new(AcceptInvitationUseCase::new(
            update_user_service,
            create_user_authority_service,
            PgInvitationRepository::new(db.clone()),
        ));
        provider.store::<AcceptInvitationService>(accept_invitation_service);
    }

    Ok(())
}
