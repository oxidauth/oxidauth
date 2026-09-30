use oxidauth_kernel::roles::find_role_by_id::FindRoleById;
use oxidauth_repository::roles::select_role_by_id::*;

use super::*;

#[async_trait]
impl SelectRoleByIdQuery for PgRoleRepository {
    #[tracing::instrument(name = "select_role_by_id_query", skip(self))]
    async fn select_role_by_id(&self, params: &FindRoleById) -> Result<Role, BoxedError> {
        let pool = self.db.read_pool();
        let mut conn = pool.acquire().await?;

        let result = select_role_by_id_query(&mut conn, params.role_id).await?;

        let role = result.into();

        Ok(role)
    }
}

pub async fn select_role_by_id_query(
    conn: &mut PgConnection,
    role_id: Uuid,
) -> Result<PgRole, BoxedError> {
    let result = sqlx::query_as::<_, PgRole>(include_str!("./select_role_by_id.sql"))
        .bind(role_id)
        .fetch_one(conn)
        .await?;

    Ok(result)
}

#[cfg(test)]
mod tests {

    use oxidauth_kernel::roles::{create_role::CreateRole, find_role_by_id::FindRoleById};
    use oxidauth_repository::roles::insert_role::*;
    use sqlx::PgPool;

    use super::*;
    use crate::Database;

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_find_the_role_by_its_id(pool: PgPool) {
        let repo = PgRoleRepository::new(Database::from_pool(pool));

        let created = repo
            .insert_role(&CreateRole {
                name: "id-lookup-role".to_owned(),
            })
            .await
            .expect("seed insert should succeed");

        let found = repo
            .select_role_by_id(&FindRoleById {
                role_id: created.id,
            })
            .await
            .expect("the role should be found by id");

        assert_eq!(found.id, created.id);
        assert_eq!(found.name, "id-lookup-role");
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_surface_row_not_found_for_an_unknown_id(pool: PgPool) {
        let repo = PgRoleRepository::new(Database::from_pool(pool));

        let err = repo
            .select_role_by_id(&FindRoleById {
                role_id: uuid::Uuid::nil(),
            })
            .await
            .expect_err("an unknown id must surface an error");

        assert!(
            matches!(
                err.downcast_ref::<sqlx::Error>(),
                Some(sqlx::Error::RowNotFound)
            ),
            "expected sqlx::Error::RowNotFound, got: {err:?}"
        );
    }
}
