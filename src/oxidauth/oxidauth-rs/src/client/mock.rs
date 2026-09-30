use std::sync::Arc;

// Auth
use oxidauth_http::auth::register::{RegisterReq, RegisterRes};
// Authorities
use oxidauth_http::authorities::create_authority::{CreateAuthorityReq, CreateAuthorityRes};
// Can
use oxidauth_http::can::CanReq;
// Invitations
use oxidauth_http::invitations::accept_invitation::AcceptInvitationRes;
// Permissions
use oxidauth_http::permissions::create_permission::{CreatePermissionReq, CreatePermissionRes};
// Public Keys
use oxidauth_http::public_keys::create_public_key::CreatePublicKeyRes;
// Refresh Tokens
use oxidauth_http::refresh_tokens::exchange::{ExchangeRefreshTokenReq, ExchangeRefreshTokenRes};
// Roles
use oxidauth_http::roles::create_role::{CreateRoleReq, CreateRoleRes};
// Role Permission Grants
use oxidauth_http::roles::permissions::create_role_permission_grant::{
    CreateRolePermissionGrantReq,
    CreateRolePermissionGrantRes,
};
// Role Role Grants
use oxidauth_http::roles::roles::create_role_role_grant::{
    CreateRoleRoleGrantReq,
    CreateRoleRoleGrantRes,
};
// Settings
use oxidauth_http::settings::fetch_setting::{FetchSettingReq, FetchSettingRes};
// User Authorities
use oxidauth_http::users::authorities::create_user_authority::{
    CreateUserAuthorityBodyReq,
    CreateUserAuthorityRes,
};
// Users
use oxidauth_http::users::create_user::{CreateUserReq, CreateUserRes};
// User Permissions
use oxidauth_http::users::permissions::create_user_permission::{
    CreateUserPermissionReq,
    CreateUserPermissionRes,
};
use oxidauth_http::{
    authorities::{
        delete_authority::DeleteAuthorityRes,
        find_authority_by_id::FindAuthorityByIdRes,
        find_authority_by_strategy::FindAuthorityByStrategyRes,
        list_all_authorities::{ListAllAuthoritiesReq, ListAllAuthoritiesRes},
        update_authority::{UpdateAuthorityReq, UpdateAuthorityRes},
    },
    invitations::{
        create_invitation::{CreateInvitationReq, CreateInvitationRes},
        find_invitation::{FindInvitationReq, FindInvitationRes},
    },
    permissions::{
        delete_permission::{DeletePermissionReq, DeletePermissionRes},
        find_permission_by_parts::{FindPermissionByPartsReq, FindPermissionByPartsRes},
        list_all_permissions::{ListAllPermissionsReq, ListAllPermissionsRes},
    },
    public_keys::{
        delete_public_key::DeletePublicKeyRes,
        find_public_key_by_id::FindPublicKeyByIdRes,
        list_all_public_keys::ListAllPublicKeysRes,
    },
    roles::{
        delete_role::DeleteRoleRes,
        find_role_by_id::FindRoleByIdRes,
        find_role_by_name::FindRoleByNameRes,
        list_all_roles::{ListAllRolesReq, ListAllRolesRes},
        permissions::{
            delete_role_permission_grant::{
                DeleteRolePermissionGrantReq,
                DeleteRolePermissionGrantRes,
            },
            list_role_permission_grants_by_role_id::{
                ListRolePermissionGrantsByRoleIdReq,
                ListRolePermissionGrantsByRoleIdRes,
            },
        },
        roles::{
            delete_role_role_grant::{DeleteRoleRoleGrantReq, DeleteRoleRoleGrantRes},
            list_role_role_grants_by_parent_id::{
                ListRoleRoleGrantsByParentIdReq,
                ListRoleRoleGrantsByParentIdRes,
            },
        },
        update_role::{UpdateRoleReq, UpdateRoleRes},
    },
    settings::save_setting::{SaveSettingReq, SaveSettingRes},
    users::{
        authorities::{
            delete_user_authority::{DeleteUserAuthorityReq, DeleteUserAuthorityRes},
            find_user_authority_by_user_id_and_authority_id::{
                FindUserAuthorityByUserIdAndAuthorityIdReq,
                FindUserAuthorityByUserIdAndAuthorityIdRes,
            },
            list_user_authorities_by_user_id::{
                ListUserAuthoritiesByUserIdReq,
                ListUserAuthoritiesByUserIdRes,
            },
            update_user_authority::UpdateUserAuthorityRes,
        },
        delete_user_by_id::DeleteUserByIdRes,
        find_user_by_id::{FindUserByIdReq, FindUserByIdRes},
        find_user_by_username::FindUserByUsernameRes,
        find_users_by_ids::{FindUsersByIdsReq, FindUsersByIdsRes},
        list_all_users::{ListAllUsersReq, ListAllUsersRes},
        permissions::{
            delete_user_permission::{DeleteUserPermissionReq, DeleteUserPermissionRes},
            list_user_permissions_by_user_id::{
                ListUserPermissionGrantsByUserIdReq,
                ListUserPermissionGrantsByUserIdRes,
            },
        },
        roles::{
            create_user_role::CreateUserRoleRes,
            delete_user_role::DeleteUserRoleRes,
            list_user_roles_by_user_id::ListUserRoleGrantsByUserIdRes,
        },
        update_user::{UpdateUserBodyReq, UpdateUserRes},
    },
};
use oxidauth_kernel::{
    error::BoxedError,
    invitations::accept_invitation::AcceptInvitationParams,
    user_authorities::update_user_authority::UpdateUserAuthority,
};
use uuid::Uuid;

