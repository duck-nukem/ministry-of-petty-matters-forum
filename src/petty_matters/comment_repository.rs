use crate::authn::session::Username;
use crate::persistence::rdbms::ModelDatabaseInterface;
use crate::persistence::repository::{HasId, ListParameters};
use crate::petty_matters::comment::{Comment, CommentId};
use chrono::Utc;
use sea_orm::entity::prelude::*;
use sea_orm::{Condition, DeriveEntityModel, Order, Set};
use serde::{Deserialize, Serialize};
use crate::petty_matters::topic::TopicId;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "comments")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub topic_id: Uuid,
    pub content: String,
    pub upvotes_count: i32,
    pub downvotes_count: i32,
    pub created_by: String,
    pub creation_time: chrono::DateTime<Utc>,
    pub last_updated_time: Option<chrono::DateTime<Utc>>,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Topic,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Topic => Entity::belongs_to(super::topic_repository::Entity)
                .from(Column::TopicId)
                .to(super::topic_repository::Column::Id)
                .into(),
        }
    }
}

impl Related<super::topic_repository::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Topic.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl HasId<Uuid> for Model {
    fn id(&self) -> Uuid {
        self.id
    }
}

impl ModelDatabaseInterface<Self, Comment, CommentId> for Entity {
    fn filter_from_params(list_parameters: &ListParameters) -> Condition {
        let mut condition = Condition::all();
        if let Some(filters) = &list_parameters.filters {
            for (key, val) in filters {
                match key.as_str() {
                    "topic_id" => condition = condition.add(Column::TopicId.eq(TopicId::from(val).0)),
                    "created_by" => condition = condition.add(Column::CreatedBy.eq(val)),
                    "creation_time" => condition = condition.add(Column::CreationTime.gte(val)),
                    _ => {}
                }
            }
        }

        condition
    }

    fn order_by_from_params(
        list_parameters: &ListParameters,
    ) -> (<Self as EntityTrait>::Column, Order) {
        list_parameters
            .order_by
            .as_ref()
            .map_or((Column::CreationTime, Order::Desc), |order_by| {
                let column = match order_by.as_str() {
                    "created_by" => Column::CreatedBy,
                    _ => Column::CreationTime,
                };
                (
                    column,
                    list_parameters.ordering.clone().unwrap_or_default().into(),
                )
            })
    }

    #[allow(clippy::cast_sign_loss)]
    fn model_from_record(record: Model) -> Comment {
        Comment {
            id: CommentId(record.id),
            topic_id: super::topic::TopicId(record.topic_id),
            content: record.content,
            upvotes_count: record.upvotes_count as u32,
            downvotes_count: record.downvotes_count as u32,
            created_by: Username(record.created_by),
            creation_time: record.creation_time,
            last_updated_time: record.last_updated_time,
        }
    }

    #[allow(clippy::cast_possible_wrap)]
    fn model_to_record(model: Comment) -> <Self as EntityTrait>::ActiveModel {
        ActiveModel {
            id: Set(model.id.0),
            topic_id: Set(model.topic_id.0),
            content: Set(model.content),
            upvotes_count: Set(model.upvotes_count as i32),
            downvotes_count: Set(model.downvotes_count as i32),
            created_by: Set(model.created_by.to_string()),
            creation_time: Set(model.creation_time),
            last_updated_time: Set(Option::from(Utc::now())),
        }
    }

    fn id_to_primary_key(
        id: &CommentId,
    ) -> <<Self as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType {
        id.0
    }
}
