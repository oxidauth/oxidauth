use oxidauth_kernel::error::BoxedError;
use std::sync::Arc;
use uuid::Uuid;

// Users
use oxidauth_http::server::api::v1::users::create_user::{
    CreateUserReq, CreateUserRes,
};
use oxidauth_http::server::api::v1::users::delete_user_by_id::DeleteUserByIdRes;
use oxidauth_http::server::api::v1::users::find_user_by_id::{
    FindUserByIdReq, FindUserByIdRes,
};
use oxidauth_http::server::api::v1::users::find_user_by_username::FindUserByUsernameRes;
use oxidauth_http::server::api::v1::users::find_users_by_ids::{
    FindUsersByIdsReq, FindUsersByIdsRes,
};
use oxidauth_http::server::api::v1::users::list_all_users::{
    ListAllUsersReq, ListAllUsersRes,
};
use oxidauth_http::server::api::v1::users::update_user::{
    UpdateUserBodyReq, UpdateUserRes,
};

// User Authorities
use oxidauth_http::server::api::v1::users::authorities::create_user_authority::{CreateUserAuthorityBodyReq, CreateUserAuthorityRes};
    use oxidauth_http::server::api::v1::users::authorities::delete_user_authority::{DeleteUserAuthorityReq, DeleteUserAuthorityRes};
    use oxidauth_http::server::api::v1::users::authorities::find_user_authority_by_user_id_and_authority_id::{FindUserAuthorityByUserIdAndAuthorityIdReq, FindUserAuthorityByUserIdAndAuthorityIdRes};
    use oxidauth_http::server::api::v1::users::authorities::list_user_authorities_by_user_id::{ListUserAuthoritiesByUserIdReq, ListUserAuthoritiesByUserIdRes};
    use oxidauth_http::server::api::v1::users::authorities::update_user_authority::UpdateUserAuthorityRes;
    use oxidauth_kernel::user_authorities::update_user_authority::UpdateUserAuthority;

// User Permissions
use oxidauth_http::server::api::v1::users::permissions::create_user_permission::{CreateUserPermissionReq, CreateUserPermissionRes};
    use oxidauth_http::server::api::v1::users::permissions::delete_user_permission::{DeleteUserPermissionReq, DeleteUserPermissionRes};
    use oxidauth_http::server::api::v1::users::permissions::list_user_permissions_by_user_id::{ListUserPermissionGrantsByUserIdReq, ListUserPermissionGrantsByUserIdRes};

// User Roles
use crate::client::users::roles::create_user_role::CreateUserRole;
use oxidauth_http::server::api::v1::users::roles::create_user_role::CreateUserRoleRes;
use oxidauth_http::server::api::v1::users::roles::delete_user_role::DeleteUserRoleRes;
use oxidauth_http::server::api::v1::users::roles::list_user_roles_by_user_id::ListUserRoleGrantsByUserIdRes;

// Auth
use oxidauth_http::server::api::v1::auth::register::{
    RegisterReq, RegisterRes,
};

// Can
use oxidauth_http::server::api::v1::can::CanReq;

// Authorities
use oxidauth_http::server::api::v1::authorities::create_authority::{
    CreateAuthorityReq, CreateAuthorityRes,
};
use oxidauth_http::server::api::v1::authorities::delete_authority::DeleteAuthorityRes;
use oxidauth_http::server::api::v1::authorities::find_authority_by_id::FindAuthorityByIdRes;
use oxidauth_http::server::api::v1::authorities::find_authority_by_strategy::FindAuthorityByStrategyRes;
use oxidauth_http::server::api::v1::authorities::list_all_authorities::{
    ListAllAuthoritiesReq, ListAllAuthoritiesRes,
};
use oxidauth_http::server::api::v1::authorities::update_authority::{
    UpdateAuthorityReq, UpdateAuthorityRes,
};

