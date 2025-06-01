use crate::authn::session::Username;
use crate::persistence::rdbms::ModelDatabaseInterface;
use crate::persistence::repository::{HasId, ListParameters};
use crate::petty_matters::topic::{Topic, TopicId};
use chrono::Utc;
use sea_orm::entity::prelude::*;
use sea_orm::{Condition, DeriveEntityModel, Order, Set};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "topics")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub upvotes_count: i32,
    pub downvotes_count: i32,
    pub created_by: String,
    pub creation_time: chrono::DateTime<Utc>,
    pub last_updated_time: Option<chrono::DateTime<Utc>>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl HasId<Uuid> for Model {
    fn id(&self) -> Uuid {
        self.id
    }
}

impl ModelDatabaseInterface<Self, Topic, TopicId> for Entity {
    fn filter_from_params(list_parameters: &ListParameters) -> Condition {
        let mut condition = Condition::all();
        if let Some(filters) = &list_parameters.filters {
            for (key, val) in filters {
                match key.as_str() {
                    "title" => condition = condition.add(Column::Title.like(val)),
                    "content" => condition = condition.add(Column::Content.eq(val)),
                    "created_by" => condition = condition.add(Column::CreatedBy.eq(val)),
                    _ => {}
                }
            }
        }

        condition
    }

    #[allow(clippy::match_same_arms)]
    fn order_by_from_params(
        list_parameters: &ListParameters,
    ) -> (<Self as EntityTrait>::Column, Order) {
        list_parameters
            .order_by
            .as_ref()
            .map_or((Column::CreationTime, Order::Desc), |order_by| {
                let column = match order_by.as_str() {
                    "created_by" => Column::CreatedBy,
                    "creation_time" => Column::CreationTime,
                    "last_updated_time" => Column::LastUpdatedTime,
                    _ => Column::CreationTime,
                };
                (
                    column,
                    list_parameters.ordering.clone().unwrap_or_default().into(),
                )
            })
    }

    #[allow(clippy::cast_sign_loss)]
    fn model_from_record(record: Model) -> Topic {
        Topic {
            id: TopicId(record.id),
            title: record.title,
            content: record.content,
            upvotes_count: record.upvotes_count as u32,
            downvotes_count: record.downvotes_count as u32,
            created_by: Username(record.created_by),
            creation_time: record.creation_time,
            last_updated_time: record.last_updated_time,
        }
    }

    #[allow(clippy::cast_possible_wrap)]
    fn model_to_record(model: Topic) -> ActiveModel {
        ActiveModel {
            id: Set(model.id.0),
            title: Set(model.title),
            content: Set(model.content),
            upvotes_count: Set(model.upvotes_count as i32),
            downvotes_count: Set(model.downvotes_count as i32),
            created_by: Set(model.created_by.0),
            creation_time: Set(model.creation_time),
            last_updated_time: Set(model.last_updated_time),
        }
    }

    fn unwrap_id(id: &TopicId) -> <<crate::petty_matters::comment_repository::Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType{
        id.0
    }
}
