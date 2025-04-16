use super::action::Action;

/// Events represent an incoming action from another agent of the coordination infrastructure.
#[derive(Debug)]
pub(crate) struct Event {
    pub(crate) from: Box<str>,
    pub(crate) action: Action,
}