// Permissions
use oxidauth_http::server::api::v1::permissions::create_permission::{
    CreatePermissionReq, CreatePermissionRes,
};
use oxidauth_http::server::api::v1::permissions::delete_permission::{
    DeletePermissionReq, DeletePermissionRes,
};
use oxidauth_http::server::api::v1::permissions::find_permission_by_parts::{
    FindPermissionByPartsReq, FindPermissionByPartsRes,
};
use oxidauth_http::server::api::v1::permissions::list_all_permissions::{
    ListAllPermissionsReq, ListAllPermissionsRes,
};

// Roles
use oxidauth_http::server::api::v1::roles::create_role::{
    CreateRoleReq, CreateRoleRes,
};
use oxidauth_http::server::api::v1::roles::delete_role::DeleteRoleRes;
use oxidauth_http::server::api::v1::roles::find_role_by_id::FindRoleByIdRes;
use oxidauth_http::server::api::v1::roles::find_role_by_name::FindRoleByNameRes;
use oxidauth_http::server::api::v1::roles::list_all_roles::{
    ListAllRolesReq, ListAllRolesRes,
};
use oxidauth_http::server::api::v1::roles::update_role::{
    UpdateRoleReq, UpdateRoleRes,
};

// Role Permission Grants
use oxidauth_http::server::api::v1::roles::permissions::create_role_permission_grant::{CreateRolePermissionGrantReq, CreateRolePermissionGrantRes};
    use oxidauth_http::server::api::v1::roles::permissions::delete_role_permission_grant::{DeleteRolePermissionGrantReq, DeleteRolePermissionGrantRes};
    use oxidauth_http::server::api::v1::roles::permissions::list_role_permission_grants_by_role_id::{ListRolePermissionGrantsByRoleIdReq, ListRolePermissionGrantsByRoleIdRes};

// Role Role Grants
use oxidauth_http::server::api::v1::roles::roles::create_role_role_grant::{CreateRoleRoleGrantReq, CreateRoleRoleGrantRes};
    use oxidauth_http::server::api::v1::roles::roles::delete_role_role_grant::{DeleteRoleRoleGrantReq, DeleteRoleRoleGrantRes};
    use oxidauth_http::server::api::v1::roles::roles::list_role_role_grants_by_parent_id::{ListRoleRoleGrantsByParentIdReq, ListRoleRoleGrantsByParentIdRes};

// Public Keys
use oxidauth_http::server::api::v1::public_keys::create_public_key::CreatePublicKeyRes;
use oxidauth_http::server::api::v1::public_keys::delete_public_key::DeletePublicKeyRes;
use oxidauth_http::server::api::v1::public_keys::find_public_key_by_id::FindPublicKeyByIdRes;
use oxidauth_http::server::api::v1::public_keys::list_all_public_keys::ListAllPublicKeysRes;

// Refresh Tokens
use oxidauth_http::server::api::v1::refresh_tokens::exchange::{
    ExchangeRefreshTokenReq, ExchangeRefreshTokenRes,
};

// Settings
use oxidauth_http::server::api::v1::settings::fetch_setting::{
    FetchSettingReq, FetchSettingRes,
};
use oxidauth_http::server::api::v1::settings::save_setting::{
    SaveSettingReq, SaveSettingRes,
};

// Invitations
use oxidauth_http::server::api::v1::invitations::accept_invitation::{
    AcceptInvitationParams, AcceptInvitationRes,
};
use oxidauth_http::server::api::v1::invitations::create_invitation::{
    CreateInvitationReq, CreateInvitationRes,
};
use oxidauth_http::server::api::v1::invitations::find_invitation::{
    FindInvitationReq, FindInvitationRes,
};

