use crate::queue::base::WriteOperation;
use crate::queue::base::{Queue, QueueError};
use async_trait::async_trait;
use tokio::sync::mpsc::Sender;

#[derive(Clone)]
pub struct WriteQueue {
    pub sender: Sender<WriteOperation>,
}

impl WriteQueue {
    pub const fn new(sender: Sender<WriteOperation>) -> Self {
        Self { sender }
    }
}

#[async_trait]
impl Queue for WriteQueue {
    async fn enqueue(&self, op: WriteOperation) -> Result<(), QueueError> {
        self.sender
            .send(op)
            .await
            .map_err(|e| QueueError::SendError(e.to_string()))
    }
}
