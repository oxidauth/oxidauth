use oxidauth_kernel::user_permission_grants::{
    list_user_permission_grants_by_user_id::ListUserPermissionGrantsByUserId,
    UserPermission,
};

use crate::prelude::*;

use super::PgUserPermission;

#[async_trait]
impl<'a> Service<&'a ListUserPermissionGrantsByUserId> for Database {
    type Response = Vec<UserPermission>;
    type Error = BoxedError;

    #[tracing::instrument(
        name = "select_user_permission_grants_by_user_id_query",
        skip(self)
    )]
    async fn call(
        &self,
        params: &'a ListUserPermissionGrantsByUserId,
    ) -> Result<Self::Response, Self::Error> {
        let user_permission_grants =
            sqlx::query_as::<_, PgUserPermission>(include_str!(
                "./select_user_permission_grants_by_user_id_query.sql"
            ))
            .bind(params.user_id)
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(Into::into)
            .collect();

        Ok(user_permission_grants)
    }
}

#[cfg(test)]
mod tests {
    use oxidauth_repository::users::insert_user::*;
    use sqlx::PgPool;

    use super::*;

    #[ignore]
    #[sqlx::test]
    async fn it_should_query_user_permission_grants_successfully(pool: PgPool) {
    }
}
