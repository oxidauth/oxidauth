use crate::prelude::*;

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct PermissionRow {
    pub id: Uuid,
    pub realm: String,
    pub resource: String,
    pub action: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PermissionParts {
    pub realm: String,
    pub resource: String,
    pub action: String,
}
