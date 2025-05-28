use std::sync::Arc;
use async_trait::async_trait;
use crate::persistence::repository::Repository;
use crate::petty_matters::comment::{Comment, CommentId};
use crate::petty_matters::topic::{Topic, TopicId};
use crate::queue::base::{Queue, QueueError, WriteOperation};

#[derive(Clone)]
pub struct StubQueue {
    pub topic_repository: Arc<dyn Repository<TopicId, Topic> + Send + Sync>,
    pub comment_repository: Arc<dyn Repository<CommentId, Comment> + Send + Sync>,
}

#[allow(dead_code)]
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