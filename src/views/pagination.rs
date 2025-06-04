use crate::persistence::repository::{PageNumber, PageSize};
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Ordering {
    #[serde(alias = "asc")]
    #[default]
    Ascending,
    #[serde(alias = "desc")]
    Descending,
}

#[derive(Clone, Deserialize, Debug)]
pub struct PageFilters {
    pub page: Option<PageNumber>,
    pub page_size: Option<PageSize>,
    pub order_by: Option<String>,
    pub ordering: Option<Ordering>,
    #[serde(flatten)]
    pub filters: BTreeMap<String, String>,
}
