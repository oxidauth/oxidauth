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
        use oxidauth_usecases::permissions::delete_permission::DeletePermissionUseCase;
        use oxidauth_kernel::permissions::delete_permission::DeletePermissionService;

        let delete_permission_service = Arc::new(DeletePermissionUseCase::new(db.clone()));
        provider.store::<DeletePermissionService>(delete_permission_service);
    }

    {
        use oxidauth_kernel::roles::create_role::CreateRoleService;
        use oxidauth_usecases::roles::create_role::CreateRoleUseCase;

        let create_role_service = Arc::new(CreateRoleUseCase::new(
            db.clone(),
        ));
        provider.store::<CreateRoleService>(create_role_service);
    }


    provider
}
