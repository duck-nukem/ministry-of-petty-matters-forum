use serde::Deserialize;
use std::collections::HashMap;
use crate::persistence::repository::{PageNumber, PageSize};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Ordering {
    #[serde(alias = "asc")]
    Ascending,
    #[serde(alias = "desc")]
    Descending,
}

#[derive(Clone, Deserialize, Debug)]
pub struct Pagination {
    pub page: Option<PageNumber>,
    pub page_size: Option<PageSize>,
    pub order_by: Option<String>,
    pub ordering: Option<Ordering>,
    #[serde(flatten)]
    pub filters: HashMap<String, String>,
}