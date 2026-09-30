use oxidauth_kernel::roles::create_role::CreateRole;
use oxidauth_repository::roles::insert_role::*;

use super::*;

#[async_trait]
impl InsertRoleQuery for PgRoleRepository {
    #[tracing::instrument(name = "insert_role_query", skip(self))]
    async fn insert_role(&self, params: &CreateRole) -> Result<Role, BoxedError> {
        let result = sqlx::query_as::<_, PgRole>(include_str!("./insert_role.sql"))
            .bind(None::<Uuid>)
            .bind(&params.name)
            .fetch_one(&self.db.write_pool())
            .await?;

        let role = result.into();

        Ok(role)
    }
}

#[cfg(test)]
mod tests {

    use oxidauth_kernel::roles::{create_role::CreateRole, find_role_by_id::FindRoleById};
    use oxidauth_repository::roles::select_role_by_id::*;
    use sqlx::PgPool;

    use super::*;
    use crate::Database;

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_insert_a_role_and_return_the_stored_row(pool: PgPool) {
        let repo = PgRoleRepository::new(Database::from_pool(pool.clone()));

        let role = repo
            .insert_role(&CreateRole {
                name: "admin-role".to_owned(),
            })
            .await
            .expect("insert_role should succeed");

        assert_eq!(&role.name, "admin-role");
        assert_ne!(role.id, uuid::Uuid::nil());

        // round-trip: the returned row is what was written
        let found = repo
            .select_role_by_id(&FindRoleById { role_id: role.id })
            .await
            .expect("the inserted role should be selectable");
        assert_eq!(found.id, role.id);
        assert_eq!(found.name, role.name);
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_reject_a_duplicate_role_name(pool: PgPool) {
        let repo = PgRoleRepository::new(Database::from_pool(pool.clone()));

        let params = CreateRole {
            name: "duplicate-role".to_owned(),
        };
        repo.insert_role(&params)
            .await
            .expect("first insert should succeed");

        let err = repo
            .insert_role(&params)
            .await
            .expect_err("duplicate name must violate the roles.name unique constraint");

        let sqlx_err = err
            .downcast_ref::<sqlx::Error>()
            .expect("the error should be a sqlx::Error");
        match sqlx_err {
            sqlx::Error::Database(db_err) => {
                assert_eq!(
                    db_err.code().as_deref(),
                    Some("23505"),
                    "expected unique_violation, got: {db_err}"
                );
            },
            other => panic!("expected a database error, got: {other:?}"),
        }

        // only the first row exists
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM roles WHERE name = $1")
            .bind("duplicate-role")
            .fetch_one(&pool)
            .await
            .expect("count query should run");
        assert_eq!(count, 1);
    }
}