#[derive(Default)]
pub struct ClientMock {
    // Users
    pub list_all_users_fn: Option<
        Arc<
            dyn Fn(ListAllUsersReq) -> Result<ListAllUsersRes, BoxedError>
                + Send
                + Sync,
        >,
    >,
    pub create_user_fn: Option<
        Arc<
            dyn Fn(CreateUserReq) -> Result<CreateUserRes, BoxedError>
                + Send
                + Sync,
        >,
    >,
    pub delete_user_fn: Option<
        Arc<
            dyn Fn(Uuid) -> Result<DeleteUserByIdRes, BoxedError> + Send + Sync,
        >,
    >,
    pub find_user_by_id_fn: Option<
        Arc<
            dyn Fn(FindUserByIdReq) -> Result<FindUserByIdRes, BoxedError>
                + Send
                + Sync,
        >,
    >,
    pub find_user_by_username_fn: Option<
        Arc<
            dyn Fn(String) -> Result<FindUserByUsernameRes, BoxedError>
                + Send
                + Sync,
        >,
    >,
    pub find_users_by_ids_fn: Option<
        Arc<
            dyn Fn(FindUsersByIdsReq) -> Result<FindUsersByIdsRes, BoxedError>
                + Send
                + Sync,
        >,
    >,
    pub update_user_fn: Option<
        Arc<
            dyn Fn(Uuid, UpdateUserBodyReq) -> Result<UpdateUserRes, BoxedError>
                + Send
                + Sync,
        >,
    >,

    // User Authorities
    pub create_user_authority_fn: Option<
        Arc<
            dyn Fn(
                    Uuid,
                    CreateUserAuthorityBodyReq,
                )
                    -> Result<CreateUserAuthorityRes, BoxedError>
                + Send
                + Sync,
        >,
    >,
    pub delete_user_authority_fn: Option<
        Arc<
            dyn Fn(
                    DeleteUserAuthorityReq,
                )
                    -> Result<DeleteUserAuthorityRes, BoxedError>
                + Send
                + Sync,
        >,
    >,
    pub find_user_authority_by_user_id_and_authority_id_fn: Option<
        Arc<
            dyn Fn(
                    FindUserAuthorityByUserIdAndAuthorityIdReq,
                ) -> Result<
                    FindUserAuthorityByUserIdAndAuthorityIdRes,
                    BoxedError,
                > + Send
                + Sync,
        >,
    >,
    pub list_user_authorities_by_user_id_fn: Option<
        Arc<
            dyn Fn(
                    ListUserAuthoritiesByUserIdReq,
                )
                    -> Result<ListUserAuthoritiesByUserIdRes, BoxedError>
                + Send
                + Sync,
        >,
    >,
    pub update_user_authority_fn: Option<
        Arc<
            dyn Fn(
                    UpdateUserAuthority,
                )
                    -> Result<UpdateUserAuthorityRes, BoxedError>
                + Send
                + Sync,
        >,
    >,

    // User Permissions
    pub create_user_permission_grant_fn: Option<
        Arc<
            dyn Fn(
                    CreateUserPermissionReq,
                )
                    -> Result<CreateUserPermissionRes, BoxedError>
                + Send
                + Sync,
        >,
    >,
    pub delete_user_permission_grant_fn: Option<
        Arc<
            dyn Fn(
                    DeleteUserPermissionReq,
                )
                    -> Result<DeleteUserPermissionRes, BoxedError>
                + Send
                + Sync,
        >,
    >,
    pub list_user_permission_grants_by_user_id_fn: Option<
        Arc<
            dyn Fn(
                    ListUserPermissionGrantsByUserIdReq,
                )
                    -> Result<ListUserPermissionGrantsByUserIdRes, BoxedError>
                + Send
                + Sync,
        >,
    >,

    // User Roles
    pub create_user_role_fn: Option<
        Arc<
            dyn Fn(CreateUserRole) -> Result<CreateUserRoleRes, BoxedError>
                + Send
                + Sync,
        >,
    >,
    pub delete_user_role_fn: Option<
        Arc<
            dyn Fn(Uuid, Uuid) -> Result<DeleteUserRoleRes, BoxedError>
                + Send
                + Sync,
        >,
    >,
    pub list_user_roles_by_user_id_fn: Option<
        Arc<
            dyn Fn(Uuid) -> Result<ListUserRoleGrantsByUserIdRes, BoxedError>
                + Send
                + Sync,
        >,
    >,

