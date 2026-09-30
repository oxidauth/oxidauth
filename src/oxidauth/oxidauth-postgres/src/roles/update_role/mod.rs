use oxidauth_kernel::roles::update_role::UpdateRole;
use oxidauth_repository::roles::update_role::*;

use super::*;

#[async_trait]
impl UpdateRoleQuery for PgRoleRepository {
    #[tracing::instrument(name = "update_role_query", skip(self))]
    async fn update_role(&self, params: &UpdateRole) -> Result<Role, BoxedError> {
        let result = sqlx::query_as::<_, PgRole>(include_str!("./update_role.sql"))
            .bind(params.role_id)
            .bind(&params.name)
            .fetch_one(&self.db.write_pool())
            .await?;

        let role = result.into();

        Ok(role)
    }
}

#[cfg(test)]
mod tests {

    use oxidauth_kernel::roles::{
        create_role::CreateRole,
        find_role_by_name::FindRoleByName,
        update_role::UpdateRole,
    };
    use oxidauth_repository::roles::{insert_role::*, select_role_by_name::*};
    use sqlx::PgPool;

    use super::*;
    use crate::Database;

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_overwrite_the_role_name(pool: PgPool) {
        let repo = PgRoleRepository::new(Database::from_pool(pool));

        let created = repo
            .insert_role(&CreateRole {
                name: "before-name".to_owned(),
            })
            .await
            .expect("seed insert should succeed");

        let updated = repo
            .update_role(&UpdateRole {
                role_id: Some(created.id),
                name: "after-name".to_owned(),
            })
            .await
            .expect("update should succeed");

        assert_eq!(updated.id, created.id, "update keeps the same row");
        assert_eq!(updated.name, "after-name");
        assert!(updated.updated_at >= updated.created_at);

        // overwrite semantics: the new name resolves, the old name is gone
        let found = repo
            .select_role_by_name(&FindRoleByName {
                role: "after-name".to_owned(),
            })
            .await
            .expect("the updated name should resolve");
        assert_eq!(found.id, created.id);

        let gone = repo
            .select_role_by_name(&FindRoleByName {
                role: "before-name".to_owned(),
            })
            .await
            .expect_err("the overwritten name must no longer resolve");
        assert!(
            matches!(
                gone.downcast_ref::<sqlx::Error>(),
                Some(sqlx::Error::RowNotFound)
            ),
            "expected sqlx::Error::RowNotFound, got: {gone:?}"
        );
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_error_when_the_role_id_is_missing(pool: PgPool) {
        let repo = PgRoleRepository::new(Database::from_pool(pool));

        // role_id binds NULL, so `WHERE id = $1` matches nothing and the
        // UPDATE ... RETURNING finds no row
        let err = repo
            .update_role(&UpdateRole {
                role_id: None,
                name: "orphan-name".to_owned(),
            })
            .await
            .expect_err("a NULL role_id must not update any row");

        assert!(
            matches!(
                err.downcast_ref::<sqlx::Error>(),
                Some(sqlx::Error::RowNotFound)
            ),
            "expected sqlx::Error::RowNotFound, got: {err:?}"
        );
    }
}
