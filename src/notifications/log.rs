use tracing::info;
use crate::notifications::{Notification, Notifier};
use crate::notifications::error::NotificationError;

pub struct LogNotifier {
}

impl LogNotifier {
    pub fn new() -> Self {
        Self {  }
    }
}

#[async_trait::async_trait]
impl Notifier for LogNotifier {
    async fn send(&self, notification: &Notification) -> Result<(), NotificationError> {
        info!("{:?}", notification);

        Ok(())
    }

    fn name(&self) -> &'static str {
        "Log"
    }
}