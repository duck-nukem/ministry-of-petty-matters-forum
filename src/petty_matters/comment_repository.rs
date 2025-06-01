use crate::authn::session::Username;
use crate::persistence::repository::{HasId, ListParameters, Page, Repository};
use crate::petty_matters::comment::{Comment, CommentId};
use async_trait::async_trait;
use chrono::Utc;
use sea_orm::entity::prelude::*;
use sea_orm::{Condition, DeriveEntityModel, IntoActiveModel, Order, Set};
use serde::{Deserialize, Serialize};
use crate::persistence::rdbms::{fetch_filtered_rows, ModelDatabaseInterface};
use crate::petty_matters::comment_repository;
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

impl ModelDatabaseInterface<Entity, Comment, CommentId> for Entity {
    fn filter_from_params(list_parameters: &ListParameters) -> Condition {
        let mut condition = Condition::all();
        if let Some(filters) = &list_parameters.filters {
            for (key, val) in filters {
                match key.as_str() {
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
                    _ => Column::CreationTime,
                };
                (column, list_parameters.ordering.clone().unwrap_or_default().into())
            }
            None => (Column::CreationTime, Order::Desc),
        }
    }

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

    fn model_to_record(model: Comment) -> <Entity as EntityTrait>::ActiveModel {
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

    fn unwrap_id(id: &CommentId) -> <<Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType {
        id.0
    }
}

pub struct CommentRepository<T> {
    pub db: DatabaseConnection,
    _marker: std::marker::PhantomData<T>,
}

impl<T> CommentRepository<T> {
    pub fn new(db: DatabaseConnection) -> Self {
        Self {
            db,
            _marker: std::marker::PhantomData,
        }
    }
}

#[async_trait]
impl<E> Repository<CommentId, Comment> for CommentRepository<E>
where
    E: Send + Sync
    + EntityTrait<Column = Column, Model = Model, ActiveModel = ActiveModel>
    + ModelDatabaseInterface<E, Comment, CommentId>,
    <E as EntityTrait>::Model: Send + Sync,
    Model: IntoActiveModel<<E as EntityTrait>::ActiveModel>,
{
    #[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
    async fn list(&self, list_parameters: ListParameters) -> crate::error::Result<Page<Comment>> {
        let (count, data) = fetch_filtered_rows(
            &self.db,
            E::filter_from_params(&list_parameters),
            &list_parameters,
            E::order_by_from_params(&list_parameters),
            E::find(),
        ).await?;
        
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
    async fn save(&self, entity: Comment) -> crate::error::Result<()> {
        let active_model: ActiveModel = E::model_to_record(entity.clone());

        if let Some(_id_already_exists) = &self.get_by_id(&entity.id).await? {
            Entity::update(active_model).exec(&self.db).await?;
        } else {
            Entity::insert(active_model).exec(&self.db).await?;
        }

        Ok(())
    }

    #[allow(clippy::cast_sign_loss)]
    async fn get_by_id(&self, id: &CommentId) -> crate::error::Result<Option<Comment>> {
        match Entity::find_by_id(id.0).one(&self.db).await {
            Ok(Some(record)) => Ok(Some(E::model_from_record(record))),
            Ok(None) | Err(_) => Ok(None),
        }
    }

    async fn delete(&self, id: &CommentId) -> crate::error::Result<()> {
        Entity::delete_by_id(id.0).exec(&self.db).await?;

        Ok(())
    }
}
