use crate::authn::session::{User, Username};
use crate::persistence::repository::DynamicAttributeValue;
use crate::persistence::repository::HasId;
use chrono::{DateTime, Utc};
use serde::{Deserialize};
use std::fmt::{Display, Formatter};
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Hash)]
pub struct TopicId(pub Uuid);

impl Display for TopicId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Topic {
    pub id: TopicId,
    pub title: String,
    pub content: String,
    pub upvotes_count: u32,
    pub downvotes_count: u32,
    pub created_by: Username,
    pub creation_time: DateTime<Utc>,
    pub last_updated_time: Option<DateTime<Utc>>,
}

impl Default for Topic {
    fn default() -> Self {
        Self {
            id: TopicId(Uuid::new_v4()),
            title: String::new(),
            content: String::new(),
            upvotes_count: 0,
            downvotes_count: 0,
            created_by: Username::default(),
            creation_time: Utc::now(),
            last_updated_time: None,
        }
    }
}

impl Topic {
    pub(crate) fn new(title: String, content: String, author: User) -> Self {
        Self {
            id: TopicId(Uuid::new_v4()),
            title,
            content,
            upvotes_count: 0,
            downvotes_count: 0,
            created_by: author.email,
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

impl DynamicAttributeValue for Topic {
    type Output = Option<String>;
    
    fn get_field_value(&self, field: &str) ->Self::Output {
        match field {
            "id" => Some(self.id.clone().to_string()),
            _ => None,
        }
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
