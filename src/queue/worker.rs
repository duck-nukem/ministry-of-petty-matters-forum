use std::sync::Arc;
use tokio::sync::mpsc::Receiver;
use crate::persistence::repository::Repository;
use crate::petty_matters::comment::{Comment, CommentId};
use crate::petty_matters::topic::{Topic, TopicId};
use crate::queue::base::{QueueError, WriteOperation};

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