pub use oxidauth_kernel::{
    error::BoxedError,
    jwt::{EntitlementsEncoding, Jwt},
};
pub use oxidauth_permission::parse_and_validate;

#[cfg(feature = "server")]
pub use crate::client::{
    Client as OxidAuthClient,
    ClientError as OxidAuthClientError,
    ClientTrait as OxidAuthClientTrait,
};
pub use crate::client::{
    auth::*,
    authorities::*,
    can::CanTrait,
    invitations::*,
    permissions::*,
    public_keys::*,
    refresh_tokens::*,
    roles::{
        CreateRoleTrait,
        DeleteRoleTrait,
        FindRoleByIdTrait,
        FindRoleByNameTrait,
        ListAllRolesTrait,
        RolesTrait,
        UpdateRoleTrait,
        permissions::{
            CreateRolePermissionGrantTrait,
            DeleteRolePermissionGrantTrait,
            ListRolePermissionGrantsByRoleIdTrait,
        },
        roles::{
            CreateRoleRoleGrantTrait,
            DeleteRoleRoleGrantTrait,
            ListRoleRoleGrantsByParentIdTrait,
        },
    },
    settings::*,
    users::{
        CreateUserTrait,
        DeleteUserTrait,
        FindUserByIdTrait,
        FindUserByUsernameTrait,
        ListAllUsersTrait,
        UsersTrait,
        authorities::{
            CreateUserAuthorityTrait,
            DeleteUserAuthorityTrait,
            FindUserAuthorityByUserIdAndAuthorityIdTrait,
            ListUserAuthoritiesByUserIdTrait,
            UpdateUserAuthorityTrait,
        },
        permissions::{
            CreateUserPermissionGrantTrait,
            DeleteUserPermissionGrantTrait,
            ListUserPermissionGrantsByUserIdTrait,
        },
        roles::{CreateUserRoleTrait, DeleteUserRoleTrait, ListUserRolesByUserIdTrait},
    },
};
