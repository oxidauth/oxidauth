use oxidauth_kernel::roles::find_role_by_name::FindRoleByName;
use oxidauth_repository::roles::select_role_by_name::*;

use super::*;

#[async_trait]
impl SelectRoleByNameQuery for PgRoleRepository {
    #[tracing::instrument(name = "select_role_by_name_query", skip(self))]
    async fn select_role_by_name(&self, params: &FindRoleByName) -> Result<Role, BoxedError> {
        let pool = self.db.read_pool();
        let mut conn = pool.acquire().await?;

        let result = select_role_by_name_query(&mut conn, &params.role).await?;

        let role = result.into();

        Ok(role)
    }
}

pub async fn select_role_by_name_query(
    conn: &mut PgConnection,
    role: &String,
) -> Result<PgRole, BoxedError> {
    let result = sqlx::query_as::<_, PgRole>(include_str!("./select_role_by_name.sql"))
        .bind(role)
        .fetch_one(conn)
        .await?;

    Ok(result)
}

#[cfg(test)]
mod tests {

    use oxidauth_kernel::roles::{create_role::CreateRole, find_role_by_name::FindRoleByName};
    use oxidauth_repository::roles::insert_role::*;
    use sqlx::PgPool;

    use super::*;
    use crate::Database;

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_find_the_role_by_its_name(pool: PgPool) {
        let repo = PgRoleRepository::new(Database::from_pool(pool));

        let created = repo
            .insert_role(&CreateRole {
                name: "lookup-role".to_owned(),
            })
            .await
            .expect("seed insert should succeed");

        let found = repo
            .select_role_by_name(&FindRoleByName {
                role: "lookup-role".to_owned(),
            })
            .await
            .expect("the role should be found by name");

        assert_eq!(found.id, created.id);
        assert_eq!(found.name, "lookup-role");
        assert!(found.created_at.timestamp() > 0);
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_surface_row_not_found_for_a_missing_role(pool: PgPool) {
        let repo = PgRoleRepository::new(Database::from_pool(pool));

        let err = repo
            .select_role_by_name(&FindRoleByName {
                role: "no-such-role".to_owned(),
            })
            .await
            .expect_err("a missing role must surface an error, not an empty role");

        assert!(
            matches!(
                err.downcast_ref::<sqlx::Error>(),
                Some(sqlx::Error::RowNotFound)
            ),
            "expected sqlx::Error::RowNotFound, got: {err:?}"
        );
    }
}
