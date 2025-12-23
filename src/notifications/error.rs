#[derive(Debug)]
pub enum NotificationError {
    Transport,
    Auth,
    InvalidConfig,
    Other(String),
}