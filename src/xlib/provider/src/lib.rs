//! Vendored from freshbrewlabs project-template @ c38ec0a (2026-09); re-diff when templates change.
use core::fmt;
use std::{
    any::{Any, TypeId, type_name},
    collections::HashMap,
    error::Error,
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
        self.bindings
            .insert(value.type_id(), Arc::new(value));
    }

    pub fn fetch<T: Any>(&self) -> Result<&T, ProviderError> {
        self.bindings
            .get(&TypeId::of::<T>())
            .and_then(|ptr| ptr.downcast_ref())
            .ok_or_else(|| ProviderError::new(type_name::<T>()))
    }

    pub fn take<T: Any + Send + Sync>(&mut self) -> Result<T, ProviderError> {
        let t = self
            .bindings
            .remove(&TypeId::of::<T>())
            .and_then(|ptr| ptr.downcast::<T>().ok())
            .ok_or_else(|| ProviderError::new(type_name::<T>()))?;

        let t = Arc::into_inner(t).ok_or_else(|| ProviderError::new(type_name::<T>()))?;

        Ok(t)
    }

    pub fn fetch_unchecked<T: Any>(&self) -> &T {
        self.fetch::<T>()
            .unwrap_or_else(|err| panic!("provider error: {err:?}"))
    }
}

/// consumer projects re-add `impl FromRef<Provider> for oxidauth::OxidAuthClient`
/// in their own tree (see parkinglot) — the impl is 5 lines and belongs to
/// whoever owns both types (orphan-rule friendly since both are foreign here).

#[derive(Debug, Clone)]
pub struct ProviderError {
    type_name: &'static str,
}

impl ProviderError {
    pub fn new(type_name: &'static str) -> Self {
        Self { type_name }
    }
}

impl fmt::Display for ProviderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} not found", self.type_name,)
    }
}

impl Error for ProviderError {
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_return_fetch() {
        let mut provider = Provider::new();

        provider.store::<&str>("value");

        let result = provider.fetch::<&str>();

        match result {
            Ok(val) => assert_eq!(val, &"value"),
            Err(_) => unreachable!(),
        }
    }

    #[test]
    fn test_provider_take() {
        let mut provider = Provider::new();

        provider.store::<&str>("value");

        let result = provider.take::<&str>();

        match result {
            Ok(val) => assert_eq!(val, "value"),
            Err(_) => unreachable!(),
        }
    }

    #[test]
    fn it_should_return_an_error() {
        let mut provider = Provider::new();

        provider.store::<&str>("value");

        let result = provider.fetch::<String>();

        match result {
            Ok(_) => unreachable!(),
            Err(err) => {
                assert_eq!("alloc::string::String not found", err.to_string())
            },
        }
    }

    #[test]
    fn storing_the_same_type_twice_overwrites_the_binding() {
        let mut provider = Provider::new();

        provider.store::<i32>(1);
        provider.store::<i32>(2);

        assert_eq!(
            provider
                .fetch::<i32>()
                .unwrap(),
            &2
        );
    }

    #[test]
    fn fetch_miss_reports_the_exact_missing_type_name() {
        let provider = Provider::new();

        let err = provider
            .fetch::<bool>()
            .unwrap_err();

        assert_eq!(err.to_string(), "bool not found");
    }

    #[test]
    fn bindings_are_isolated_per_type() {
        let mut provider = Provider::new();

        provider.store::<i32>(7);
        provider.store::<String>("seven".into());
        provider.store::<u64>(7);

        assert_eq!(
            provider
                .fetch::<i32>()
                .unwrap(),
            &7
        );
        assert_eq!(
            provider
                .fetch::<String>()
                .unwrap(),
            &"seven".to_string()
        );
        assert_eq!(
            provider
                .fetch::<u64>()
                .unwrap(),
            &7
        );
        assert!(
            provider
                .fetch::<bool>()
                .is_err(),
            "stored types must not leak"
        );
    }

    #[test]
    fn take_removes_only_its_own_type() {
        let mut provider = Provider::new();

        provider.store::<i32>(7);
        provider.store::<u32>(9);

        assert_eq!(
            provider
                .take::<u32>()
                .unwrap(),
            9
        );
        assert!(
            provider
                .fetch::<u32>()
                .is_err(),
            "take must remove the binding"
        );
        assert_eq!(
            provider
                .fetch::<i32>()
                .unwrap(),
            &7
        );
    }

    #[test]
    fn take_fails_when_the_binding_is_shared_with_a_clone() {
        let mut provider = Provider::new();

        provider.store::<u16>(5);

        // Cloning shares the Arc<dyn Any>, so `Arc::into_inner` in `take` finds a
        // second strong ref held by `provider` and must yield ProviderError.
        let mut cloned = provider.clone();

        assert!(cloned.take::<u16>().is_err());
        assert_eq!(
            provider
                .fetch::<u16>()
                .unwrap(),
            &5,
            "original must be untouched"
        );
    }

    #[test]
    fn fetch_unchecked_returns_the_binding() {
        let mut provider = Provider::new();

        provider.store::<i64>(42);

        assert_eq!(*provider.fetch_unchecked::<i64>(), 42);
    }

    #[test]
    #[should_panic(expected = "provider error")]
    fn fetch_unchecked_panics_on_miss() {
        Provider::new().fetch_unchecked::<bool>();
    }
}
