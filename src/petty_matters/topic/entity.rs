use crate::persistence::in_memory_repository::HasId;
use chrono::{DateTime, Utc};

pub type TopicId = u32;

#[derive(Clone, Eq, PartialEq)]
pub struct Topic {
    pub id: TopicId,
    title: String,
    content: String,
    upvotes_count: u32,
    downvotes_count: u32,
    creation_time: DateTime<Utc>,
    last_updated_time: Option<DateTime<Utc>>,
}

impl Default for Topic {
    fn default() -> Self {
        Self {
            id: 0,
            title: String::new(),
            content: String::new(),
            upvotes_count: 0,
            downvotes_count: 0,
            creation_time: Utc::now(),
            last_updated_time: None,
        }
    }
}

impl HasId<TopicId> for Topic {
    fn id(&self) -> TopicId {
        self.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_topic_factory() {
        let rough_expected_creation_time = Utc::now();

        let topic = Topic::default();

        let is_recent_creation_time = rough_expected_creation_time
            .signed_duration_since(topic.creation_time)
            <= chrono::Duration::seconds(3);
        assert!(is_recent_creation_time);
        assert_eq!(topic.last_updated_time, None);
        assert_eq!(topic.upvotes_count, 0);
        assert_eq!(topic.downvotes_count, 0);
    }
}
