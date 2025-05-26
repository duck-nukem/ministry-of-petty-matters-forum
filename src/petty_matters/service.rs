use std::collections::HashMap;
use crate::error::Result;
use crate::persistence::repository::{ListParameters, Page, Repository};
use crate::petty_matters::topic::{Topic, TopicId};
use std::sync::Arc;
use crate::authn::session::User;
use crate::petty_matters::comment::{Comment, CommentId};

type TopicRepository = dyn Repository<TopicId, Topic> + Send + Sync;
type CommentRepository = dyn Repository<CommentId, Comment> + Send + Sync;

pub struct TopicService {
    pub topic_repository: Arc<TopicRepository>,
    pub comment_repository: Arc<CommentRepository>,
}


impl TopicService {
    pub fn new(
        topic_repository: Arc<TopicRepository>,
        comment_repository: Arc<CommentRepository>,
    ) -> Self {
        Self { topic_repository, comment_repository }
    }

    pub async fn create_topic(&self, topic: Topic) -> Result<()> {
        self.topic_repository.save(topic).await
    }

    pub async fn get_topic(&self, topic_id: &TopicId) -> Result<Option<Topic>> {
        self.topic_repository.get_by_id(topic_id).await
    }

    pub async fn list_topics(&self, list_parameters: ListParameters) -> Result<Page<Topic>> {
        self.topic_repository.list(list_parameters).await
    }

    pub async fn reply_to_topic(&self, topic_id: &TopicId, message: String, user: User) -> Result<()> {
        let comment = Comment::new(topic_id.clone(), message, user);
        self.comment_repository.save(comment).await
    }

    pub async fn list_comments(&self, for_topic: &TopicId, mut list_parameters: ListParameters) -> Result<Page<Comment>> {
        list_parameters.filters = Some(HashMap::from([
            ("topic_id".to_string(), for_topic.to_string()),
        ]));
        self.comment_repository.list(list_parameters).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistence::in_memory_repository::InMemoryRepository;
    use crate::petty_matters::topic::Topic;

    #[tokio::test]
    async fn test_start_topic_should_persist_a_topic() {
        let topic_repository = InMemoryRepository::new();
        let comment_repository = InMemoryRepository::new();
        let topic_service = TopicService::new(Arc::new(topic_repository), Arc::new(comment_repository));
        let topic = Topic::default();

        topic_service
            .create_topic(topic.clone())
            .await
            .expect("Failed to start topic");

        assert!(
            topic_service
                .get_topic(&topic.id)
                .await
                .is_ok_and(|result| result.is_some_and(|entity| entity == topic))
        );
    }

    #[tokio::test]
    async fn test_should_add_comment_to_a_topic() {
        let topic_repository = InMemoryRepository::new();
        let comment_repository = InMemoryRepository::new();
        let topic_service = TopicService::new(Arc::new(topic_repository), Arc::new(comment_repository));
        let topic = Topic::default();
        topic_service
            .create_topic(topic.clone())
            .await
            .expect("Failed to start topic");

        topic_service.reply_to_topic(&topic.id, "This is a comment".to_string(), User::anonymous()).await.expect("Failed to add comment");

        assert!(
            topic_service
                .list_comments(&topic.id, ListParameters::default())
                .await
                .is_ok_and(|result| result.items.first().is_some_and(|c| c.content == "This is a comment"))
        );
    }

    #[tokio::test]
    async fn test_should_only_return_comments_relevant_for_the_topic() {
        let topic_repository = InMemoryRepository::new();
        let comment_repository = InMemoryRepository::new();
        let topic_service = TopicService::new(Arc::new(topic_repository), Arc::new(comment_repository));
        let unrelated_topic = Topic::default();
        topic_service
            .create_topic(unrelated_topic.clone())
            .await
            .expect("Failed to start topic");
        let topic = Topic::default();
        topic_service
            .create_topic(topic.clone())
            .await
            .expect("Failed to start topic");

        topic_service.reply_to_topic(&unrelated_topic.id, "This is a comment".to_string(), User::anonymous()).await.expect("Failed to add comment");

        assert!(
            topic_service
                .list_comments(&topic.id, ListParameters::default())
                .await
                .is_ok_and(|result| result.items.is_empty())
        );
    }
}
