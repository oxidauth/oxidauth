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
        use oxidauth_kernel::users::find_user_by_id::FindUserByIdService;
        use oxidauth_usecases::users::find_user_by_id::FindUserByIdUseCase;

        let find_user_by_id_service = Arc::new(FindUserByIdUseCase::new(
            db.clone(),
        ));
        provider.store::<FindUserByIdService>(find_user_by_id_service);
    }

    {
        use oxidauth_usecases::permissions::create_permission::CreatePermissionUseCase;
        use oxidauth_kernel::permissions::create_permission::CreatePermissionService;

        let create_permission_service = Arc::new(CreatePermissionUseCase::new(db.clone()));
        provider.store::<CreatePermissionService>(create_permission_service);
    }

    {
        use oxidauth_usecases::permissions::find_permission_by_parts::FindPermissionByPartsUseCase;
        use oxidauth_kernel::permissions::find_permission_by_parts::FindPermissionByPartsService;

        let find_permission_by_parts_service = Arc::new(FindPermissionByPartsUseCase::new(db.clone()));
        provider.store::<FindPermissionByPartsService>(find_permission_by_parts_service);
    }

    provider
}
