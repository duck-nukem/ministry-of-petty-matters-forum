use crate::authn::session::Username;
use crate::persistence::rdbms::{fetch_filtered_rows, ModelDatabaseInterface};
use crate::persistence::repository::{HasId, ListParameters, Page, Repository};
use crate::petty_matters::topic::{Topic, TopicId};
use async_trait::async_trait;
use chrono::Utc;
use sea_orm::entity::prelude::*;
use sea_orm::{Condition, DeriveEntityModel, IntoActiveModel, Order, Set};
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

impl ModelDatabaseInterface<Entity, Topic> for Entity {
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

    fn order_by_from_params(list_parameters: &ListParameters) -> (<Entity as EntityTrait>::Column, Order) {
        match &list_parameters.order_by {
            Some(order_by) => {
                let column = match order_by.as_str() {
                    "created_by" => Column::CreatedBy,
                    "creation_time" => Column::CreationTime,
                    "last_updated_time" => Column::LastUpdatedTime,
                    _ => Column::CreationTime,
                };
                (column, list_parameters.ordering.clone().unwrap_or_default().into())
            }
            None => (Column::CreationTime, Order::Desc),
        }
    }

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

    fn model_to_record(model: Topic) -> ActiveModel {
        ActiveModel {
            id: Set(model.id.0),
            title: Set(model.title),
            content: Set(model.content),
            upvotes_count: Set(model.upvotes_count as i32),
            downvotes_count: Set(model.downvotes_count as i32),
            created_by: Set(model.created_by.0),
            creation_time: Set(model.creation_time),
            last_updated_time: Set(Option::from(model.last_updated_time)),
        }
    }
}

pub struct TopicRepository<T> {
    pub db: DatabaseConnection,
    _marker: std::marker::PhantomData<T>,
}

impl<T> TopicRepository<T> {
    pub fn new(db: DatabaseConnection) -> Self {
        Self {
            db,
            _marker: std::marker::PhantomData,
        }
    }
}

#[async_trait]
impl<E> Repository<TopicId, Topic> for TopicRepository<E>
where
    E: Send + Sync + EntityTrait<Column = Column, Model = Model, ActiveModel = ActiveModel> + ModelDatabaseInterface<E, Topic>,
    <E as EntityTrait>::Model: Send + Sync, Model: IntoActiveModel<<E as EntityTrait>::ActiveModel>
{
    #[allow(clippy::cast_sign_loss)]
    async fn list(&self, list_parameters: ListParameters) -> crate::error::Result<Page<Topic>> {
        let (count, data) = fetch_filtered_rows(
            &self.db,
            Entity::filter_from_params(&list_parameters),
            &list_parameters,
            E::order_by_from_params(&list_parameters),
            Entity::find(),
        )
        .await?;

        Ok(Page {
            items: data
                .into_iter()
                .map(E::model_from_record)
                .collect(),
            size: list_parameters.page_size,
            current_page_number: list_parameters.page_number,
            total_count: count,
        })
    }

    #[allow(clippy::cast_possible_wrap)]
    async fn save(&self, entity: Topic) -> crate::error::Result<()> {
        let active_model: ActiveModel = E::model_to_record(entity.clone());

        if let Some(_id_already_exists) = &self.get_by_id(&entity.id).await? {
            Entity::update(active_model).exec(&self.db).await?;
        } else {
            Entity::insert(active_model).exec(&self.db).await?;
        }

        Ok(())
    }

    #[allow(clippy::cast_sign_loss)]
    async fn get_by_id(&self, id: &TopicId) -> crate::error::Result<Option<Topic>> {
        match Entity::find_by_id(id.0).one(&self.db).await {
            Ok(Some(record)) => Ok(Some(E::model_from_record(record))),
            Ok(None) | Err(_) => Ok(None),
        }
    }

    async fn delete(&self, id: &TopicId) -> crate::error::Result<()> {
        Entity::delete_by_id(id.0).exec(&self.db).await?;

        Ok(())
    }
}
