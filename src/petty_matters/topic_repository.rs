use crate::authn::session::Username;
use crate::persistence::repository::{Filterable, HasId, ListParameters, Page, Repository};
use crate::petty_matters::topic::{Topic, TopicId};
use async_trait::async_trait;
use chrono::Utc;
use sea_orm::entity::prelude::*;
use sea_orm::{Condition, DeriveEntityModel, Order, Set};
use serde::{Deserialize, Serialize};
use crate::persistence::rdbms::fetch_filtered_rows;

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

impl Filterable for Model {
    type Output = Option<String>;

    fn get_field_value(&self, field: &str) -> Self::Output {
        match field {
            "id" => Some(self.id.clone().to_string()),
            "title" => Some(self.title.clone()),
            "created_by" => Some(self.created_by.clone()),
            _ => None,
        }
    }
}

impl HasId<Uuid> for Model {
    fn id(&self) -> Uuid {
        self.id
    }
}

pub struct TopicRepository {
    pub db: DatabaseConnection,
}

#[async_trait]
impl Repository<TopicId, Topic> for TopicRepository {
    #[allow(clippy::cast_sign_loss)]
    async fn list(&self, list_parameters: ListParameters) -> crate::error::Result<Page<Topic>> {
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
        
        let (count, data) = fetch_filtered_rows(
            &self.db, 
            condition.clone(),
            &list_parameters,
            (Column::CreationTime, Order::Desc),
            Entity::find(),
        ).await?;
        
        Ok(Page {
            items: data
                .into_iter()
                .map(|record| Topic {
                    id: TopicId(record.id),
                    title: record.title,
                    content: record.content,
                    upvotes_count: record.upvotes_count as u32,
                    downvotes_count: record.downvotes_count as u32,
                    created_by: Username(record.created_by),
                    creation_time: record.creation_time,
                    last_updated_time: record.last_updated_time,
                })
                .collect(),
            size: list_parameters.page_size,
            current_page_number: list_parameters.page_number,
            total_count: count,
        })
    }

    #[allow(clippy::cast_possible_wrap)]
    async fn save(&self, entity: Topic) -> crate::error::Result<()> {
        let mut active_model = ActiveModel {
            id: Set(entity.id.0),
            title: Set(entity.title),
            content: Set(entity.content),
            upvotes_count: Set(entity.upvotes_count as i32),
            downvotes_count: Set(entity.downvotes_count as i32),
            created_by: Set(entity.created_by.to_string()),
            creation_time: Set(entity.creation_time),
            last_updated_time: Set(entity.last_updated_time),
        };

        if let Some(_id_already_exists) = &self.get_by_id(&entity.id).await? {
            active_model.last_updated_time = Set(Option::from(Utc::now()));
            Entity::update(active_model).exec(&self.db).await?;
        } else {
            Entity::insert(active_model).exec(&self.db).await?;
        }

        Ok(())
    }

    #[allow(clippy::cast_sign_loss)]
    async fn get_by_id(&self, id: &TopicId) -> crate::error::Result<Option<Topic>> {
        match Entity::find_by_id(id.0).one(&self.db).await {
            Ok(Some(record)) => Ok(Some(Topic {
                id: TopicId(record.id),
                title: record.title,
                content: record.content,
                upvotes_count: record.upvotes_count as u32,
                downvotes_count: record.downvotes_count as u32,
                created_by: Username(record.created_by),
                creation_time: record.creation_time,
                last_updated_time: record.last_updated_time,
            })),
            Ok(None) | Err(_) => Ok(None),
        }
    }

    async fn delete(&self, id: &TopicId) -> crate::error::Result<()> {
        Entity::delete_by_id(id.0).exec(&self.db).await?;

        Ok(())
    }
}
