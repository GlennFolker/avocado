use crate::incl::*;

/// Send this event to request exit.
pub struct ExitEvent {
    pub reason: ExitReason,
}

impl ExitEvent {
    pub fn graceful() -> Self {
        Self { reason: ExitReason::Graceful }
    }

    pub fn error(msg: impl Into<Cow<'static, str>>) -> Self {
        Self { reason: ExitReason::Error(msg.into()) }
    }
}

#[derive(Clone)]
pub enum ExitReason {
    Graceful,
    Error(Cow<'static, str>),
}
