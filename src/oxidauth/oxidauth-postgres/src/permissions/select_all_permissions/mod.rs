use oxidauth_kernel::permissions::list_all_permissions::ListAllPermissions;
use oxidauth_repository::permissions::select_all_permissions::*;

use super::*;

#[async_trait]
impl SelectAllPermissionsQuery for PgPermissionRepository {
    #[tracing::instrument(name = "select_all_permissions_query", skip(self))]
    async fn select_all_permissions(
        &self,
        params: &ListAllPermissions,
    ) -> Result<Vec<Permission>, BoxedError> {
        let result =
            sqlx::query_as::<_, PgPermission>(include_str!("./select_all_permissions.sql"))
                .fetch_all(&self.db.read_pool())
                .await?
                .into_iter()
                .map(Into::into)
                .collect();

        Ok(result)
    }
}

#[cfg(test)]
mod tests {

    use oxidauth_kernel::permissions::list_all_permissions::ListAllPermissions;
    use sqlx::PgPool;

    use super::*;
    use crate::{Database, test_fixtures::seed_permission};

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_return_an_empty_list_for_an_empty_table(pool: PgPool) {
        let repo = PgPermissionRepository::new(Database::from_pool(pool));

        let permissions = repo
            .select_all_permissions(&ListAllPermissions)
            .await
            .expect("select on an empty table should succeed");

        assert!(
            permissions.is_empty(),
            "fresh test db should have no permissions"
        );
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_return_every_stored_permission(pool: PgPool) {
        let mut seeded = Vec::new();
        seeded.push(seed_permission(&pool, "realmA", "resourceA", "actionA").await);
        seeded.push(seed_permission(&pool, "realmB", "resourceB", "actionB").await);
        // Guard (OXA-000019 precedent): force the tie — seed rows tie on created_at
        // only if NOW() never ticks between INSERTs, and while updated_at equals
        // created_at an updated_at-first orderer is indistinguishable from the
        // correct one. Pin every row's created_at to the newest one, then drop the
        // max-id row's updated_at below that tie: `ORDER BY updated_at ASC, id ASC`
        // must then hoist the max-id row to the front — observably wrong — while
        // the correct orderer never reads updated_at and keeps it last.
        sqlx::query(
            "UPDATE permissions SET created_at = (SELECT max(created_at) FROM permissions), \
             updated_at = CASE WHEN id = (SELECT id FROM permissions ORDER BY id DESC LIMIT 1) \
             THEN (SELECT max(created_at) FROM permissions) - INTERVAL '1 hour' \
             ELSE (SELECT max(created_at) FROM permissions) END",
        )
        .execute(&pool)
        .await
        .expect("diverge one row's updated_at");

        let repo = PgPermissionRepository::new(Database::from_pool(pool));
        let permissions = repo
            .select_all_permissions(&ListAllPermissions)
            .await
            .expect("select_all_permissions should succeed");

        assert_eq!(permissions.len(), 2, "both seeded rows should come back");

        assert!(
            permissions
                .windows(2)
                .all(|w| (w[0].created_at, w[0].id) <= (w[1].created_at, w[1].id)),
            "rows must be returned in (created_at, id) order"
        );

        let again = repo
            .select_all_permissions(&ListAllPermissions)
            .await
            .expect("repeated query should succeed");
        assert_eq!(
            permissions
                .iter()
                .map(|p| p.id)
                .collect::<Vec<_>>(),
            again
                .iter()
                .map(|p| p.id)
                .collect::<Vec<_>>(),
            "repeated calls must return identical id sequences"
        );

        // content is asserted as a set; the returned order is asserted above
        let mut parts: Vec<String> = permissions
            .iter()
            .map(|p| format!("{}:{}:{}", p.realm, p.resource, p.action))
            .collect();
        parts.sort();
        assert_eq!(
            parts,
            vec![
                "realmA:resourceA:actionA".to_owned(),
                "realmB:resourceB:actionB".to_owned()
            ]
        );

        let mut returned_ids: Vec<uuid::Uuid> = permissions
            .iter()
            .map(|p| p.id)
            .collect();
        returned_ids.sort();
        seeded.sort();
        assert_eq!(returned_ids, seeded, "ids must round-trip");
    }
}