// User Roles
use crate::client::users::roles::create_user_role::CreateUserRole;

#[derive(Default)]
pub struct ClientMock {
    // Users
    pub list_all_users_fn:
        Option<Arc<dyn Fn(ListAllUsersReq) -> Result<ListAllUsersRes, BoxedError> + Send + Sync>>,
    pub create_user_fn:
        Option<Arc<dyn Fn(CreateUserReq) -> Result<CreateUserRes, BoxedError> + Send + Sync>>,
    pub delete_user_fn:
        Option<Arc<dyn Fn(Uuid) -> Result<DeleteUserByIdRes, BoxedError> + Send + Sync>>,
    pub find_user_by_id_fn:
        Option<Arc<dyn Fn(FindUserByIdReq) -> Result<FindUserByIdRes, BoxedError> + Send + Sync>>,
    pub find_user_by_username_fn:
        Option<Arc<dyn Fn(String) -> Result<FindUserByUsernameRes, BoxedError> + Send + Sync>>,
    pub find_users_by_ids_fn: Option<
        Arc<dyn Fn(FindUsersByIdsReq) -> Result<FindUsersByIdsRes, BoxedError> + Send + Sync>,
    >,
    pub update_user_fn: Option<
        Arc<dyn Fn(Uuid, UpdateUserBodyReq) -> Result<UpdateUserRes, BoxedError> + Send + Sync>,
    >,

    // User Authorities
    pub create_user_authority_fn: Option<
        Arc<
            dyn Fn(Uuid, CreateUserAuthorityBodyReq) -> Result<CreateUserAuthorityRes, BoxedError>
                + Send
                + Sync,
        >,
    >,
    pub delete_user_authority_fn: Option<
        Arc<
            dyn Fn(DeleteUserAuthorityReq) -> Result<DeleteUserAuthorityRes, BoxedError>
                + Send
                + Sync,
        >,
    >,
    pub find_user_authority_by_user_id_and_authority_id_fn: Option<
        Arc<
            dyn Fn(
                    FindUserAuthorityByUserIdAndAuthorityIdReq,
                )
                    -> Result<FindUserAuthorityByUserIdAndAuthorityIdRes, BoxedError>
                + Send
                + Sync,
        >,
    >,
    pub list_user_authorities_by_user_id_fn: Option<
        Arc<
            dyn Fn(
                    ListUserAuthoritiesByUserIdReq,
                ) -> Result<ListUserAuthoritiesByUserIdRes, BoxedError>
                + Send
                + Sync,
        >,
    >,
    pub update_user_authority_fn: Option<
        Arc<
            dyn Fn(UpdateUserAuthority) -> Result<UpdateUserAuthorityRes, BoxedError> + Send + Sync,
        >,
    >,

