use crate::views::pagination::{Ordering, PageFilters};
use async_trait::async_trait;
use axum::extract::Query;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::Display;
use std::hash::Hash;

#[derive(Clone, Debug, Copy, Default, Deserialize, Eq, PartialEq, Hash)]
pub struct PageNumber(pub usize);

#[derive(Clone, Debug, Copy, Default, Deserialize, Eq, PartialEq, Hash)]
pub struct PageSize(pub usize);

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ListParameters {
    pub page_size: PageSize,
    pub page_number: PageNumber,
    pub order_by: Option<String>,
    pub ordering: Option<Ordering>,
    pub filters: Option<BTreeMap<String, String>>,
}

impl Default for ListParameters {
    fn default() -> Self {
        Self {
            page_size: PageSize(20),
            page_number: PageNumber(1),
            filters: Option::default(),
            order_by: Option::default(),
            ordering: Option::default(),
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

    pub fn from_query_params(page_filters: &Query<PageFilters>) -> Self {
        Self {
            page_size: page_filters.page_size.unwrap_or(PageSize(20)),
            page_number: page_filters.page.unwrap_or(PageNumber(1)),
            filters: Some(page_filters.filters.clone()),
            order_by: page_filters.order_by.clone(),
            ordering: page_filters.ordering.clone(),
        }
    }
}

#[derive(Clone, Default)]
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RepositoryError {
    GenericError(String),
}

impl Display for RepositoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GenericError(msg) => write!(f, "Repository error: {msg}"),
        }
    }
}

impl Error for RepositoryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::GenericError(_) => None,
        }
    }
}

#[async_trait]
#[allow(dead_code)]
pub trait Repository<ID, Entity>
where
    ID: Send + Sync,
    Entity: Send + Sync + HasId<ID>,
{
    async fn list(&self, list_parameters: ListParameters) -> Result<Page<Entity>, RepositoryError>;
    async fn save(&self, entity: Entity) -> Result<(), RepositoryError>;
    async fn get_by_id(&self, id: &ID) -> Result<Option<Entity>, RepositoryError>;
    async fn delete(&self, id: &ID) -> Result<(), RepositoryError>;
}

pub trait HasId<ID> {
    fn id(&self) -> ID;
}
