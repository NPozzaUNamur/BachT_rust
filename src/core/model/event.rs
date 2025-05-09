use super::action::Action;

/// Events represent an incoming action from another agent of the coordination infrastructure.
pub struct Event {
    pub action: Action
}

impl Event {
    pub fn new(action: Action) -> Self {
        Self {
            action
        }
        
    }
}