    // User Permissions
    pub create_user_permission_grant_fn: Option<
        Arc<
            dyn Fn(CreateUserPermissionReq) -> Result<CreateUserPermissionRes, BoxedError>
                + Send
                + Sync,
        >,
    >,
    pub delete_user_permission_grant_fn: Option<
        Arc<
            dyn Fn(DeleteUserPermissionReq) -> Result<DeleteUserPermissionRes, BoxedError>
                + Send
                + Sync,
        >,
    >,
    pub list_user_permission_grants_by_user_id_fn: Option<
        Arc<
            dyn Fn(
                    ListUserPermissionGrantsByUserIdReq,
                ) -> Result<ListUserPermissionGrantsByUserIdRes, BoxedError>
                + Send
                + Sync,
        >,
    >,

    // User Roles
    pub create_user_role_fn:
        Option<Arc<dyn Fn(CreateUserRole) -> Result<CreateUserRoleRes, BoxedError> + Send + Sync>>,
    pub delete_user_role_fn:
        Option<Arc<dyn Fn(Uuid, Uuid) -> Result<DeleteUserRoleRes, BoxedError> + Send + Sync>>,
    pub list_user_roles_by_user_id_fn: Option<
        Arc<dyn Fn(Uuid) -> Result<ListUserRoleGrantsByUserIdRes, BoxedError> + Send + Sync>,
    >,

    // Auth
    pub authenticate_fn: Option<Arc<dyn Fn() -> Result<bool, BoxedError> + Send + Sync>>,
    pub register_fn:
        Option<Arc<dyn Fn(RegisterReq) -> Result<RegisterRes, BoxedError> + Send + Sync>>,

    // Can
    pub can_fn: Option<Arc<dyn Fn(CanReq) -> Result<bool, BoxedError> + Send + Sync>>,

    // Authorities
    pub create_authority_fn: Option<
        Arc<dyn Fn(CreateAuthorityReq) -> Result<CreateAuthorityRes, BoxedError> + Send + Sync>,
    >,
    pub delete_authority_fn:
        Option<Arc<dyn Fn(Uuid) -> Result<DeleteAuthorityRes, BoxedError> + Send + Sync>>,
    pub find_authority_by_id_fn:
        Option<Arc<dyn Fn(Uuid) -> Result<FindAuthorityByIdRes, BoxedError> + Send + Sync>>,
    pub find_authority_by_strategy_fn:
        Option<Arc<dyn Fn(String) -> Result<FindAuthorityByStrategyRes, BoxedError> + Send + Sync>>,
    pub list_all_authorities_fn: Option<
        Arc<
            dyn Fn(ListAllAuthoritiesReq) -> Result<ListAllAuthoritiesRes, BoxedError>
                + Send
                + Sync,
        >,
    >,
    pub update_authority_fn: Option<
        Arc<
            dyn Fn(Uuid, UpdateAuthorityReq) -> Result<UpdateAuthorityRes, BoxedError>
                + Send
                + Sync,
        >,
    >,

    // Permissions
    pub create_permission_fn: Option<
        Arc<dyn Fn(CreatePermissionReq) -> Result<CreatePermissionRes, BoxedError> + Send + Sync>,
    >,
    pub delete_permission_fn: Option<
        Arc<dyn Fn(DeletePermissionReq) -> Result<DeletePermissionRes, BoxedError> + Send + Sync>,
    >,
    pub find_permission_by_parts_fn: Option<
        Arc<
            dyn Fn(FindPermissionByPartsReq) -> Result<FindPermissionByPartsRes, BoxedError>
                + Send
                + Sync,
        >,
    >,
    pub list_all_permissions_fn: Option<
        Arc<
            dyn Fn(ListAllPermissionsReq) -> Result<ListAllPermissionsRes, BoxedError>
                + Send
                + Sync,
        >,
    >,

