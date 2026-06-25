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
    can::CanTrait as _,
    invitations::*,
    permissions::*,
    public_keys::*,
     refresh_tokens::*,
     roles::{
         CreateRoleTrait as _,
         DeleteRoleTrait as _,
         FindRoleByIdTrait as _,
         FindRoleByNameTrait as _,
         ListAllRolesTrait as _,
         RolePermissionsTrait as _,
         RoleRoleGrantsTrait as _,
         RolesTrait as _,
         UpdateRoleTrait as _,
         permissions::{
             CreateRolePermissionGrantTrait as _,
             DeleteRolePermissionGrantTrait as _,
             ListRolePermissionGrantsByRoleIdTrait as _,
         },
         roles::{
             CreateRoleRoleGrantTrait as _,
             DeleteRoleRoleGrantTrait as _,
             ListRoleRoleGrantsByParentIdTrait as _,
         },
     },
     settings::*,
     users::{
         CreateUserTrait as _,
         DeleteUserTrait as _,
         FindUserByIdTrait as _,
         FindUsersByIdsTrait as _,
         FindUserByUsernameTrait as _,
         ListAllUsersTrait as _,
         UpdateUserTrait as _,
         UserAuthoritiesTrait as _,
         UserPermissionsTrait as _,
         UserRolesTrait as _,
         UsersTrait as _,
         authorities::{
             CreateUserAuthorityTrait as _,
             DeleteUserAuthorityTrait as _,
             FindUserAuthorityByUserIdAndAuthorityIdTrait as _,
             ListUserAuthoritiesByUserIdTrait as _,
             UpdateUserAuthorityTrait as _,
         },
         permissions::{
             CreateUserPermissionGrantTrait as _,
             DeleteUserPermissionGrantTrait as _,
             ListUserPermissionGrantsByUserIdTrait as _,
         },
         roles::{
             CreateUserRoleTrait as _,
             DeleteUserRoleTrait as _,
             ListUserRolesByUserIdTrait as _,
         },
     },
};
