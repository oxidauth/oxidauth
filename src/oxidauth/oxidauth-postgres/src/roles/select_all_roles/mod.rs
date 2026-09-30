use oxidauth_kernel::roles::list_all_roles::ListAllRoles;
use oxidauth_repository::roles::select_all_roles::*;

use super::*;

#[async_trait]
impl SelectAllRolesQuery for PgRoleRepository {
    #[tracing::instrument(name = "select_all_roles_query", skip(self))]
    async fn select_all_roles(&self, _params: &ListAllRoles) -> Result<Vec<Role>, BoxedError> {
        let result = sqlx::query_as::<_, PgRole>(include_str!("./select_all_roles.sql"))
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

    use oxidauth_kernel::roles::{create_role::CreateRole, list_all_roles::ListAllRoles};
    use oxidauth_repository::roles::insert_role::*;
    use sqlx::PgPool;

    use super::*;
    use crate::Database;

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_return_an_empty_list_for_an_empty_table(pool: PgPool) {
        let repo = PgRoleRepository::new(Database::from_pool(pool));

        let roles = repo
            .select_all_roles(&ListAllRoles)
            .await
            .expect("select on an empty table should succeed");

        assert!(roles.is_empty(), "fresh test db should have no roles");
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_return_every_stored_role(pool: PgPool) {
        let repo = PgRoleRepository::new(Database::from_pool(pool.clone()));

        let mut ids = Vec::new();
        for name in ["alpha-role", "beta-role"] {
            let role = repo
                .insert_role(&CreateRole {
                    name: name.to_owned(),
                })
                .await
                .expect("seed insert should succeed");
            ids.push(role.id);
        }

        // Guard (OXA-000019 precedent): force the tie — seed rows tie on created_at
        // only if NOW() never ticks between INSERTs, and while updated_at equals
        // created_at an updated_at-first orderer is indistinguishable from the
        // correct one. Pin every row's created_at to the newest one, then drop the
        // max-id row's updated_at below that tie: `ORDER BY updated_at ASC, id ASC`
        // must then hoist the max-id row to the front — observably wrong — while
        // the correct orderer never reads updated_at and keeps it last.
        sqlx::query(
            "UPDATE roles SET created_at = (SELECT max(created_at) FROM roles), \
             updated_at = CASE WHEN id = (SELECT id FROM roles ORDER BY id DESC LIMIT 1) \
             THEN (SELECT max(created_at) FROM roles) - INTERVAL '1 hour' \
             ELSE (SELECT max(created_at) FROM roles) END",
        )
        .execute(&pool)
        .await
        .expect("diverge one row's updated_at");

        let roles = repo
            .select_all_roles(&ListAllRoles)
            .await
            .expect("select_all_roles should succeed");

        assert_eq!(roles.len(), 2, "both seeded roles should come back");

        assert!(
            roles
                .windows(2)
                .all(|w| (w[0].created_at, w[0].id) <= (w[1].created_at, w[1].id)),
            "rows must be returned in (created_at, id) order"
        );

        let again = repo
            .select_all_roles(&ListAllRoles)
            .await
            .expect("repeated query should succeed");
        assert_eq!(
            roles
                .iter()
                .map(|r| r.id)
                .collect::<Vec<_>>(),
            again
                .iter()
                .map(|r| r.id)
                .collect::<Vec<_>>(),
            "repeated calls must return identical id sequences"
        );

        // content is asserted as a set; the returned order is asserted above
        let mut names: Vec<String> = roles
            .iter()
            .map(|r| r.name.clone())
            .collect();
        names.sort();
        assert_eq!(names, vec!["alpha-role".to_owned(), "beta-role".to_owned()]);

        let mut returned_ids: Vec<uuid::Uuid> = roles
            .iter()
            .map(|r| r.id)
            .collect();
        returned_ids.sort();
        ids.sort();
        assert_eq!(returned_ids, ids);
    }
}
