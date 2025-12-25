use crate::notifications::Notification;

pub struct StartupNotification {
    started: bool,
}

impl StartupNotification {
    pub fn new(started: bool) -> StartupNotification {
        Self {
            started,
        }
    }
}

impl Into<Notification> for StartupNotification {
    fn into(self) -> Notification {
        Notification::new("Backend", if self.started { "Started" } else { "Starting" })
    }
}