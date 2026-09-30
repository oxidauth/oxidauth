use oxidauth_repository::user_authorities::insert_user_authority::*;

use super::*;

#[async_trait]
impl InsertUserAuthorityQuery for PgUserAuthorityRepository {
    #[tracing::instrument(name = "insert_user_authority_query", skip(self))]
    async fn call(
        &self,
        params: impl Into<InsertUserAuthority> + Send + fmt::Debug + 'async_trait,
    ) -> Result<UserAuthority, BoxedError> {
        let params = params.into();

        let row = sqlx::query_as::<_, PgUserAuthority>(include_str!("./insert_user_authority.sql"))
            .bind(params.user_id)
            .bind(params.authority_id)
            .bind(&params.user_identifier)
            .bind(params.params.inner_value())
            .fetch_one(&self.db.write_pool())
            .await?;

        let user_authority = row.into();

        Ok(user_authority)
    }
}

#[cfg(test)]
mod tests {

    use oxidauth_kernel::JsonValue;
    use serde_json::json;
    use sqlx::PgPool;
    use uuid::Uuid;

    use super::*;
    use crate::{
        Database,
        test_fixtures::{assert_sql_state, seed_authority, seed_user},
    };

    fn repo(pool: &PgPool) -> PgUserAuthorityRepository {
        PgUserAuthorityRepository::new(Database::from_pool(pool.clone()))
    }

    // `InsertUserAuthorityQuery::call` is the trait method itself (the template
    // trait's own name, not generic dispatch); call it fully-qualified for clarity
    async fn insert(
        pool: &PgPool,
        params: InsertUserAuthority,
    ) -> Result<UserAuthority, BoxedError> {
        InsertUserAuthorityQuery::call(&repo(pool), params).await
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_insert_and_round_trip_params_json(pool: PgPool) {
        let user = seed_user(&pool).await;
        let authority = seed_authority(&pool, "params-round-trip").await;
        let params = json!({ "p": "argon2id$v=19$m=19456,t=2,p=1$anp6dGVzdA", "s": "c2FsdA" });

        let user_authority = insert(
            &pool,
            InsertUserAuthority {
                user_id: user,
                authority_id: authority,
                user_identifier: "alice@example.com".to_owned(),
                params: JsonValue::new(params.clone()),
            },
        )
        .await
        .expect("insert should succeed");

        assert_eq!(user_authority.user_id, user);
        assert_eq!(user_authority.authority_id, authority);
        assert_eq!(user_authority.user_identifier, "alice@example.com");
        assert_eq!(
            user_authority
                .params
                .inner_value(),
            params
        );
        assert!(
            user_authority
                .created_at
                .timestamp()
                > 0
        );
        assert!(user_authority.updated_at >= user_authority.created_at);

        let stored: serde_json::Value = sqlx::query_scalar(
            "SELECT params FROM user_authorities WHERE user_id = $1 AND authority_id = $2",
        )
        .bind(user)
        .bind(authority)
        .fetch_one(&pool)
        .await
        .expect("stored params should be readable");
        assert_eq!(
            stored, params,
            "the argon2 hash payload must round-trip through jsonb"
        );
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_reject_duplicate_user_authority_pair(pool: PgPool) {
        let user = seed_user(&pool).await;
        let authority = seed_authority(&pool, "dup-pair").await;
        let make = |identifier: &str| {
            InsertUserAuthority {
                user_id: user,
                authority_id: authority,
                user_identifier: identifier.to_owned(),
                params: JsonValue::new(json!({})),
            }
        };

        insert(&pool, make("alice@example.com"))
            .await
            .expect("first insert should succeed");

        let err = insert(&pool, make("alice-second@example.com"))
            .await
            .expect_err("the composite primary key must reject the duplicate pair");
        assert_sql_state(err, "23505");

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM user_authorities")
            .fetch_one(&pool)
            .await
            .expect("count should run");
        assert_eq!(count, 1);
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_reject_duplicate_identifier_within_one_authority(pool: PgPool) {
        // UNIQUE (user_identifier, authority_id) is what makes the identifier lookup
        // used by login deterministic within an authority
        let user_a = seed_user(&pool).await;
        let user_b = seed_user(&pool).await;
        let authority = seed_authority(&pool, "dup-identifier").await;
        let other_authority = seed_authority(&pool, "dup-identifier-other").await;

        insert(
            &pool,
            InsertUserAuthority {
                user_id: user_a,
                authority_id: authority,
                user_identifier: "shared-identifier".to_owned(),
                params: JsonValue::new(json!({})),
            },
        )
        .await
        .expect("first identifier should succeed");

        let err = insert(
            &pool,
            InsertUserAuthority {
                user_id: user_b,
                authority_id: authority,
                user_identifier: "shared-identifier".to_owned(),
                params: JsonValue::new(json!({})),
            },
        )
        .await
        .expect_err("UNIQUE (user_identifier, authority_id) must reject the clash");
        assert_sql_state(err, "23505");

        insert(
            &pool,
            InsertUserAuthority {
                user_id: user_b,
                authority_id: other_authority,
                user_identifier: "shared-identifier".to_owned(),
                params: JsonValue::new(json!({})),
            },
        )
        .await
        .expect("the same identifier under a different authority is allowed");
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_reject_unknown_user_or_authority_ids(pool: PgPool) {
        let user = seed_user(&pool).await;
        let authority = seed_authority(&pool, "fk-check").await;

        let err = insert(
            &pool,
            InsertUserAuthority {
                user_id: Uuid::new_v4(),
                authority_id: authority,
                user_identifier: "no-user@example.com".to_owned(),
                params: JsonValue::new(json!({})),
            },
        )
        .await
        .expect_err("an unknown user id must violate the users FK");
        assert_sql_state(err, "23503");

        let err = insert(
            &pool,
            InsertUserAuthority {
                user_id: user,
                authority_id: Uuid::new_v4(),
                user_identifier: "no-authority@example.com".to_owned(),
                params: JsonValue::new(json!({})),
            },
        )
        .await
        .expect_err("an unknown authority id must violate the authorities FK");
        assert_sql_state(err, "23503");
    }
}
