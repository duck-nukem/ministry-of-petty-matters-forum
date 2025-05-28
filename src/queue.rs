use crate::persistence::repository::Repository;
use crate::petty_matters::comment::{Comment, CommentId};
use crate::petty_matters::topic::{Topic, TopicId};
use std::fmt::Display;
use std::sync::Arc;
use async_trait::async_trait;
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

#[async_trait]
pub trait Queue {
    async fn enqueue(&self, op: WriteOperation) -> Result<(), QueueError>;
}

#[derive(Clone)]
pub struct WriteQueue {
    pub sender: Sender<WriteOperation>,
}

impl WriteQueue {
    pub fn new(sender: Sender<WriteOperation>) -> Self {
        Self { sender }
    }
}

#[async_trait]
impl Queue for WriteQueue {
    async fn enqueue(&self, op: WriteOperation) -> Result<(), QueueError> {
        self
            .sender
            .send(op)
            .await
            .map_err(|e| QueueError::SendError(e.to_string()))
    }
}

#[derive(Clone)]
pub struct StubQueue {
    pub topic_repository: Arc<dyn Repository<TopicId, Topic> + Send + Sync>,
    pub comment_repository: Arc<dyn Repository<CommentId, Comment> + Send + Sync>,
}

impl StubQueue {
    pub fn new(
        topic_repository: Arc<dyn Repository<TopicId, Topic> + Send + Sync>,
        comment_repository: Arc<dyn Repository<CommentId, Comment> + Send + Sync>,
    ) -> Self {
        Self {
            topic_repository,
            comment_repository,
        }
    }
}

#[async_trait]
impl Queue for StubQueue {
    async fn enqueue(&self, op: WriteOperation) -> Result<(), QueueError> {
        match op {
            WriteOperation::CreateTopic(topic) => {
                self.topic_repository
                    .save(topic)
                    .await
                    .map_err(|e| QueueError::OperationFailed(e.to_string()))
            }
            WriteOperation::AddComment(comment) => {
                self.comment_repository
                    .save(comment)
                    .await
                    .map_err(|e| QueueError::OperationFailed(e.to_string()))
            }
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
