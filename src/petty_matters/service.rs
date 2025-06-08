use crate::authn::session::User;
use crate::error::AnyError;
use crate::feature_flags::FEATURE_FLAGS;
use crate::persistence::in_memory_repository::InMemoryRepository;
use crate::persistence::rdbms::RdbmsRepository;
use crate::persistence::repository::{ListParameters, Page, Repository, RepositoryError};
use crate::petty_matters::comment::{Comment, CommentId};
use crate::petty_matters::comment_repository::Entity as CommentDbModel;
use crate::petty_matters::topic::{Topic, TopicId};
use crate::petty_matters::topic_repository::Entity as TopicDbModel;
use crate::queue::base::{Queue, QueueError, WriteOperation};
use crate::queue::in_memory_queue::WriteQueue;
use crate::queue::worker::start_write_worker;
use moka::future::Cache;
use moka::policy::EvictionPolicy;
use sea_orm::{DatabaseConnection, DbErr};
use std::collections::BTreeMap;
use std::sync::{Arc, LazyLock};
use std::time::Duration;
use tokio::sync::mpsc::channel;

static CACHE: LazyLock<Cache<ListParameters, Page<Topic>>> = LazyLock::new(|| {
    Cache::builder()
        .eviction_policy(EvictionPolicy::tiny_lfu())
        .time_to_live(Duration::from_secs(30))
        .max_capacity(10_000)
        .build()
});

pub struct PettyMattersService<Q>
where
    Q: Queue + Send + Sync,
{
    pub topic_repository: Arc<dyn Repository<TopicId, Topic> + Send + Sync>,
    pub comment_repository: Arc<dyn Repository<CommentId, Comment> + Send + Sync>,
    pub write_queue: Arc<Q>,
}

impl<Q> PettyMattersService<Q>
where
    Q: Queue + Send + Sync,
{
    pub const fn new(
        topic_repository: Arc<dyn Repository<TopicId, Topic> + Send + Sync>,
        comment_repository: Arc<dyn Repository<CommentId, Comment> + Send + Sync>,
        write_queue: Arc<Q>,
    ) -> Self {
        Self {
            topic_repository,
            comment_repository,
            write_queue,
        }
    }

    pub async fn create_topic(&self, topic: Topic) -> Result<(), QueueError> {
        self.write_queue
            .enqueue(WriteOperation::CreateTopic(topic))
            .await
    }

    pub async fn get_topic(&self, topic_id: &TopicId) -> Result<Option<Topic>, RepositoryError> {
        self.topic_repository.get_by_id(topic_id).await
    }

    pub async fn list_topics(
        &self,
        list_parameters: ListParameters,
    ) -> Result<Page<Topic>, RepositoryError> {
        if let Some(cached) = CACHE.get(&list_parameters).await {
            return Ok(cached);
        }

        match self.topic_repository.list(list_parameters.clone()).await {
            Ok(page) => {
                CACHE.insert(list_parameters, page.clone()).await;
                Ok(page)
            }
            Err(e) => Err(e),
        }
    }

    pub async fn reply_to_topic(
        &self,
        topic_id: &TopicId,
        message: String,
        user: User,
    ) -> Result<(), QueueError> {
        if message.is_empty() {
            return Err(QueueError::InvalidInput(
                "Comment body cannot be empty".to_string(),
            ));
        }

        let comment = Comment::new(*topic_id, message, user);
        self.write_queue
            .enqueue(WriteOperation::AddComment(comment))
            .await
    }

    pub async fn list_comments(
        &self,
        for_topic: &TopicId,
        mut list_parameters: ListParameters,
    ) -> Result<Page<Comment>, RepositoryError> {
        list_parameters.filters = Some(BTreeMap::from([(
            "topic_id".to_string(),
            for_topic.to_string(),
        )]));
        self.comment_repository.list(list_parameters).await
    }
}

