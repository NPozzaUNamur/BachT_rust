use std::future::Future;
use std::sync::{Arc, Mutex};
use mockall::automock;
use tokio::sync::Notify;
use tokio::sync::oneshot::Receiver;
use crate::model::event::Event;
use crate::model::task::{Task, TaskError};

#[automock]
pub trait TaskQueueTrait {

    /// @summary - The constructor of the TaskQueue
    fn new() -> Self;

    /// @summary - Constructor of the TaskQueue with predefined task queue and notify
    ///
    /// @param task_queue - The predefined task queue
    ///
    /// @param notify - The predefined notify
    fn new_with(task_queue: Arc<Mutex<Vec<Task>>>, notify: Arc<Notify>) -> Self;

    /// @summary - Allow to add an event to the queue w.r.t. FIFO Policy
    ///
    /// @param task_queue - The event queue to add the event to
    ///
    /// @param event - The event to add to the queue
    ///
    /// @returns - A promise of the reception channel to get the result of the task
    fn add_event_to_queue(&self, event: Event) -> Receiver<Result<bool, TaskError>>;

    /// @summary - Allow to get the task form the queue w.r.t. FIFO Policy
    ///
    /// @returns - The oldest task in the queue
    fn get_task(&self) -> Option<Task>;
    
    /// @summary - Notify the worker that there is a new task in the queue
    /// 
    /// @note - It is similar as task_queue.notifier.notified()
    fn notify(&self) -> impl Future<Output = ()> + Send;
    
    /// @summary - Allow to cancel the notification received
    /// 
    /// @note - Used to resend notification if the worker can't process the task (ex. The task must stop but receive notification)
    fn cancel_notification(&self);

    /// @summary - Allow to clone the TaskQueue
    ///
    /// @returns - A clone of the TaskQueue
    fn clone(&self) -> Self;
}

/// The TaskQueue hold incoming event to be processed by the worker in order to operate on the store.
/// This is inspired by: [T. Simmer's work](https://medium.com/@thomas.simmer/rust-build-a-simple-celery-like-worker-7ae90f170515)
pub struct TaskQueue {
    pub task_queue: Arc<Mutex<Vec<Task>>>,
    notifier: Arc<Notify>
}

impl TaskQueueTrait for TaskQueue {
    
    fn new() -> Self {
        Self {
            task_queue: Arc::new(Mutex::new(Vec::new())),
            notifier: Arc::new(Notify::new()),
        }
    }

    fn new_with(task_queue: Arc<Mutex<Vec<Task>>>, notifier: Arc<Notify>) -> Self {
        Self {
            task_queue,
            notifier
        }
    }
    
    fn add_event_to_queue(&self, event: Event) -> Receiver<Result<bool, TaskError>> {
        let (task, rx) = Task::new(event);
        let mut queue = self.task_queue.lock().unwrap();
        queue.insert(0, task);
        self.notifier.notify_one();
        rx
    }
    
    fn get_task(&self) -> Option<Task> {
        let mut queue = self.task_queue.lock().unwrap();
        queue.pop()
    }

    async fn notify(&self) {
        self.notifier.notified().await;
    }
    
    fn cancel_notification(&self) {
        self.notifier.notify_one();
    }

    fn clone(&self) -> Self {
        Self {
            task_queue: self.task_queue.clone(),
            notifier: self.notifier.clone()
        }
    }
}


/// ===============
/// |    TESTS    |
/// ===============

#[cfg(test)]
mod test {
    use tokio::{
        task,
        time::{timeout, sleep}
    };
    use std::time::Duration;
    use tokio::task::JoinHandle;
    use super::*;
    use crate::model::event::Event;
    use crate::model::action::Action::Tell;
    use crate::model::task::Task;
    use crate::model::task::TaskError::UnspecifiedError;

    // Test add event
    #[tokio::test]
    async fn queue_should_add_event_when_function_called() {
        let task_queue = TaskQueue::new();
        let event = Event::new(Tell("token".into()));

        task_queue.add_event_to_queue(event);

        let queue = task_queue.task_queue.lock().unwrap();

        assert_eq!(queue.len(), 1);
    }

