use async_trait::async_trait;
use serde::Deserialize;
use crate::error::Result;

#[derive(Clone, Deserialize)]
pub struct PageNumber(pub usize);

#[derive(Clone, Deserialize)]
pub struct PageSize(pub usize);

pub struct ListParameters {
    pub page_size: PageSize,
    pub page_number: PageNumber,
}

#[async_trait]
#[allow(dead_code)]
pub trait Repository<ID, Entity>
where
    ID: Send + Sync,
    Entity: Send + Sync,
{
    async fn list(&self, list_parameters: ListParameters) -> Result<Vec<Entity>>;
    async fn save(&self, entity: Entity) -> Result<()>;
    async fn get_by_id(&self, id: &ID) -> Result<Option<Entity>>;
    async fn delete(&self, id: &ID) -> Result<()>;
}
