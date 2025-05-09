use std::ascii::escape_default;
use std::future::Future;
use mockall::automock;
use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream};
use crate::blackboard::{BlackboardTrait};

const DEFAULT_SOCKET_PORT: u16 = 2138; // BACH in alphabetical order

#[automock]
pub trait SocketListenerTrait<B: BlackboardTrait + 'static> {

    /// @summary - The constructor of the SocketListener
    ///
    /// @param port - The port to listen on (must not be 0)
    fn new(blackboard: B, port: Option<u16>) -> Self;

    /// @summary - The function that starts listening for incoming messages
    ///
    /// @returns - A Result indicating success or failure
    ///
    /// @note - It starts a thread that listens for incoming messages and parses them into events
    fn listen(&self) -> impl Future<Output=Result<(), String>>;

}

/// @summary - The SocketListener is responsible for listening to incoming message, and parse it into event.
pub struct SocketListener<B: BlackboardTrait> {
    port: u16,
    blackboard: B
}

impl<B: BlackboardTrait + Sync + Send + 'static> SocketListenerTrait<B> for SocketListener<B> {

    fn new(blackboard: B, port: Option<u16>) -> Self {
        let port = port.unwrap_or(DEFAULT_SOCKET_PORT);
        if port == 0 {
            panic!("Port must not be 0");
        }
        Self{
            port,
            blackboard
        }
    }

    async fn listen(&self) -> Result<(), String> {
        let addr = format!("127.0.0.1:{}", self.port);
        let socket = TcpListener::bind(&addr).await.map_err(|e| format!("Failed to bind socket: {}", e));
        let mut i = 0;
        match socket {
            Ok(listener) => {
                println!("Listening on {}", addr);
                loop {
                    let (stream, _) = listener.accept().await.map_err(|e| format!("Failed to accept connection: {}", e))?;
                    let cloned_bb = self.blackboard.clone();
                    let _ = tokio::spawn(async move {
                        handle_connection(stream, cloned_bb, i.clone().to_string()).await.unwrap_or_else(|e| {
                            eprintln!("Error handling connection: {}", e);
                        });
                    });
                    i += 1;
                };
                Ok(())
            },
            Err(e) => Err(format!("Failed to bind socket: {}", e)),

        }
    }
}

async fn handle_connection<B: BlackboardTrait>(mut stream: TcpStream, _blackboard: B, name: String) -> Result<(), String> {
    let mut buffer = vec![0; 1024];
    loop {
        let n = stream.read(&mut buffer).await.map_err(|e| format!("Failed to read from socket: {}", e))?;
        if n == 0 {
            break;
        }
        let message = String::from_utf8_lossy(&buffer[..n]);
        println!("[{}] Received message: {}",name, message);
    }
    println!("[{}] Connection dead",name);
    Ok(())
}