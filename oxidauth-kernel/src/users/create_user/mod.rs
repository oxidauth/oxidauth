use async_trait::async_trait;

use oxidauth_postgres::prelude::{DateTime, Utc, Uuid, Value};

pub struct CreateUserService<R: InsertUser> {
    db: R,
}

#[async_trait]
pub trait InsertUser {
    async fn insert_user(&self, user: CreateUser) -> Result<User, CreateUserError>;
}

pub struct CreateUser {
    pub username: String,
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub status: Option<String>,
    pub kind: Option<String>,
    pub profile: Option<Value>,
}

pub struct User {
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

pub enum CreateUserError {
    InsertError,
}

impl<R: InsertUser> CreateUserService<R> {
    pub async fn create_user(&self, user: CreateUser) -> Result<User, CreateUserError> {
        let user = self.db.insert_user(user).await?;

        Ok(user)
    }
}

pub mod postgres {
    use async_trait::async_trait;

    pub struct Database {}

    impl Database {
        async fn insert_user(&self, user: InsertUser<'_>) -> Result<UserRow, InsertUserError> {
            todo!()
        }
    }

    pub struct InsertUser<'a> {
        pub username: &'a String,
    }

    pub struct UserRow {}

    pub enum InsertUserError {}

    impl Into<super::CreateUserError> for InsertUserError {
        fn into(self) -> super::CreateUserError {
            super::CreateUserError::InsertError
        }
    }

    pub struct CreateUser(super::CreateUser);

    impl<'a> From<&'a super::CreateUser> for InsertUser<'a> {
        fn from(user: &'a super::CreateUser) -> Self {
            Self {
                username: &user.username,
            }
        }
    }

    #[async_trait]
    impl super::InsertUser for Database {
        async fn insert_user(
            &self,
            user: super::CreateUser,
        ) -> Result<super::User, super::CreateUserError> {
            let user = (&user).into();

            self.insert_user(user).await.map_err(Into::into)?;

            todo!()
        }
    }
}

pub mod memory {}
