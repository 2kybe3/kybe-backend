use crate::auth::AuthError;
use crate::db::Database;
use chrono::Utc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::error;
use uuid::Uuid;

#[derive(Debug, sqlx::Type, PartialEq, Clone, Default)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
pub enum UserRole {
    #[default]
    User,
    Admin,
}

impl TryFrom<&str> for UserRole {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, <UserRole as TryFrom<&str>>::Error> {
        match s.to_lowercase().as_str() {
            "admin" => Ok(Self::Admin),
            "user" => Ok(Self::User),
            other => Err(format!("Invalid user role: {}", other)),
        }
    }
}

#[derive(Debug)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub email_verified: bool,

    pub password_hash: String,

    pub discord_id: Option<String>,
    pub discord_linked: Option<chrono::DateTime<Utc>>,

    pub last_password_change: chrono::DateTime<Utc>,
    pub created_at: chrono::DateTime<Utc>,
    pub last_login: Option<chrono::DateTime<Utc>>,

    pub role: UserRole,
}

impl Default for User {
    fn default() -> Self {
        Self {
            id: Uuid::now_v7(),
            username: String::new(),
            email: String::new(),
            email_verified: false,

            password_hash: String::new(),

            discord_id: None,
            discord_linked: None,

            last_password_change: Utc::now(),
            created_at: Utc::now(),
            last_login: None,

            role: UserRole::default(),
        }
    }
}

impl Database {
    #[allow(unused)]
    pub async fn get_user_by_id(&self, id: Uuid) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as!(
            User,
            r#"
            SELECT
                id, username, email, email_verified, password_hash, discord_id,
                discord_linked, last_password_change, created_at, last_login,
                role AS "role: UserRole"
            FROM users
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(self.pool())
        .await
    }

    pub async fn get_user_by_username(&self, username: &str) -> Result<Option<User>, AuthError> {
        sqlx::query_as!(
            User,
            r#"
            SELECT
                id, username, email, email_verified, password_hash, discord_id,
                discord_linked, last_password_change, created_at, last_login,
                role AS "role: UserRole"
            FROM users
            WHERE username = $1
            "#,
            username
        )
        .fetch_optional(self.pool())
        .await
        .map_err(AuthError::DatabaseError)
    }

    pub async fn get_user_by_email(&self, email: &str) -> Result<Option<User>, AuthError> {
        sqlx::query_as!(
            User,
            r#"
            SELECT
                id, username, email, email_verified, password_hash, discord_id,
                discord_linked, last_password_change, created_at, last_login,
                role AS "role: UserRole"
            FROM users
            WHERE email = $1
            "#,
            email
        )
        .fetch_optional(self.pool())
        .await
        .map_err(AuthError::DatabaseError)
    }

    pub async fn create_user(&self, user: User) -> Result<Uuid, sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO users (
                id, username, email, email_verified, password_hash, discord_id,
                discord_linked, last_password_change, created_at, last_login, role
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
            user.id,
            user.username,
            user.email,
            user.email_verified,
            user.password_hash,
            user.discord_id,
            user.discord_linked,
            user.last_password_change,
            user.created_at,
            user.last_login,
            user.role.clone() as UserRole,
        )
        .execute(self.pool())
        .await?;

        Ok(user.id)
    }

    pub async fn delete_old_unverified_users_loop(&self) {
        let self_clone = self.clone();
        tokio::spawn(async move {
            loop {
                if let Err(e) = self_clone.delete_old_unverified_users().await {
                    error!("error deleting old unverified users: {:?}", e)
                }
                sleep(Duration::from_mins(1)).await;
            }
        });
    }

    pub async fn delete_old_unverified_users(&self) -> Result<u64, sqlx::Error> {
        let cutoff = Utc::now() - chrono::Duration::minutes(30);

        let result = sqlx::query!(
            r#"
            DELETE FROM users
            WHERE
                email_verified = false
            AND
                created_at < $1
            "#,
            cutoff
        )
        .execute(self.pool())
        .await?;

        Ok(result.rows_affected())
    }
}
