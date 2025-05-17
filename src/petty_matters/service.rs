use crate::error::Result;
use crate::persistence::repository::{PageNumber, PageSize, Repository};
use crate::petty_matters::topic::entity::{Topic, TopicId};
use std::sync::Arc;

type TopicRepository = dyn Repository<TopicId, Topic> + Send + Sync;

pub struct TopicService {
    pub topic_repository: Arc<TopicRepository>,
}

impl TopicService {
    pub fn new(topic_repository: Arc<TopicRepository>) -> Self {
        Self { topic_repository }
    }

    pub async fn create_topic(&self, topic: Topic) -> Result<()> {
        self.topic_repository.save(topic).await
    }
    
    pub async fn get_topic(&self, topic_id: &TopicId) -> Result<Option<Topic>> {
        self.topic_repository.get_by_id(topic_id).await
    }
    
    pub async fn list_topics(&self) -> Result<Vec<Topic>> {
        self.topic_repository.list(PageNumber(1), PageSize(50)).await
    }
}


#[cfg(test)]
mod tests {
    use crate::persistence::in_memory_repository::InMemoryRepository;
    use super::*;
    use crate::petty_matters::topic::entity::Topic;

    #[tokio::test]
    async fn test_start_topic_should_persist_a_topic() {
        let topic_repository = InMemoryRepository::new();
        let topic_service = TopicService::new(Arc::new(topic_repository));
        let topic = Topic::default();
        
        topic_service.create_topic(topic.clone()).await.expect("Failed to start topic");

        assert!(topic_service.get_topic(&topic.id).await.is_ok_and(|result| result.is_some_and(|entity| entity == topic)));
    }
}