    // Auth
    pub authenticate_fn:
        Option<Arc<dyn Fn() -> Result<bool, BoxedError> + Send + Sync>>,
    pub register_fn: Option<
        Arc<
            dyn Fn(RegisterReq) -> Result<RegisterRes, BoxedError>
                + Send
                + Sync,
        >,
    >,

    // Can
    pub can_fn:
        Option<Arc<dyn Fn(CanReq) -> Result<bool, BoxedError> + Send + Sync>>,

    // Authorities
    pub create_authority_fn: Option<
        Arc<
            dyn Fn(CreateAuthorityReq) -> Result<CreateAuthorityRes, BoxedError>
                + Send
                + Sync,
        >,
    >,
    pub delete_authority_fn: Option<
        Arc<
            dyn Fn(Uuid) -> Result<DeleteAuthorityRes, BoxedError>
                + Send
                + Sync,
        >,
    >,
    pub find_authority_by_id_fn: Option<
        Arc<
            dyn Fn(Uuid) -> Result<FindAuthorityByIdRes, BoxedError>
                + Send
                + Sync,
        >,
    >,
    pub find_authority_by_strategy_fn: Option<
        Arc<
            dyn Fn(String) -> Result<FindAuthorityByStrategyRes, BoxedError>
                + Send
                + Sync,
        >,
    >,
    pub list_all_authorities_fn: Option<
        Arc<
            dyn Fn(
                    ListAllAuthoritiesReq,
                ) -> Result<ListAllAuthoritiesRes, BoxedError>
                + Send
                + Sync,
        >,
    >,
    pub update_authority_fn: Option<
        Arc<
            dyn Fn(
                    Uuid,
                    UpdateAuthorityReq,
                ) -> Result<UpdateAuthorityRes, BoxedError>
                + Send
                + Sync,
        >,
    >,

    // Permissions
    pub create_permission_fn: Option<
        Arc<
            dyn Fn(
                    CreatePermissionReq,
                ) -> Result<CreatePermissionRes, BoxedError>
                + Send
                + Sync,
        >,
    >,
    pub delete_permission_fn: Option<
        Arc<
            dyn Fn(
                    DeletePermissionReq,
                ) -> Result<DeletePermissionRes, BoxedError>
                + Send
                + Sync,
        >,
    >,
    pub find_permission_by_parts_fn: Option<
        Arc<
            dyn Fn(
                    FindPermissionByPartsReq,
                )
                    -> Result<FindPermissionByPartsRes, BoxedError>
                + Send
                + Sync,
        >,
    >,
    pub list_all_permissions_fn: Option<
        Arc<
            dyn Fn(
                    ListAllPermissionsReq,
                ) -> Result<ListAllPermissionsRes, BoxedError>
                + Send
                + Sync,
        >,
    >,

    // Roles
    pub create_role_fn: Option<
        Arc<
            dyn Fn(CreateRoleReq) -> Result<CreateRoleRes, BoxedError>
                + Send
                + Sync,
        >,
    >,
    pub delete_role_fn: Option<
        Arc<dyn Fn(Uuid) -> Result<DeleteRoleRes, BoxedError> + Send + Sync>,
    >,
    pub find_role_by_id_fn: Option<
        Arc<dyn Fn(Uuid) -> Result<FindRoleByIdRes, BoxedError> + Send + Sync>,
    >,
    pub find_role_by_name_fn: Option<
        Arc<
            dyn Fn(String) -> Result<FindRoleByNameRes, BoxedError>
                + Send
                + Sync,
        >,
    >,
    pub list_all_roles_fn: Option<
        Arc<
            dyn Fn(ListAllRolesReq) -> Result<ListAllRolesRes, BoxedError>
                + Send
                + Sync,
        >,
    >,
    pub update_role_fn: Option<
        Arc<
            dyn Fn(Uuid, UpdateRoleReq) -> Result<UpdateRoleRes, BoxedError>
                + Send
                + Sync,
        >,
    >,

