use crate::prelude::*;

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct RoleRow {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
