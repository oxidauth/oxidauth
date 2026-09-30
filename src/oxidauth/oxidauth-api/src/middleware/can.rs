// API-local `Can` permission check (migration plan 09; OXA-000069 rebuild).
//
// The kernel originals — `oxidauth_kernel::service::{CanLayer, CanService,
// CanError}` — were removed in OXA-000069. This file no longer composes over
// them: the check is now a plain stateless validation against the
// `oxidauth_permission` crate's `parse`/`validate`, which is what the old
// `CanService::call` did around its inner-service dispatch.

// Permission checks here are stateless by design: the decision is recomputed per
// request from the JWT `entitlements` claim
// (`middleware::permission_extractor::ExtractEntitlements`). There is no decision
// cache; the staleness bound on any revocation is the authority's
// `settings.jwt_ttl`, and status enforcement (absent — tracked) belongs upstream
// in authenticate/refresh, not here.
//
// This invariant survived the OXA-000069 rebuild (Service/CanLayer removal):
// the stateless permission check it describes is unchanged in behavior — same
// parse, same `validate`, same Ok/`CanError::Unauthorized` decision, same
// `BadPermissions` mapping for malformed entitlement input.

use std::convert::Infallible;

use oxidauth_permission::{PermissionParseErr, Token, parse::parse, validate};

pub trait ExtractPermissions {
    fn permissions(&self) -> &str;
}

/// Permission-check failure, previously `oxidauth_kernel::service::CanError`
/// (removed in OXA-000069); same variants, same mapping.
#[derive(Debug)]
pub enum CanError<E> {
    Unauthorized,
    BadPermissions(PermissionParseErr),
    Other(E),
}

/// Stateless permission gate: parse the required-permission string once per
/// check, split the request's entitlements on spaces, and let
/// `oxidauth_permission::validate` decide. `Ok(())` grants,
/// `Err(CanError::Unauthorized)` denies, `Err(CanError::BadPermissions)`
/// reports unparseable input — exactly the old middleware's decision table.
pub fn can<'a, R: ExtractPermissions + ?Sized>(
    raw_required: &'a str,
    req: &R,
) -> Result<(), CanError<Infallible>> {
    let required: Vec<Token<'a>> = match parse(raw_required) {
        Ok(required) => required,
        Err(err) => return Err(CanError::BadPermissions(err)),
    };

    let entitlements: Vec<String> = req
        .permissions()
        .split(' ')
        .map(|s| s.to_string())
        .collect();

    match validate(&required, &entitlements) {
        Ok(true) => Ok(()),
        Ok(false) => Err(CanError::Unauthorized),
        Err(err) => Err(CanError::BadPermissions(err)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub struct MockPermissions(&'static str);

    impl ExtractPermissions for MockPermissions {
        fn permissions(&self) -> &str {
            self.0
        }
    }

    #[test]
    fn it_should_allow_call_with_good_permissions() {
        let req = MockPermissions("**:**:**");

        let result = can("oxidauth:users:read", &req);

        match result {
            Ok(_) => {},
            Err(_) => unreachable!(),
        }
    }

    #[test]
    fn it_should_return_unauthorized_with_bad_permissions() {
        let req = MockPermissions("oxidauth:**:write");

        let result = can("oxidauth:users:read", &req);

        match result {
            Err(CanError::Unauthorized) => {},
            Ok(_) => unreachable!(),
            Err(_) => unreachable!(),
        }
    }
}
