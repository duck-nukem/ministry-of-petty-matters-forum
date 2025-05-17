use async_trait::async_trait;
use crate::error::Result;

pub struct PageNumber(pub usize);
pub struct PageSize(pub usize);

#[async_trait]
pub trait Repository<ID, Entity>
where
    ID: Send + Sync,
    Entity: Send + Sync,
{
    async fn list(&self, page: PageNumber, page_size: PageSize) -> Result<Vec<Entity>>;
    async fn save(&self, entity: Entity) -> Result<()>;
    async fn get_by_id(&self, id: &ID) -> Result<Option<Entity>>;
    async fn delete(&self, id: &ID) -> Result<()>;
}
