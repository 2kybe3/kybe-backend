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

	#[error("username already taken")]
	UsernameTaken,

	#[error("email already taken")]
	EmailTaken,
}

impl From<argon2::password_hash::Error> for AuthError {
	fn from(e: Error) -> Self {
		AuthError::PasswordHashing(e)
	}
}

pub struct AuthService {
	database: Database,
}

impl AuthService {
	pub fn new(database: Database) -> Self {
		Self { database }
	}

	pub async fn register(
		&self,
		username: String,
		email: String,
		pass: String,
	) -> Result<Uuid, AuthError> {
		if self
			.database
			.get_user_by_username(&username)
			.await?
			.is_some()
		{
			return Err(AuthError::UsernameTaken);
		}
		if self.database.get_user_by_email(&email).await?.is_some() {
			return Err(AuthError::EmailTaken);
		}

		let salt = SaltString::generate(&mut OsRng);
		let password_hash: String = Argon2::default()
			.hash_password(pass.as_bytes(), &salt)?
			.to_string();

		let user = User {
			username,
			email,
			password_hash,
			..Default::default()
		};

		match self.database.create_user(user).await {
			Ok(res) => Ok(res),
			Err(e) => Err(AuthError::DatabaseError(e)),
		}
	}
}
