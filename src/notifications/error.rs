use std::fmt;
use std::fmt::Formatter;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum NotificationError {
    InvalidConfig(String),
    Transport(String),
    Other(String),
    Auth(String),
}

impl fmt::Display for NotificationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            NotificationError::InvalidConfig(msg) => write!(f, "Invalid config: {}", msg),
            NotificationError::Transport(msg) => write!(f, "Transport error: {}", msg),
            NotificationError::Other(msg) => write!(f, "Other error: {}", msg),
            NotificationError::Auth(msg) => write!(f, "Auth error: {}", msg),
        }
    }
}