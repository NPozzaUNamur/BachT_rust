pub(crate) trait SocketListenerTrait {
}

/// @summary - The SocketListener is responsible for listening to incoming message, and parse it into event.
pub(super) struct SocketListener {
    port: u16
}

impl SocketListenerTrait for SocketListener {
}

impl SocketListener {
    /// @summary - The constructor of the SocketListener
    pub(crate) fn new(port: u16) -> Self {
        if port == 0 {
            panic!("Port must not be 0");
        }
        Self{
            port
        }
    }
}