use crate::db::Database;
use chrono::Utc;
use uuid::Uuid;

#[derive(Debug, sqlx::Type, PartialEq, Clone, Default)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
pub enum UserRole {
    #[default]
    User,
    Admin,
}

impl UserRole {
    #[allow(unused)]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Admin => "admin",
            Self::User => "user",
        }
    }
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
#[allow(unused)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub email_verified: bool,

    pub password_hash: String,

    pub discord_id: Option<String>,
    pub discord_linked: Option<chrono::DateTime<Utc>>,

    pub last_password_change: chrono::DateTime<Utc>,
    pub created: chrono::DateTime<Utc>,
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
            created: Utc::now(),
            last_login: None,

            role: UserRole::default(),
        }
    }
}

impl Database {
    pub async fn create_user(&self, user: User) -> Result<Uuid, sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO users (
                id, username, email, email_verified, password_hash, discord_id,
                discord_linked, last_password_change, created, last_login, role
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
            user.created,
            user.last_login,
            user.role.as_str(),
        )
        .execute(self.pool())
        .await?;

        Ok(user.id)
    }
}
