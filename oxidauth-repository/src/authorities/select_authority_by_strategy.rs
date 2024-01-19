pub use oxidauth_kernel::authorities::find_authority_by_strategy::FindAuthorityByStrategy;
pub use oxidauth_kernel::{service::Service, authorities::Authority};

pub use crate::prelude::*;

pub trait SelectAuthorityByStrategyQuery:
    for<'a> Service<&'a FindAuthorityByStrategy, Response = Authority, Error = BoxedError>
{
}

impl<T> SelectAuthorityByStrategyQuery for T where
    T: for<'a> Service<&'a FindAuthorityByStrategy, Response = Authority, Error = BoxedError>
{
}
