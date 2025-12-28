use argon2::{Argon2, PasswordHasher};
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use crate::db::Database;

pub struct Auth {
    database: Database,
}

impl Auth {
    pub fn new(database: Database) -> Self {
        Self {
            database,
        }
    }

    pub async fn register(&self, username: String, email: String, password: String) -> Result<(), anyhow::Error> {
        let salt = SaltString::generate(&mut OsRng);

        let argon2 = Argon2::default();
        let password_hash = argon2.hash_password(password.as_bytes(), &salt).unwrap().to_string();

        //database.insert_user(&username, &email, &password_hash).await?;

        Ok(())
    }
}