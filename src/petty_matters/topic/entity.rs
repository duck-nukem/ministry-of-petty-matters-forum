use std::fmt::{Display, Formatter};
use crate::persistence::in_memory_repository::HasId;
use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Clone, Deserialize, Eq, PartialEq, Hash)]
pub struct TopicId(pub u32);

impl Display for TopicId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct Topic {
    pub id: TopicId,
    pub title: String,
    pub content: String,
    pub upvotes_count: u32,
    pub downvotes_count: u32,
    pub creation_time: DateTime<Utc>,
    pub last_updated_time: Option<DateTime<Utc>>,
}

impl Default for Topic {
    fn default() -> Self {
        Self {
            id: TopicId(0),
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
        self.id.clone()
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
