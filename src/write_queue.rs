use crate::persistence::repository::Repository;
use crate::petty_matters::comment::{Comment, CommentId};
use crate::petty_matters::topic::{Topic, TopicId};
use std::fmt::Display;
use std::sync::Arc;
use tokio::sync::mpsc::{Receiver, Sender};

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

#[derive(Clone)]
pub struct WriteQueue {
    pub sender: Option<Sender<WriteOperation>>,
}

impl WriteQueue {
    pub fn new(sender: Sender<WriteOperation>) -> Self {
        Self {
            sender: Some(sender),
        }
    }

    #[allow(dead_code)]
    pub fn noop() -> Self {
        Self { sender: None }
    }

    pub async fn enqueue(&self, op: WriteOperation) -> Result<(), QueueError> {
        if let Some(sender) = &self.sender {
            sender
                .send(op)
                .await
                .map_err(|_| QueueError::SendError("Can't accept operation".to_string()))
        } else {
            Ok(())
        }
    }
}

pub async fn start_write_worker(
    mut receiver: Receiver<WriteOperation>,
    topic_repository: Arc<dyn Repository<TopicId, Topic> + Send + Sync>,
    comment_repository: Arc<dyn Repository<CommentId, Comment> + Send + Sync>,
) -> Result<(), QueueError> {
    while let Some(op) = receiver.recv().await {
        match op {
            WriteOperation::CreateTopic(topic) => {
                topic_repository
                    .save(topic)
                    .await
                    .map_err(|e| QueueError::OperationFailed(e.to_string()))?;
            }
            WriteOperation::AddComment(comment) => {
                comment_repository
                    .save(comment)
                    .await
                    .map_err(|e| QueueError::OperationFailed(e.to_string()))?;
            }
        }
    }
    Ok(())
}
