use std::{
    any::{type_name, Any, TypeId},
    collections::HashMap,
    sync::Arc,
};

#[derive(Default, Clone)]
pub struct Provider {
    bindings: HashMap<TypeId, Arc<dyn Any + Send + Sync + 'static>>,
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
