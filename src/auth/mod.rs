use std::str::FromStr;
use std::sync::Arc;

use crate::db::Database;
use crate::db::users::User;
use crate::email::IncomingEmail;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{Error, SaltString};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use thiserror::Error;
use tokio::sync::broadcast::Receiver;
use tokio::task::JoinHandle;
use tracing::info;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum AuthError {
	#[error("invalid crendetials")]
	InvalidCredentials,

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

	pub async fn launch_email_checker_loop(
		self: Arc<AuthService>,
		mut receiver: Receiver<IncomingEmail>,
	) -> JoinHandle<()> {
		tokio::spawn(async move {
			while let Ok(mail) = receiver.recv().await {
				let msg_id = mail.message_id.clone().unwrap_or_default();
				info!("started processing mail: {}", msg_id);
				let _ = self.process_mail(mail).await;
				info!("finished processing mail: {}", msg_id);
			}
		})
	}

	pub async fn process_mail(&self, mail: IncomingEmail) -> anyhow::Result<()> {
		if let Some(subject) = mail.subject
			&& let Some(first) = mail.from.first()
			&& let Some(addy) = &first.address
		{
			let uuid = Uuid::from_str(subject.trim())?;
			if let Some(user) = self
				.database
				.get_user_by_id_and_email(uuid, addy.trim().to_string())
				.await?
			{
				self.database.verify_user(user.id).await?;
			}
		}
		Ok(())
	}

	pub async fn login(&self, username: String, password: String) -> Result<Uuid, AuthError> {
		let user = match self.database.get_user_by_username(&username).await {
			Ok(Some(u)) => u,
			Ok(None) => return Err(AuthError::InvalidCredentials),
			Err(e) => return Err(AuthError::DatabaseError(e)),
		};

		let parsed_hash =
			PasswordHash::new(&user.password_hash).map_err(|_| AuthError::InvalidCredentials)?;

		if Argon2::default()
			.verify_password(password.as_bytes(), &parsed_hash)
			.is_err()
		{
			return Err(AuthError::InvalidCredentials);
		}

		Ok(user.id)
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
			.await
			.map_err(AuthError::DatabaseError)?
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
