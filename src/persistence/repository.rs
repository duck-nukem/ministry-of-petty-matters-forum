use async_trait::async_trait;
use crate::error::Result;

#[async_trait]
pub trait Repository<ID, Entity>
where
    ID: Send + Sync,
    Entity: Send + Sync,
{
    async fn save(&self, entity: Entity) -> Result<()>;
    async fn get_by_id(&self, id: &ID) -> Result<Option<Entity>>;
    async fn delete(&self, id: &ID) -> Result<()>;
}