pub fn petty_matters_service_factory(
    db_connection: Result<DatabaseConnection, DbErr>,
) -> Result<Arc<PettyMattersService<WriteQueue>>, AnyError> {
    println!("Instantiating Petty Matters service");

    let topic_repository: Arc<dyn Repository<TopicId, Topic> + Send + Sync>;
    let comment_repository: Arc<dyn Repository<CommentId, Comment> + Send + Sync>;
    match db_connection {
        Ok(db) => {
            println!("Connection established");
            topic_repository = Arc::new(RdbmsRepository::<TopicDbModel>::new(db.clone()));
            comment_repository = Arc::new(RdbmsRepository::<CommentDbModel>::new(db));
        }
        Err(e) => {
            if FEATURE_FLAGS.is_ephemeral_db_allowed {
                eprintln!(
                    "Database connection failed: {e},
                    using in-memory repositories as fallback.
                    If you'd like to disallow the fallback behavior,
                    set the EPHEMERAL_DB_ALLOWED environment variable to false."
                );
                topic_repository = Arc::new(InMemoryRepository::<TopicId, Topic>::new());
                comment_repository = Arc::new(InMemoryRepository::<CommentId, Comment>::new());
            } else {
                eprintln!(
                    "Database connection failed: {e}
                    and ephemeral DB is not allowed, exiting.
                    To allow the application to run with an in-memory database,
                    set the EPHEMERAL_DB_ALLOWED environment variable to true."
                );
                return Err(e.into());
            }
        }
    }

    let (tx, rx) = channel(100);
    tokio::spawn(start_write_worker(
        rx,
        topic_repository.clone(),
        comment_repository.clone(),
    ));
    let topic_service = Arc::new(PettyMattersService::new(
        topic_repository,
        comment_repository,
        Arc::new(WriteQueue::new(tx)),
    ));
    println!("Service configuration done");

    Ok(topic_service)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistence::in_memory_repository::InMemoryRepository;
    use crate::petty_matters::topic::Topic;
    use crate::queue::stub_queue::StubQueue;

    fn setup_service() -> PettyMattersService<StubQueue> {
        let topic_repository = Arc::new(InMemoryRepository::new());
        let comment_repository = Arc::new(InMemoryRepository::new());
        let queue = StubQueue::new(topic_repository.clone(), comment_repository.clone());

        PettyMattersService::new(topic_repository, comment_repository, Arc::new(queue))
    }

    #[tokio::test]
    async fn test_start_topic_should_persist_a_topic() {
        let service = setup_service();
        let topic = Topic::default();

        service
            .create_topic(topic.clone())
            .await
            .expect("Failed to start topic");

        assert!(
            service
                .get_topic(&topic.id)
                .await
                .is_ok_and(|result| result.is_some_and(|entity| entity == topic))
        );
    }

    #[tokio::test]
    async fn test_should_add_comment_to_a_topic() {
        let service = setup_service();
        let topic = Topic::default();
        service
            .create_topic(topic.clone())
            .await
            .expect("Failed to start topic");

        service
            .reply_to_topic(
                &topic.id,
                "This is a comment".to_string(),
                User::anonymous(),
            )
            .await
            .expect("Failed to add comment");

        assert!(
            service
                .list_comments(&topic.id, ListParameters::default())
                .await
                .is_ok_and(|result| result
                    .items
                    .first()
                    .is_some_and(|c| c.content == "This is a comment"))
        );
    }

    #[tokio::test]
    async fn test_should_refuse_to_add_comments_without_a_body() {
        let service = setup_service();
        let topic = Topic::default();
        service
            .create_topic(topic.clone())
            .await
            .expect("Failed to start topic");

        let result = service
            .reply_to_topic(&topic.id, String::new(), User::anonymous())
            .await;

        assert!(matches!(result, Err(QueueError::InvalidInput(_))));
        assert!(
            service
                .list_comments(&topic.id, ListParameters::default())
                .await
                .is_ok_and(|topic_comments| topic_comments.items.len() == 0)
        );
    }

    #[tokio::test]
    async fn test_should_only_return_comments_relevant_for_the_topic() {
        let service = setup_service();
        let unrelated_topic = Topic::default();
        service
            .create_topic(unrelated_topic.clone())
            .await
            .expect("Failed to start topic");
        let topic = Topic::default();
        service
            .create_topic(topic.clone())
            .await
            .expect("Failed to start topic");

        service
            .reply_to_topic(
                &unrelated_topic.id,
                "This is a comment".to_string(),
                User::anonymous(),
            )
            .await
            .expect("Failed to add comment");

        assert!(
            service
                .list_comments(&topic.id, ListParameters::default())
                .await
                .is_ok_and(|result| result.items.is_empty())
        );
    }
}
