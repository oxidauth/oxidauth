use axum::{extract::State, response::IntoResponse, Json};
use oxidauth_kernel::users::user_create::*;
use serde::{Deserialize, Serialize};

use crate::provider::Provider;
use crate::response::Response;

#[derive(Serialize, Deserialize)]
pub struct CreateUserReq {
    pub user: UserCreate,
}

#[derive(Serialize, Deserialize)]
pub struct CreateUserRes {
    pub user: User,
}

#[axum_macros::debug_handler]
pub async fn handle(
    State(provider): State<Provider>,
    Json(params): Json<CreateUserReq>,
) -> impl IntoResponse {
    println!("got request");

    let service = provider.fetch::<CreateUserService>();

    let result = service
        .create_user(&params.user)
        .await;

    dbg!(&result);
    println!("user: {:?}", &result);

    match result {
        Ok(user) => Response::success().payload(CreateUserRes { user }),
        Err(err) => Response::fail().error(format!("{err:?}")),
    }
}
