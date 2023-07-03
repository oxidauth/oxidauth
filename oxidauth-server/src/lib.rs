#![allow(unused)]

use std::error::Error;

pub mod authorities;
pub mod axum;
pub mod error;
pub mod jwt;
pub mod prelude;
pub mod rsa;

pub type BoxedError = Box<(dyn Error + Send + Sync + 'static)>;

pub mod traits {
    use async_trait::async_trait;
    use chrono::NaiveDateTime;
    use serde_json::Value;
    use uuid::Uuid;

    use crate::prelude::*;

    #[async_trait]
    pub trait Service {
        fn users(&self) -> dyn users::UserService;
    }

    pub mod users {
        use super::*;

        #[async_trait]
        pub trait UserService:
            all::UsersAllService
            + by_id::UserByIdService
            + by_username::UserByUsernameService
            + create::UserCreateService
            + update::UserUpdateService
            + delete_by_id::UserDeleteById
        {
        }

        impl<T> UserService for T where
            T: all::UsersAllService
                + by_id::UserByIdService
                + by_username::UserByUsernameService
                + create::UserCreateService
                + update::UserUpdateService
                + delete_by_id::UserDeleteById
        {
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
            pub created_at: NaiveDateTime,
            pub updated_at: NaiveDateTime,
        }

        pub mod all {
            use super::*;

            pub struct UsersAllRes {
                pub users: Vec<User>,
            }

            #[async_trait]
            pub trait UsersAllService {
                async fn all(&self) -> Result<UsersAllRes>;
            }
        }

        pub mod by_id {
            use super::*;

            pub struct UserByIdReq {
                pub user_id: Uuid,
            }

            pub struct UserByIdRes {
                pub user: User,
            }

            #[async_trait]
            pub trait UserByIdService {
                async fn by_id(&self, req: UserByIdReq) -> Result<UserByIdRes>;
            }
        }

        pub mod by_username {
            use super::*;

            pub struct UserByUsernameReq {
                pub username: String,
            }

            pub struct UserByUsernameRes {
                pub user: User,
            }

            #[async_trait]
            pub trait UserByUsernameService {
                async fn by_id(&self, req: UserByUsernameReq) -> Result<UserByUsernameRes>;
            }
        }

        pub mod create {
            use super::*;

            pub struct UserCreateReq {
                pub user: UserCreate,
            }

            pub struct UserCreate {
                pub username: String,
                pub email: Option<String>,
                pub first_name: Option<String>,
                pub last_name: Option<String>,
                pub status: Option<String>,
                pub kind: Option<String>,
                pub profile: Option<Value>,
            }

            pub struct UserCreateRes {
                pub user: User,
            }

            #[async_trait]
            pub trait UserCreateService {
                async fn create(&self, req: UserCreateReq) -> Result<UserCreateRes>;
            }
        }

        pub mod update {
            use super::*;

            pub struct UserUpdateReq {
                user: UserUpdate,
            }

            pub struct UserUpdate {
                pub id: Option<Uuid>,
                pub email: Option<String>,
                pub first_name: Option<String>,
                pub last_name: Option<String>,
                pub status: Option<String>,
                pub profile: Option<Value>,
            }

            pub struct UserUpdateRes {
                pub user: User,
            }

            #[async_trait]
            pub trait UserUpdateService {
                async fn update(&self, req: UserUpdateReq) -> Result<UserUpdateRes>;
            }
        }

        pub mod delete_by_id {
            use super::*;

            pub struct DeleteByIdReq {
                pub user_id: Uuid,
            }

            #[async_trait]
            pub trait UserDeleteById {
                async fn delete(&self, req: DeleteByIdReq) -> Result<()>;
            }
        }
    }
}
