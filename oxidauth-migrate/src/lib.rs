use sqlx::PgPool;

// NOTE(dewey4iv): eventually we want this to be able to handle logical
// migrations that use Rust code instead of just SQL schema changes.
// The new version should also support "checkpoints" for when an intall
// is brand new and only needs to load the schema.
pub async fn migrate(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::migrate!().run(pool).await?;

    Ok(())
}
