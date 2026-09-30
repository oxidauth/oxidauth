use oxidauth_kernel::permissions::{
    RawPermission,
    find_permission_by_parts::FindPermissionByParts,
};
use oxidauth_repository::permissions::select_permission_by_parts::*;

use super::*;

#[async_trait]
impl SelectPermissionByPartsQuery for PgPermissionRepository {
    #[tracing::instrument(name = "select_permission_by_parts_query", skip(self))]
    async fn select_permission_by_parts(
        &self,
        params: &FindPermissionByParts,
    ) -> Result<Option<Permission>, BoxedError> {
        let perm_string = &params.permission;
        let permission: RawPermission = perm_string.try_into()?;

        let result =
            sqlx::query_as::<_, PgPermission>(include_str!("./query_permission_by_parts.sql"))
                .bind(&permission.realm)
                .bind(&permission.resource)
                .bind(&permission.action)
                .fetch_optional(&self.db.read_pool())
                .await?;

        let permission = result.map(Into::into);

        Ok(permission)
    }
}

#[cfg(test)]
mod tests {

    use oxidauth_kernel::permissions::find_permission_by_parts::FindPermissionByParts;
    use sqlx::PgPool;

    use super::*;
    use crate::{Database, test_fixtures::seed_permission};

    async fn find_by_parts(pool: PgPool, permission: &str) -> Option<Permission> {
        let repo = PgPermissionRepository::new(Database::from_pool(pool));
        repo.select_permission_by_parts(&FindPermissionByParts {
            permission: permission.to_owned(),
        })
        .await
        .unwrap_or_else(|err| panic!("query for '{permission}' should succeed, got: {err:?}"))
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_find_the_permission_matching_all_three_parts(pool: PgPool) {
        let id = seed_permission(&pool, "realmA", "resourceA", "actionA").await;

        let found = find_by_parts(pool, "realmA:resourceA:actionA").await;

        let found = found.expect("the seeded permission should be found");
        assert_eq!(found.id, id);
        assert_eq!(
            (
                found.realm.as_str(),
                found.resource.as_str(),
                found.action.as_str()
            ),
            ("realmA", "resourceA", "actionA")
        );
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_return_none_for_a_missing_row(pool: PgPool) {
        seed_permission(&pool, "realmA", "resourceA", "actionA").await;

        // pinned: this query returns Option, not a RowNotFound error
        let found = find_by_parts(pool, "realmA:resourceA:no-such-action").await;

        assert!(found.is_none(), "a missing row must come back as Ok(None)");
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_match_wildcards_literally_not_globally(pool: PgPool) {
        // kernel RawPermission accepts any non-empty parts, so '*' rows are
        // storable; the SQL compares each part with =, so lookups are literal
        let wildcard_id = seed_permission(&pool, "oxidauth", "*", "read").await;
        seed_permission(&pool, "oxidauth", "users", "read").await;

        let exact = find_by_parts(pool.clone(), "oxidauth:*:read").await;
        assert_eq!(
            exact
                .expect("the literal '*' row should be findable")
                .id,
            wildcard_id
        );

        assert!(
            find_by_parts(pool.clone(), "oxidauth:anything:read")
                .await
                .is_none(),
            "the stored '*' must not glob-match a different resource"
        );
        assert!(
            find_by_parts(pool.clone(), "oxidauth:users*:read")
                .await
                .is_none(),
            "a queried partial wildcard must not match (exact equality only)"
        );
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_error_on_a_partial_permission_string(pool: PgPool) {
        let repo = PgPermissionRepository::new(Database::from_pool(pool));

        let err = repo
            .select_permission_by_parts(&FindPermissionByParts {
                permission: "realmA:resourceA".to_owned(),
            })
            .await
            .expect_err("two parts are not a permission");
        assert!(
            err.to_string()
                .contains("all three parts"),
            "expected the RawPermission parse error, got: {err:?}"
        );
    }
}
