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

impl Error for ProviderError {}

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
            }
        }
    }
}