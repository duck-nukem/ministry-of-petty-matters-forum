use crate::authn::session::{User, Username};
use crate::persistence::in_memory_repository::FilterableAttributes;
use crate::persistence::repository::HasId;
use crate::petty_matters::topic::TopicId;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::fmt::{Display, Formatter};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Hash)]
pub struct CommentId(pub Uuid);

impl Display for CommentId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Comment {
    pub id: CommentId,
    pub topic_id: TopicId,
    pub content: String,
    pub upvotes_count: u32,
    pub downvotes_count: u32,
    pub created_by: Username,
    pub creation_time: DateTime<Utc>,
    pub last_updated_time: Option<DateTime<Utc>>,
}

impl Comment {
    pub(crate) fn new(topic_id: TopicId, content: String, author: User) -> Self {
        Self {
            id: CommentId(Uuid::new_v4()),
            topic_id,
            content,
            upvotes_count: 0,
            downvotes_count: 0,
            created_by: author.email,
            creation_time: Utc::now(),
            last_updated_time: None,
        }
    }
}

impl HasId<CommentId> for Comment {
    fn id(&self) -> CommentId {
        self.id
    }
}

impl FilterableAttributes for Comment {
    type Output = Option<String>;

    fn get_field_value(&self, field: &str) -> Self::Output {
        match field {
            "id" => Some(self.id.to_string()),
            "topic_id" => Some(self.topic_id.to_string()),
            _ => None,
        }
    }
}
