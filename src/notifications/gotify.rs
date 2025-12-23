use tracing::info;
use crate::notifications::{Notification, Notifier};
use crate::notifications::error::NotificationError;

#[derive(Debug)]
pub struct GotifyNotifier {
    url: String,
    token: String,
}

impl GotifyNotifier {
    pub fn new(url: String, token: String) -> Self {
        Self { url, token }
    }
}


impl Notifier for GotifyNotifier {
    fn send(&self, notification: &Notification) -> Result<(), NotificationError> {
        info!("Sending {:?} to {:?}", notification, self);
        todo!()
    }
}