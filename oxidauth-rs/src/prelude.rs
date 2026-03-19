pub use oxidauth_http::prelude::parse_and_validate;
pub use oxidauth_kernel::{
    error::BoxedError,
    jwt::{
        EntitlementsEncoding,
        Jwt,
    },
};

pub use crate::client::{
    Client as OxidAuthClient,
    ClientError as OxidAuthClientError,
    ClientTrait as OxidAuthClientTrait,
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
        roles::{
            CreateUserRoleTrait,
            DeleteUserRoleTrait,
            ListUserRolesByUserIdTrait,
        },
    },
};
