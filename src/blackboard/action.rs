
#[derive(Debug)]
pub(crate) struct Action {
    /// The action to perform
    action: ActionType,
    /// The value to perform the action on
    value: Box<str>,
}

#[derive(Debug)]
pub(crate) enum ActionType {
    Tell, Ask, Nask, Get
}