use blackboard::create_blackboard;
use communication::socket_listener::SocketListener;
use blackboard::Blackboard;
use blackboard::store::Store;
use blackboard::task_queue::TaskQueue;
use blackboard::worker::Worker;
use communication::socket_listener::SocketListenerTrait;

pub mod blackboard;
pub mod model;
mod communication;

#[tokio::main]
async fn main() {
    // Create a blackboard
    let blackboard = create_blackboard();
    
    // Start listening for events
    let listener: SocketListener<Blackboard<TaskQueue, Worker, Store>> = SocketListener::new(blackboard, None);
    let res = listener.listen().await;
    match res {
        Ok(_) => {
            println!("Listening for events on port 1908...");
        }
        Err(e) => {
            eprintln!("Error starting listener: {}", e);
            return;
        }
    }
}