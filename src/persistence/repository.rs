use crate::error::Result;
use async_trait::async_trait;
use serde::Deserialize;

#[derive(Clone, Deserialize, Eq, PartialEq)]
pub struct PageNumber(pub usize);

#[derive(Clone, Deserialize)]
pub struct PageSize(pub usize);

pub struct ListParameters {
    pub page_size: PageSize,
    pub page_number: PageNumber,
}

pub struct Page<T> {
    pub page_number: PageNumber,
    pub page_size: PageSize,
    pub total_count: usize,
    pub items: Vec<T>,
}

impl<T> Page<T> {
    pub fn is_first_page(&self) -> bool {
        self.page_number.0 == 1
    }
    
    pub fn has_next_page(&self) -> bool {
        self.total_count > self.page_number.0 * self.page_size.0
    }
}

#[async_trait]
#[allow(dead_code)]
pub trait Repository<ID, Entity>
where
    ID: Send + Sync,
    Entity: Send + Sync,
{
    async fn list(&self, list_parameters: ListParameters) -> Result<Page<Entity>>;
    async fn save(&self, entity: Entity) -> Result<()>;
    async fn get_by_id(&self, id: &ID) -> Result<Option<Entity>>;
    async fn delete(&self, id: &ID) -> Result<()>;
}
