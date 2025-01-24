pub mod models;
use crate::constants::db as constants;

pub async fn connect() -> Result<sea_orm::DatabaseConnection, sea_orm::DbErr> {
    sea_orm::Database::connect(constants::DB_URL.clone()).await
}

// use crate::constants::secrets::db::DB_URL;
// use argon2::password_hash::{
//     rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
// };
//
// pub enum CredentialType {
//     Password(String),
// }
//
// #[derive(sqlx::FromRow)]
// pub struct AppUser {
//     id: i64,
//     email: String,
//     age: i16,
//     forename: String,
//     surname: String
// }
//
// #[derive(sqlx::FromRow)]
// pub struct Password {
//     user_id: i64,
//     password: String,
// }
//
//
// pub fn verify_password(password: &str, hash: &str) -> bool {
//     let parsed_hash_result = argon2::PasswordHash::new(hash);
//     let Ok(parsed_hash) = parsed_hash_result else {
//         return false;
//     };
//     argon2::Argon2::new(
//         argon2::Algorithm::Argon2id,
//         argon2::Version::V0x13,
//         argon2::Params::new(12288, 3, 1, None).expect("INVALID ARGON2 PARAMETERS"),
//     )
//     .verify_password(password.as_bytes(), &parsed_hash)
//     .is_ok()
// }
//
// pub fn hash_password(password: &str) -> String {
//     let salt = SaltString::generate(&mut OsRng);
//     argon2::Argon2::new(
//         argon2::Algorithm::Argon2id,
//         argon2::Version::V0x13,
//         argon2::Params::new(1288, 3, 1, None).expect("INVALID ARGON2 PARAMETERS"),
//     )
//     .hash_password(password.as_bytes(), &salt)
//     .expect("Failed to hash password")
//     .serialize()
//     .as_str()
//     .to_string()
// }
// pub async fn get_connection_pool() -> sqlx::Result<sqlx::PgPool> {
//     sqlx::PgPool::connect(&DB_URL).await
// }
//
// pub async fn validate_credential(
//     pool: sqlx::PgPool,
//     user_id: i64,
//     credential: CredentialType,
// ) -> sqlx::Result<bool> {
//     match credential {
//         CredentialType::Password(password) => {
//             let res = sqlx::query_as!(
//                 Password,
//                 "SELECT * FROM authentication_svc.Password WHERE user_id = $1",
//                 user_id
//             )
//             .fetch_optional(&pool)
//             .await?;
//             Ok(match res {
//                 None => false,
//                 Some(row) => verify_password(&password, &row.password),
//             })
//         }
//     }
// }
//
// pub async fn login_user(
//     email: &str,
//     credential: CredentialType,
// ) -> Result<(), Result<(), sqlx::Error>> {
//     let appuser = sqlx::query_as!(
//         AppUser,
//         "SELECT * FROM appuser.data WHERE email = $1",
//         email
//     );
// }
