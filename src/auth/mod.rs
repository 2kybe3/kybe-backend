use crate::db::Database;
use crate::db::users::User;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{Error, SaltString};
use argon2::{Argon2, PasswordHasher};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("password hashing failed")]
    PasswordHashing(argon2::password_hash::Error),

    #[error("error creating user")]
    DatabaseError(sqlx::Error),
}

impl From<argon2::password_hash::Error> for AuthError {
    fn from(e: Error) -> Self {
        AuthError::PasswordHashing(e)
    }
}

pub struct Auth {
    database: Database,
}

impl Auth {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn register(
        &self,
        username: String,
        email: String,
        pass: String,
    ) -> Result<Uuid, AuthError> {
        let salt = SaltString::generate(&mut OsRng);
        let password_hash: String =
            Argon2::default().hash_password(pass.as_bytes(), &salt)?.to_string();

        let user = User { username, email, password_hash, ..Default::default() };

        match self.database.create_user(user).await {
            Ok(res) => Ok(res),
            Err(e) => Err(AuthError::DatabaseError(e)),
        }
    }
}
