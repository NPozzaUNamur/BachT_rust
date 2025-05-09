use super::event::Event;
use tokio::sync::oneshot::{Sender, Receiver, channel};

/// Task represents a unit of work that will be processed by the event queue worker.
pub(crate) struct Task {
    pub(crate) event: Event,
    // Response channel, through which the event will send the result of the event
    pub(crate) res_chanel: Sender<Result<bool, TaskError>>,
}

impl Task {
    pub fn new(event: Event) -> (Self, Receiver<Result<bool, TaskError>>) {
        let (tx, rx) = channel::<Result<bool, TaskError>>();
        (
            Self {
                event,
                res_chanel: tx,
            },
            rx
        )
    }
}

#[derive(Debug)]
pub enum TaskError {
    UnspecifiedError,
    //TimeOutError,
    ChannelError,
}