    // Roles
    pub create_role_fn:
        Option<Arc<dyn Fn(CreateRoleReq) -> Result<CreateRoleRes, BoxedError> + Send + Sync>>,
    pub delete_role_fn:
        Option<Arc<dyn Fn(Uuid) -> Result<DeleteRoleRes, BoxedError> + Send + Sync>>,
    pub find_role_by_id_fn:
        Option<Arc<dyn Fn(Uuid) -> Result<FindRoleByIdRes, BoxedError> + Send + Sync>>,
    pub find_role_by_name_fn:
        Option<Arc<dyn Fn(String) -> Result<FindRoleByNameRes, BoxedError> + Send + Sync>>,
    pub list_all_roles_fn:
        Option<Arc<dyn Fn(ListAllRolesReq) -> Result<ListAllRolesRes, BoxedError> + Send + Sync>>,
    pub update_role_fn:
        Option<Arc<dyn Fn(Uuid, UpdateRoleReq) -> Result<UpdateRoleRes, BoxedError> + Send + Sync>>,

    // Role Permission Grants
    pub create_role_permission_grant_fn: Option<
        Arc<
            dyn Fn(CreateRolePermissionGrantReq) -> Result<CreateRolePermissionGrantRes, BoxedError>
                + Send
                + Sync,
        >,
    >,
    pub delete_role_permission_grant_fn: Option<
        Arc<
            dyn Fn(DeleteRolePermissionGrantReq) -> Result<DeleteRolePermissionGrantRes, BoxedError>
                + Send
                + Sync,
        >,
    >,
    pub list_role_permission_grants_by_role_id_fn: Option<
        Arc<
            dyn Fn(
                    ListRolePermissionGrantsByRoleIdReq,
                ) -> Result<ListRolePermissionGrantsByRoleIdRes, BoxedError>
                + Send
                + Sync,
        >,
    >,

    // Role Role Grants
    pub create_role_role_grant_fn: Option<
        Arc<
            dyn Fn(CreateRoleRoleGrantReq) -> Result<CreateRoleRoleGrantRes, BoxedError>
                + Send
                + Sync,
        >,
    >,
    pub delete_role_role_grant_fn: Option<
        Arc<
            dyn Fn(DeleteRoleRoleGrantReq) -> Result<DeleteRoleRoleGrantRes, BoxedError>
                + Send
                + Sync,
        >,
    >,
    pub list_role_role_grants_by_parent_id_fn: Option<
        Arc<
            dyn Fn(
                    ListRoleRoleGrantsByParentIdReq,
                ) -> Result<ListRoleRoleGrantsByParentIdRes, BoxedError>
                + Send
                + Sync,
        >,
    >,

    // Public Keys
    pub create_public_key_fn:
        Option<Arc<dyn Fn() -> Result<CreatePublicKeyRes, BoxedError> + Send + Sync>>,
    pub delete_public_key_fn:
        Option<Arc<dyn Fn(Uuid) -> Result<DeletePublicKeyRes, BoxedError> + Send + Sync>>,
    pub find_public_key_by_id_fn:
        Option<Arc<dyn Fn(Uuid) -> Result<FindPublicKeyByIdRes, BoxedError> + Send + Sync>>,
    pub list_all_public_keys_fn:
        Option<Arc<dyn Fn() -> Result<ListAllPublicKeysRes, BoxedError> + Send + Sync>>,

    // Refresh Tokens
    pub exchange_refresh_token_fn: Option<
        Arc<
            dyn Fn(ExchangeRefreshTokenReq) -> Result<ExchangeRefreshTokenRes, BoxedError>
                + Send
                + Sync,
        >,
    >,

    // Settings
    pub fetch_setting_fn:
        Option<Arc<dyn Fn(FetchSettingReq) -> Result<FetchSettingRes, BoxedError> + Send + Sync>>,
    pub save_setting_fn:
        Option<Arc<dyn Fn(SaveSettingReq) -> Result<SaveSettingRes, BoxedError> + Send + Sync>>,

    // Invitations
    pub accept_invitation_fn: Option<
        Arc<
            dyn Fn(AcceptInvitationParams) -> Result<AcceptInvitationRes, BoxedError> + Send + Sync,
        >,
    >,
    pub create_invitation_fn: Option<
        Arc<dyn Fn(CreateInvitationReq) -> Result<CreateInvitationRes, BoxedError> + Send + Sync>,
    >,
    pub find_invitation_fn: Option<
        Arc<dyn Fn(FindInvitationReq) -> Result<FindInvitationRes, BoxedError> + Send + Sync>,
    >,
}
