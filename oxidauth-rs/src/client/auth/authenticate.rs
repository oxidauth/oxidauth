use oxidauth_kernel::error::BoxedError;

use super::*;

impl Client {
    pub async fn authenticate(&self) -> Result<bool, BoxedError> {
        self.auth()
            .await
            .map_err(|err| err.into())
    }
}
