use std::future::Future;

use async_trait::async_trait;
use oxidauth_permission::{parse::parse, validate, PermissionParseErr, Token};

#[async_trait]
pub trait Service<Request>: Send + Sync + 'static {
    type Response;
    type Error;

    async fn call(&self, req: Request) -> Result<Self::Response, Self::Error>
    where
        Request: 'async_trait;
}

pub trait Layer<S> {
    type Service;

    fn layer(&self, inner: S) -> Self::Service;
}

pub struct CanLayer<'a> {
    permissions: Vec<Token<'a>>,
}

impl<'a> CanLayer<'a> {
    pub fn new(raw: &'a str) -> Result<Self, PermissionParseErr> {
        let permissions = parse(raw)?;

        Ok(Self { permissions })
    }
}

impl<'a, S> Layer<S> for CanLayer<'a> {
    type Service = CanService<'a, S>;

    fn layer(&self, service: S) -> Self::Service {
        CanService {
            permissions: self.permissions.clone(),
            service,
        }
    }
}

pub struct CanService<'a, S> {
    permissions: Vec<Token<'a>>,
    service: S,
}

#[async_trait]
impl<S, Request> Service<Request> for CanService<'static, S>
where
    S: Service<Request>,
    Request: Send + 'static,
    Request: ExtractPermissions,
{
    type Response = S::Response;
    type Error = CanError<S::Error>;

    async fn call(&self, req: Request) -> Result<Self::Response, Self::Error> {
        let permissions = req.permissions();

        match validate(&self.permissions, permissions) {
            Ok(true) => self
                .service
                .call(req)
                .await
                .map_err(CanError::Other),
            Ok(false) => Err(CanError::Unauthorized),
            Err(err) => Err(CanError::BadPermissions(err)),
        }
    }
}

pub enum CanError<E> {
    Unauthorized,
    BadPermissions(PermissionParseErr),
    Other(E),
}

pub trait ExtractPermissions {
    fn permissions(&self) -> &str;
}

#[cfg(test)]
mod tests {
    use super::*;

    pub struct MockService;
    pub struct MockServiceParams(&'static str);

    #[async_trait]
    impl Service<MockServiceParams> for MockService {
        type Response = ();
        type Error = ();

        async fn call(
            &self,
            _req: MockServiceParams,
        ) -> Result<Self::Response, Self::Error> {
            Ok(())
        }
    }

    impl ExtractPermissions for MockServiceParams {
        fn permissions(&self) -> &str {
            self.0
        }
    }

    #[tokio::test]
    async fn it_should_allow_call_with_good_permissions() {
        let inner = MockService;

        let can_middleware = CanLayer::new("oxidauth:users:read").expect(
            "should be able to parse a valid static set of permissions",
        );

        let service = can_middleware.layer(inner);

        let req = MockServiceParams("**:**:**");

        let result = service.call(req).await;

        match result {
            Ok(_) => {},
            Err(_) => unreachable!(),
        }
    }

    #[tokio::test]
    async fn it_should_return_unauthorized_with_bad_permissions() {
        let inner = MockService;

        let can_middleware = CanLayer::new("oxidauth:users:read").expect(
            "should be able to parse a valid static set of permissions",
        );

        let service = can_middleware.layer(inner);

        let req = MockServiceParams("oxidauth:**:write");

        let result = service.call(req).await;

        match result {
            Err(CanError::Unauthorized) => {},
            Ok(_) => unreachable!(),
            Err(_) => unreachable!(),
        }
    }
}