    #[tokio::test]
    async fn queue_should_add_even_if_queue_is_filled() {
        let queue: Arc<Mutex<Vec<Task>>> = Arc::new(Mutex::new(Vec::new()));
        let notify = Arc::new(Notify::new());
        let mut locked_queue = queue.lock().unwrap();
        async {
            // [0;100[
            for i in 0..100 {
                println!("{:?}", i);
                let event = Event::new(Tell(format!("token{:?}", i).into()));
                let (task, _) = Task::new(event);
                locked_queue.push(task);
            }
        }.await;
        drop(locked_queue);
        let task_queue = TaskQueue::new_with(queue, notify);
        let event = Event::new(Tell("token".into()));
        task_queue.add_event_to_queue(event);
        let locked_queue = task_queue.task_queue.lock().unwrap();
        assert_eq!(locked_queue.len(), 101, "Queue should have 102 elements in stead of {:?}", locked_queue.len());
    }

    #[tokio::test]
    async fn queue_should_handle_two_threads_adding_tasks() {
        let task_queue = TaskQueue::new();
        let event1 = Event::new(Tell("token1".into()));
        let event2 = Event::new(Tell("token2".into()));

        let task_queue_clone1 = task_queue.clone();
        let task_queue_clone2 = task_queue.clone();

        let lock = task_queue.task_queue.lock().unwrap();

        let worker1: JoinHandle<()> = task::spawn(async move {
            task_queue_clone1.add_event_to_queue(event1);
        });

        let worker2: JoinHandle<()> = task::spawn(async move {
            task_queue_clone2.add_event_to_queue(event2);
        });

        drop(lock);

        let result1 = worker1.await;
        let result2 = worker2.await;

        let queue = task_queue.task_queue.lock().unwrap();

        assert!(result1.is_ok(), "Worker 1 task should have completed successfully");
        assert!(result2.is_ok(), "Worker 2 task should have completed successfully");
        assert_eq!(queue.len(), 2, "Queue should have 2 elements in stead of {:?}", queue.len());
    }

    // Test get task
    #[tokio::test]
    async fn queue_should_allow_getting_task() {
        let (task, _) = Task::new(Event::new(Tell("token".into())));
        let queue = Arc::new(Mutex::new(vec!(task)));
        let notify = Arc::new(Notify::new());
        let task_queue = TaskQueue::new_with(queue, notify);

        let task_from_queue = task_queue.get_task();

        assert!(task_from_queue.is_some(), "Task should not be None");
        match task_from_queue.unwrap().event.action {
            Tell(t) => {
                assert_eq!(t, "token".into(), "The token should be the same as the one added");
            },
            _ => {
                assert!(false, "Should be an Tell action");
            }
        }
    }

    #[tokio::test]
    async fn queue_should_return_none_if_no_task() {
        let queue = Arc::new(Mutex::new(Vec::new()));
        let notify = Arc::new(Notify::new());
        let task_queue = TaskQueue::new_with(queue, notify);

        let task_from_queue = task_queue.get_task();

        assert!(task_from_queue.is_none(), "Task should be None");
    }

    #[tokio::test]
    async fn queue_should_respect_fifo_policy() {
        let task_queue = TaskQueue::new();
        let event1 = Event::new(Tell("token1".into()));
        let event2 = Event::new(Tell("token2".into()));

        task_queue.add_event_to_queue(event1);
        task_queue.add_event_to_queue(event2);

        let task1 = task_queue.get_task();
        let task2 = task_queue.get_task();

        assert!(task1.is_some(), "Task 1 should not be None");
        assert!(task2.is_some(), "Task 2 should not be None");
        match task1.unwrap().event.action {
            Tell(t) => {
                assert_eq!(t, "token1".into(), "Task 1 should be the first one added");
            },
            _ => {
                assert!(false, "Should be an Tell action");
            }
        }
        match task2.unwrap().event.action {
            Tell(t) => {
                assert_eq!(t, "token2".into(), "Task 2 should be the second one added");
            },
            _ => {
                assert!(false, "Should be an Tell action");
            }
        }
    }

    // Test notify
    #[tokio::test]
    async fn queue_should_notify_when_event_added() {
        let task_queue = TaskQueue::new();

        let event = Event::new(Tell("token".into()));

        let task_queue_clone = task_queue.clone();

        let worker: JoinHandle<Result<(), ()>> = task::spawn(async move {
            // Timeout send error if the worker is not notified within 5 seconds
            match timeout(Duration::from_secs(5), task_queue_clone.notify()).await {
                Ok(_) => {Ok(())},
                Err(_) => {Err(())},
            }
        });

        task_queue.add_event_to_queue(event);

        let result = worker.await;
        assert!(result.is_ok(), "Worker task should have completed successfully");
        assert!(result.unwrap().is_ok(), "Worker should have been notified within the timeout period");
    }
    
