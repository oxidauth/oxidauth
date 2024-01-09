use std::{
    any::{type_name, Any, TypeId},
    collections::HashMap,
    sync::Arc,
};

use oxidauth_kernel::users::user_create::CreateUserService;
use oxidauth_services::users::create::CreateUserUseCase;

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

    let user_create_service: CreateUserService = Arc::new(
        CreateUserUseCase::new(db.clone()),
    );
    provider.store(user_create_service);

    // let register_service = RegisterUseCase::new();

    provider
}
