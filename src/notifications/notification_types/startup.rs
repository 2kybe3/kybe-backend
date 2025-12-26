use crate::notifications::Notification;

pub struct StartupNotification {
    started: bool,
}

impl StartupNotification {
    pub fn new(started: bool) -> StartupNotification {
        Self { started }
    }
}

impl From<StartupNotification> for Notification {
    fn from(value: StartupNotification) -> Self {
        Notification::new("Backend", if value.started { "Started" } else { "Starting" })
    }
}
