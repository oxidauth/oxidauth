// Not sure if I need this?

// use async_trait::async_trait;
// use oxidauth_kernel::{
//     auth::Authenticator, authorities::Authority, error::BoxedError,
//     user_authorities::UserAuthority, JsonValue, Password,
// };
// use serde::Deserialize;

// use super::{
//     AuthorityParams, UserAuthorityParams,
// };

// #[derive(Clone, Deserialize)]
// pub struct AuthenticateParams {
// }

// impl TryFrom<JsonValue> for AuthenticateParams {
//     type Error = BoxedError;

//     fn try_from(value: JsonValue) -> Result<Self, Self::Error> {
//         let s: Self = serde_json::from_value(value.inner_value())?;

//         Ok(s)
//     }
// }

// #[async_trait]
// impl Authenticator for UsernamePassword {
//     #[tracing::instrument(name = "username_password authenticate", skip(self))]
//     async fn authenticate(
//         &self,
//         authenticate_params: JsonValue,
//         user_authority: &UserAuthority,
//     ) -> Result<(), BoxedError> {
//         let authenticate_params: AuthenticateParams =
//             authenticate_params.try_into()?;

//         let password = raw_password_hash(
//             &authenticate_params
//                 .password
//                 .inner_value(),
//             &self.params.password_salt,
//             &self.password_pepper,
//         );

//         let user_authority_params: UserAuthorityParams = user_authority
//             .params
//             .clone()
//             .try_into()?;

//         verify_password(
//             password,
//             user_authority_params.password_hash,
//         )
//         .map_err(|err| err.to_string())?;

//         Ok(())
//     }
// }

// #[tracing::instrument(name = "new authenticator")]
// pub async fn new(
//     authority: &Authority,
// ) -> Result<Box<dyn Authenticator>, BoxedError> {
//     let params: AuthorityParams = authority
//         .params
//         .clone()
//         .try_into()?;

//     let password_pepper = std::env::var("OXIDAUTH_USERNAME_PASSWORD_PEPPER")?;

//     let authority_id = authority.id;

//     Ok(Box::new(UsernamePassword {
//         authority_id,
//         params,
//         password_pepper,
//     }))
// }
