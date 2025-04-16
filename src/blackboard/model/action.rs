
#[derive(Debug)]
pub(crate) enum Action {
    Tell(Box<str>),
    Ask(Box<str>),
    Nask(Box<str>),
    Get(Box<str>)
}

impl PartialEq for Action {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Action::Tell(a), Action::Tell(b)) => a == b,
            (Action::Ask(a), Action::Ask(b)) => a == b,
            (Action::Nask(a), Action::Nask(b)) => a == b,
            (Action::Get(a), Action::Get(b)) => a == b,
            _ => false
        }
    }
}