    #[tokio::test]
    async fn queue_should_add_even_if_no_one_wait_notify() {
        let task_queue = TaskQueue::new();
        let event = Event::new(Tell("token".into()));

        task_queue.add_event_to_queue(event);

        let queue = task_queue.task_queue.lock().unwrap();

        assert_eq!(queue.len(), 1);
    }
    
    #[tokio::test]
    async fn queue_should_send_has_many_notification_as_receiving_event() {
        let task_queue = TaskQueue::new();
        let event1 = Event::new(Tell("token".into()));
        let event2 = Event::new(Tell("token2".into()));
        let event3 = Event::new(Tell("token3".into()));

        let worker1 = task::spawn({
            let task_queue = task_queue.clone();
            async move {
                task_queue.notify().await;
            }
        });

        let worker2 = task::spawn({
            let task_queue = task_queue.clone();
            async move {
                task_queue.notify().await;
            }
        });

        let worker3 = task::spawn({
            let task_queue = task_queue.clone();
            async move {
                task_queue.notify().await;
            }
        });

        // Wait for the workers to be ready (not best practice, should await for the worker to be ready)
        sleep(Duration::from_secs(1)).await;

        task_queue.add_event_to_queue(event1);
        task_queue.add_event_to_queue(event2);
        task_queue.add_event_to_queue(event3);

        let result1 = timeout(Duration::from_secs(2), worker1).await;
        let result2 = timeout(Duration::from_secs(2), worker2).await;
        let result3 = timeout(Duration::from_secs(2), worker3).await;

        assert!(result1.is_ok(), "Worker 1 task should have completed successfully");
        assert!(result2.is_ok(), "Worker 2 task should have completed successfully");
        assert!(result3.is_ok(), "Worker 3 task should have completed successfully");
    }

    #[tokio::test]
    async fn queue_should_notify_only_one_element_per_append() {
        let task_queue = TaskQueue::new();
        let event = Event::new(Tell("token".into()));

        let worker1 = task::spawn({
            let task_queue = task_queue.clone();
            async move {
                task_queue.notify().await;
            }
        });

        let worker2 = task::spawn({
            let task_queue = task_queue.clone();
            async move {
                task_queue.notify().await;
            }
        });

        // Wait for the workers to be ready (not best practice, should await for the worker to be ready)
        sleep(Duration::from_secs(1)).await;

        task_queue.add_event_to_queue(event);

        let result1 = timeout(Duration::from_secs(2), worker1).await;
        let result2 = timeout(Duration::from_secs(2), worker2).await;

        assert!(
            (result1.is_ok() && result2.is_err()) ||
            (result1.is_err() && result2.is_ok()),
            "One worker should have been notified and the other should not"
        );
    }

    // Test channel
    #[tokio::test]
    async fn queue_should_allow_sending_result_back() {
        let task_queue = TaskQueue::new();
        let event = Event::new(Tell("token".into()));

        let rx = task_queue.add_event_to_queue(event);

        // Simulate the worker processing the event and sending the result back
        let clone_task_queue = task_queue.clone();
        task::spawn(async move {
            // Simulate some processing
            let task = clone_task_queue.get_task().unwrap();
            task.res_chanel.send(Ok(true))
        });

        let result = rx.await;
        assert!(result.is_ok(), "Receiver should not be dropped");
    }

    #[tokio::test]
    async fn queue_should_allow_sending_error() {
        let task_queue = TaskQueue::new();
        let event = Event::new(Tell("token".into()));

        let rx = task_queue.add_event_to_queue(event);

        // Simulate the worker processing the event and sending the result back
        let clone_task_queue = task_queue.clone();
        task::spawn(async move {
            // Simulate some processing
            let task = clone_task_queue.get_task().unwrap();
            task.res_chanel.send(Err(UnspecifiedError))
        });

        let result_chanel = rx.await;
        let result = result_chanel.unwrap();
        assert!(result.is_err(), "Receiver should successfully receive an error");
    }

    #[tokio::test]
    async fn queue_should_handle_channel_communication_failure() {
        let task_queue = TaskQueue::new();
        let event = Event::new(Tell("token".into()));

        let rx = task_queue.add_event_to_queue(event);

        // Simulate the worker processing the event and sending the result back
        let clone_task_queue = task_queue.clone();
        task::spawn(async move {
            // Simulate some processing
            let task = clone_task_queue.get_task().unwrap();
            // Drop the receiver to simulate a communication failure
            drop(task.res_chanel);
        });

        let result = rx.await;
        assert!(result.is_err(), "Receiver should be dropped");
    }

}