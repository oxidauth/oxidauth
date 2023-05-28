use crate::prelude::*;

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserRow {
    pub id: Uuid,
    pub username: String,
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub status: String,
    pub kind: String,
    pub profile: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, sqlx::FromRow)]
pub struct UserCreateRow {
    pub username: String,
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub status: Option<String>,
    pub kind: Option<String>,
    pub profile: Option<Value>,
}

#[derive(Clone, Debug, Deserialize, sqlx::FromRow)]
pub struct UserUpdateRow {
    pub id: Option<Uuid>,
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub status: Option<String>,
    pub profile: Option<Value>,
}
