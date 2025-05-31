use std::collections::HashMap;
use crate::error::Result;
use async_trait::async_trait;
use serde::Deserialize;

#[derive(Clone, Debug, Copy, Default, Deserialize, Eq, PartialEq)]
pub struct PageNumber(pub usize);

#[derive(Clone, Debug, Copy, Default, Deserialize)]
pub struct PageSize(pub usize);

#[derive(Debug)]
pub struct ListParameters {
    pub page_size: PageSize,
    pub page_number: PageNumber,
    pub filters: Option<HashMap<String, String>>
}

impl Default for ListParameters {
    fn default() -> Self {
        Self {
            page_size: PageSize(20),
            page_number: PageNumber(1),
            filters: None,
        }
    }
}

impl ListParameters {
    pub const fn calculate_offset(&self) -> usize {
        (self.page_number.0 - 1) * self.page_size.0
    }
    
    pub const fn calculate_limit(&self) -> usize {
        self.page_size.0
    }
}

pub trait Filterable {
    type Output;
    
    fn get_field_value(&self, field: &str) -> Self::Output;
}

#[derive(Default)]
pub struct Page<T> {
    pub current_page_number: PageNumber,
    pub size: PageSize,
    pub total_count: u64,
    pub items: Vec<T>,
}

impl<T> Page<T> {
    pub const fn is_first_page(&self) -> bool {
        self.current_page_number.0 == 1
    }
    
    pub const fn has_next_page(&self) -> bool {
        self.total_count > (self.current_page_number.0 * self.size.0) as u64
    }
    
    pub const fn get_next_page_number(&self) -> usize {
        if self.has_next_page() {
            self.current_page_number.0 + 1
        } else {
            0
        }
    }
    
    pub const fn get_previous_page_number(&self) -> usize {
        self.current_page_number.0.saturating_sub(1)
    }
}

#[async_trait]
#[allow(dead_code)]
pub trait Repository<ID, Entity>
where
    ID: Send + Sync,
    Entity: Send + Sync + HasId<ID> + Filterable,
{
    async fn list(&self, list_parameters: ListParameters) -> Result<Page<Entity>>;
    async fn save(&self, entity: Entity) -> Result<()>;
    async fn get_by_id(&self, id: &ID) -> Result<Option<Entity>>;
    async fn delete(&self, id: &ID) -> Result<()>;
}

pub trait HasId<ID> {
    fn id(&self) -> ID;
}