    // Role Permission Grants
    pub create_role_permission_grant_fn: Option<
        Arc<
            dyn Fn(
                    CreateRolePermissionGrantReq,
                )
                    -> Result<CreateRolePermissionGrantRes, BoxedError>
                + Send
                + Sync,
        >,
    >,
    pub delete_role_permission_grant_fn: Option<
        Arc<
            dyn Fn(
                    DeleteRolePermissionGrantReq,
                )
                    -> Result<DeleteRolePermissionGrantRes, BoxedError>
                + Send
                + Sync,
        >,
    >,
    pub list_role_permission_grants_by_role_id_fn: Option<
        Arc<
            dyn Fn(
                    ListRolePermissionGrantsByRoleIdReq,
                )
                    -> Result<ListRolePermissionGrantsByRoleIdRes, BoxedError>
                + Send
                + Sync,
        >,
    >,

    // Role Role Grants
    pub create_role_role_grant_fn: Option<
        Arc<
            dyn Fn(
                    CreateRoleRoleGrantReq,
                )
                    -> Result<CreateRoleRoleGrantRes, BoxedError>
                + Send
                + Sync,
        >,
    >,
    pub delete_role_role_grant_fn: Option<
        Arc<
            dyn Fn(
                    DeleteRoleRoleGrantReq,
                )
                    -> Result<DeleteRoleRoleGrantRes, BoxedError>
                + Send
                + Sync,
        >,
    >,
    pub list_role_role_grants_by_parent_id_fn: Option<
        Arc<
            dyn Fn(
                    ListRoleRoleGrantsByParentIdReq,
                )
                    -> Result<ListRoleRoleGrantsByParentIdRes, BoxedError>
                + Send
                + Sync,
        >,
    >,

    // Public Keys
    pub create_public_key_fn: Option<
        Arc<dyn Fn() -> Result<CreatePublicKeyRes, BoxedError> + Send + Sync>,
    >,
    pub delete_public_key_fn: Option<
        Arc<
            dyn Fn(Uuid) -> Result<DeletePublicKeyRes, BoxedError>
                + Send
                + Sync,
        >,
    >,
    pub find_public_key_by_id_fn: Option<
        Arc<
            dyn Fn(Uuid) -> Result<FindPublicKeyByIdRes, BoxedError>
                + Send
                + Sync,
        >,
    >,
    pub list_all_public_keys_fn: Option<
        Arc<dyn Fn() -> Result<ListAllPublicKeysRes, BoxedError> + Send + Sync>,
    >,

    // Refresh Tokens
    pub exchange_refresh_token_fn: Option<
        Arc<
            dyn Fn(
                    ExchangeRefreshTokenReq,
                )
                    -> Result<ExchangeRefreshTokenRes, BoxedError>
                + Send
                + Sync,
        >,
    >,

    // Settings
    pub fetch_setting_fn: Option<
        Arc<
            dyn Fn(FetchSettingReq) -> Result<FetchSettingRes, BoxedError>
                + Send
                + Sync,
        >,
    >,
    pub save_setting_fn: Option<
        Arc<
            dyn Fn(SaveSettingReq) -> Result<SaveSettingRes, BoxedError>
                + Send
                + Sync,
        >,
    >,

    // Invitations
    pub accept_invitation_fn: Option<
        Arc<
            dyn Fn(
                    AcceptInvitationParams,
                ) -> Result<AcceptInvitationRes, BoxedError>
                + Send
                + Sync,
        >,
    >,
    pub create_invitation_fn: Option<
        Arc<
            dyn Fn(
                    CreateInvitationReq,
                ) -> Result<CreateInvitationRes, BoxedError>
                + Send
                + Sync,
        >,
    >,
    pub find_invitation_fn: Option<
        Arc<
            dyn Fn(FindInvitationReq) -> Result<FindInvitationRes, BoxedError>
                + Send
                + Sync,
        >,
    >,
}
