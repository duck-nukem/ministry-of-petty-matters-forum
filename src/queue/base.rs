use std::fmt::Display;
use async_trait::async_trait;
use crate::petty_matters::comment::Comment;
use crate::petty_matters::topic::Topic;

pub enum WriteOperation {
    CreateTopic(Topic),
    AddComment(Comment),
}

#[derive(Debug)]
pub enum QueueError {
    SendError(String),
    OperationFailed(String),
}

impl Display for QueueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueueError::SendError(msg) => write!(f, "Send error: {}", msg),
            QueueError::OperationFailed(msg) => write!(f, "Operation failed: {}", msg),
        }
    }
}

impl std::error::Error for QueueError {}

#[async_trait]
pub trait Queue {
    async fn enqueue(&self, op: WriteOperation) -> Result<(), QueueError>;
}