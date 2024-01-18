use std::{
    any::{type_name, Any, TypeId},
    collections::HashMap,
    sync::Arc,
};

#[derive(Default, Clone)]
pub struct Provider {
    pub bindings: HashMap<TypeId, Arc<dyn Any + Send + Sync + 'static>>,
}

impl Provider {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn store<T: Any + Send + Sync + 'static>(&mut self, value: T) {
        self.bindings.insert(
            value.type_id(),
            Arc::new(value),
        );
    }

    pub fn fetch<T: Any>(&self) -> &T {
        self.bindings
            .get(&TypeId::of::<T>())
            .and_then(|ptr| ptr.downcast_ref())
            .unwrap_or_else(|| {
                panic!(
                    "error getting {} from provider",
                    type_name::<T>()
                )
            })
    }
}

pub async fn setup() -> Provider {
    let mut provider = Provider::new();

    let db = oxidauth_postgres::Database::from_env()
        .await
        .unwrap();

    {
        use oxidauth_kernel::users::create_user::CreateUserService;
        use oxidauth_usecases::users::create_user::CreateUserUseCase;

        let create_user_service = Arc::new(CreateUserUseCase::new(
            db.clone(),
        ));
        provider.store::<CreateUserService>(create_user_service);
    }

    {
        use oxidauth_kernel::users::update_user::UpdateUserService;
        use oxidauth_usecases::users::update_user::UpdateUserUseCase;

        let update_user_service = Arc::new(UpdateUserUseCase::new(
            db.clone(),
        ));
        provider.store::<UpdateUserService>(update_user_service);
    }

    {
        use oxidauth_kernel::users::find_user_by_id::FindUserByIdService;
        use oxidauth_usecases::users::find_user_by_id::FindUserByIdUseCase;

        let find_user_by_id_service = Arc::new(FindUserByIdUseCase::new(
            db.clone(),
        ));
        provider.store::<FindUserByIdService>(find_user_by_id_service);
    }

    {
        use oxidauth_kernel::users::delete_user_by_id::DeleteUserByIdService;
        use oxidauth_usecases::users::delete_user_by_id::DeleteUserByIdUseCase;

        let delete_user_by_id_service = Arc::new(DeleteUserByIdUseCase::new(
            db.clone(),
        ));
        provider.store::<DeleteUserByIdService>(delete_user_by_id_service);
    }

    {
        use oxidauth_kernel::users::find_user_by_username::FindUserByUsernameService;
        use oxidauth_usecases::users::find_user_by_username::FindUserByUsernameUseCase;

        let find_user_by_username_service =
            Arc::new(FindUserByUsernameUseCase::new(db.clone()));
        provider
            .store::<FindUserByUsernameService>(find_user_by_username_service);
    }

    {
        use oxidauth_kernel::users::list_all_users::ListAllUsersService;
        use oxidauth_usecases::users::list_all_users::ListAllUsersUseCase;

        let list_all_users_service = Arc::new(ListAllUsersUseCase::new(
            db.clone(),
        ));
        provider.store::<ListAllUsersService>(list_all_users_service);
    }

    {
        use oxidauth_kernel::permissions::create_permission::CreatePermissionService;
        use oxidauth_usecases::permissions::create_permission::CreatePermissionUseCase;

        let create_permission_service = Arc::new(CreatePermissionUseCase::new(
            db.clone(),
        ));
        provider.store::<CreatePermissionService>(create_permission_service);
    }

    {
        use oxidauth_kernel::permissions::find_permission_by_parts::FindPermissionByPartsService;
        use oxidauth_usecases::permissions::find_permission_by_parts::FindPermissionByPartsUseCase;

        let find_permission_by_parts_service =
            Arc::new(FindPermissionByPartsUseCase::new(db.clone()));
        provider.store::<FindPermissionByPartsService>(
            find_permission_by_parts_service,
        );
    }

    {
        use oxidauth_kernel::permissions::list_all_permissions::ListAllPermissionsService;
        use oxidauth_usecases::permissions::list_all_permissions::ListAllPermissionsUseCase;

        let list_all_permissions_service =
            Arc::new(ListAllPermissionsUseCase::new(db.clone()));
        provider
            .store::<ListAllPermissionsService>(list_all_permissions_service);
    }

    {
        use oxidauth_kernel::permissions::delete_permission::DeletePermissionService;
        use oxidauth_usecases::permissions::delete_permission::DeletePermissionUseCase;

        let delete_permission_service = Arc::new(DeletePermissionUseCase::new(
            db.clone(),
        ));
        provider.store::<DeletePermissionService>(delete_permission_service);
    }

    {
        use oxidauth_kernel::user_permission_grants::create_user_permission_grant::CreateUserPermissionGrantService;
        use oxidauth_usecases::user_permission_grants::create_user_permission_grant::CreateUserPermissionGrantUseCase;

        let create_user_permission_grant_service = Arc::new(
            CreateUserPermissionGrantUseCase::new(
                db.clone(),
                db.clone(),
                db.clone(),
            ),
        );
        provider.store::<CreateUserPermissionGrantService>(
            create_user_permission_grant_service,
        );
    }

    {
        use oxidauth_kernel::user_permission_grants::list_user_permission_grants_by_user_id::ListUserPermissionGrantsByUserIdService;
        use oxidauth_usecases::user_permission_grants::list_user_permission_grants_by_user_id::ListUserPermissionGrantsByUserIdUseCase;

        let list_user_permission_grants_by_user_id_service =
            Arc::new(ListUserPermissionGrantsByUserIdUseCase::new(db.clone()));
        provider.store::<ListUserPermissionGrantsByUserIdService>(
            list_user_permission_grants_by_user_id_service,
        );
    }

    {
        use oxidauth_kernel::user_permission_grants::delete_user_permission_grant::DeleteUserPermissionGrantService;
        use oxidauth_usecases::user_permission_grants::delete_user_permission_grant::DeleteUserPermissionGrantUseCase;

        let delete_user_permission_grant_service = Arc::new(
            DeleteUserPermissionGrantUseCase::new(
                db.clone(),
                db.clone(),
                db.clone(),
            ),
        );
        provider.store::<DeleteUserPermissionGrantService>(
            delete_user_permission_grant_service,
        );
    }

    {
        use oxidauth_kernel::user_authorities::update_user_authority::UpdateUserAuthorityService;
        use oxidauth_usecases::user_authorities::update_user_authority::UpdateUserAuthorityUseCase;

        let update_user_authority_service =
            Arc::new(UpdateUserAuthorityUseCase::new(db.clone()));
        provider
            .store::<UpdateUserAuthorityService>(update_user_authority_service);
    }

    {
        use oxidauth_kernel::user_authorities::delete_user_authority::DeleteUserAuthorityService;
        use oxidauth_usecases::user_authorities::delete_user_authority::DeleteUserAuthorityUseCase;

        let delete_user_authority_service =
            Arc::new(DeleteUserAuthorityUseCase::new(db.clone()));
        provider
            .store::<DeleteUserAuthorityService>(delete_user_authority_service);
    }

    {
        use oxidauth_kernel::user_authorities::find_user_authority_by_user_id_and_authority_id::FindUserAuthorityByUserIdAndAuthorityIdService;
        use oxidauth_usecases::user_authorities::find_user_authority_by_user_id_and_authority_id::FindUserAuthorityByUserIdAndAuthorityIdUseCase;

        let find_user_authority_by_user_id_and_authority_id_service = Arc::new(
            FindUserAuthorityByUserIdAndAuthorityIdUseCase::new(db.clone()),
        );
        provider.store::<FindUserAuthorityByUserIdAndAuthorityIdService>(
            find_user_authority_by_user_id_and_authority_id_service,
        );
    }

    {
        use oxidauth_kernel::roles::create_role::CreateRoleService;
        use oxidauth_usecases::roles::create_role::CreateRoleUseCase;

        let create_role_service = Arc::new(CreateRoleUseCase::new(
            db.clone(),
        ));
        provider.store::<CreateRoleService>(create_role_service);
    }

    {
        use oxidauth_kernel::roles::find_role_by_id::FindRoleByIdService;
        use oxidauth_usecases::roles::find_role_by_id::FindRoleByIdUseCase;

        let find_role_by_id_service = Arc::new(FindRoleByIdUseCase::new(
            db.clone(),
        ));
        provider.store::<FindRoleByIdService>(find_role_by_id_service);
    }

    {
        use oxidauth_kernel::roles::list_all_roles::ListAllRolesService;
        use oxidauth_usecases::roles::list_all_roles::ListAllRolesUseCase;

        let list_all_roles_service = Arc::new(ListAllRolesUseCase::new(
            db.clone(),
        ));
        provider.store::<ListAllRolesService>(list_all_roles_service);
    }

    {
        use oxidauth_kernel::roles::update_role::UpdateRoleService;
        use oxidauth_usecases::roles::update_role::UpdateRoleUseCase;

        let update_role_service = Arc::new(UpdateRoleUseCase::new(
            db.clone(),
        ));
        provider.store::<UpdateRoleService>(update_role_service);
    }

    {
        use oxidauth_kernel::roles::delete_role::DeleteRoleService;
        use oxidauth_usecases::roles::delete_role::DeleteRoleUseCase;

        let delete_role_service = Arc::new(DeleteRoleUseCase::new(
            db.clone(),
        ));
        provider.store::<DeleteRoleService>(delete_role_service);
    }

    {
        use oxidauth_kernel::user_role_grants::create_user_role_grant::CreateUserRoleGrantService;
        use oxidauth_usecases::user_role_grants::create_user_role_grant::CreateUserRoleGrantUseCase;

        let create_user_role_service = Arc::new(
            CreateUserRoleGrantUseCase::new(
                db.clone(),
                db.clone(),
                db.clone(),
            ),
        );
        provider.store::<CreateUserRoleGrantService>(create_user_role_service);
    }

    {
        use oxidauth_kernel::user_role_grants::delete_user_role_grant::DeleteUserRoleGrantService;
        use oxidauth_usecases::user_role_grants::delete_user_role_grant::DeleteUserRoleGrantUseCase;

        let delete_user_role_service = Arc::new(
            DeleteUserRoleGrantUseCase::new(
                db.clone(),
                db.clone(),
                db.clone(),
            ),
        );
        provider.store::<DeleteUserRoleGrantService>(delete_user_role_service);
    }

    {
        use oxidauth_kernel::user_role_grants::list_user_role_grants_by_user_id::ListUserRoleGrantsByUserIdService;
        use oxidauth_usecases::user_role_grants::list_user_role_grants_by_user_id::ListUserRoleGrantsByUserIdUseCase;

        let list_user_role_grants_by_user_id_service =
            Arc::new(ListUserRoleGrantsByUserIdUseCase::new(db.clone()));
        provider.store::<ListUserRoleGrantsByUserIdService>(
            list_user_role_grants_by_user_id_service,
        );
    }

    {
        use oxidauth_kernel::role_role_grants::create_role_role_grant::CreateRoleRoleGrantService;
        use oxidauth_usecases::role_role_grants::create_role_role_grant::CreateRoleRoleGrantUseCase;

        let create_role_role_grant_service =
            Arc::new(CreateRoleRoleGrantUseCase::new(db.clone(), db.clone()));
        provider.store::<CreateRoleRoleGrantService>(
            create_role_role_grant_service,
        );
    }

    {
        use oxidauth_kernel::role_role_grants::delete_role_role_grant::DeleteRoleRoleGrantService;
        use oxidauth_usecases::role_role_grants::delete_role_role_grant::DeleteRoleRoleGrantUseCase;

        let delete_role_role_grant_service =
            Arc::new(DeleteRoleRoleGrantUseCase::new(db.clone()));
        provider.store::<DeleteRoleRoleGrantService>(
            delete_role_role_grant_service,
        );
    }
    {
        use oxidauth_kernel::role_role_grants::list_role_role_grants_by_parent_id::ListRoleRoleGrantsByParentIdService;
        use oxidauth_usecases::role_role_grants::list_role_role_grants_by_parent_id::ListRoleRoleGrantsByParentIdUseCase;

        let list_role_role_grants_by_parent_id_service =
            Arc::new(ListRoleRoleGrantsByParentIdUseCase::new(db.clone()));
        provider.store::<ListRoleRoleGrantsByParentIdService>(
            list_role_role_grants_by_parent_id_service,
        );
    }

    {
        use oxidauth_kernel::role_permission_grants::create_role_permission_grant::CreateRolePermissionGrantService;
        use oxidauth_usecases::role_permission_grants::create_role_permission_grant::CreateRolePermissionGrantUseCase;

        let create_role_permission_grant_service = Arc::new(
            CreateRolePermissionGrantUseCase::new(
                db.clone(),
                db.clone(),
                db.clone(),
            ),
        );
        provider.store::<CreateRolePermissionGrantService>(
            create_role_permission_grant_service,
        );
    }

    {
        use oxidauth_kernel::role_permission_grants::create_role_permission_grant::CreateRolePermissionGrantService;
        use oxidauth_usecases::role_permission_grants::create_role_permission_grant::CreateRolePermissionGrantUseCase;

        let create_role_permission_grant_service = Arc::new(
            CreateRolePermissionGrantUseCase::new(
                db.clone(),
                db.clone(),
                db.clone(),
            ),
        );
        provider.store::<CreateRolePermissionGrantService>(
            create_role_permission_grant_service,
        );
    }

    {
        use oxidauth_kernel::role_permission_grants::list_role_permission_grants_by_role_id::ListRolePermissionGrantsByRoleIdService;
        use oxidauth_usecases::role_permission_grants::list_role_permission_grants_by_role_id::ListRolePermissionGrantsByRoleIdUseCase;

        let list_role_permission_grants_by_role_id_service =
            Arc::new(ListRolePermissionGrantsByRoleIdUseCase::new(db.clone()));
        provider.store::<ListRolePermissionGrantsByRoleIdService>(
            list_role_permission_grants_by_role_id_service,
        );
    }

    {
        use oxidauth_kernel::role_permission_grants::delete_role_permission_grant::DeleteRolePermissionGrantService;
        use oxidauth_usecases::role_permission_grants::delete_role_permission_grant::DeleteRolePermissionGrantUseCase;

        let delete_role_permission_grant_service = Arc::new(
            DeleteRolePermissionGrantUseCase::new(
                db.clone(),
                db.clone(),
                db.clone(),
            ),
        );
        provider.store::<DeleteRolePermissionGrantService>(
            delete_role_permission_grant_service,
        );
    }

    provider
}
