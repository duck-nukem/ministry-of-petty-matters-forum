use crate::persistence::repository::{HasId, ListParameters, Page, Repository, RepositoryError};
use async_trait::async_trait;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct InMemoryRepository<ID, Entity> {
    store: Arc<Mutex<HashMap<ID, Entity>>>,
}

#[allow(dead_code)]
impl<ID, Entity> InMemoryRepository<ID, Entity>
where
    ID: Eq + Hash + Clone,
{
    pub fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl<ID, Entity> Repository<ID, Entity> for InMemoryRepository<ID, Entity>
where
    ID: Send + Sync + Eq + Hash + Clone,
    Entity: Send + Sync + Clone + HasId<ID> + FilterableAttributes<Output = Option<String>>,
{
    #[allow(clippy::significant_drop_tightening)]
    async fn list(&self, list_parameters: ListParameters) -> Result<Page<Entity>, RepositoryError> {
        let offset = list_parameters.calculate_offset();
        let collection = self.store.lock().await;
        let key_value_pairs = collection.values();
        let page = Page {
            current_page_number: list_parameters.page_number,
            size: list_parameters.page_size,
            total_count: key_value_pairs.len() as u64,
            items: key_value_pairs
                .filter(|entity| {
                    list_parameters.filters.as_ref().is_none_or(|filters| {
                        filters.iter().all(|(key, val)| {
                            entity.get_field_value(key).is_some_and(|v| v == *val)
                        })
                    })
                })
                .skip(offset)
                .take(list_parameters.calculate_limit())
                .cloned()
                .collect(),
        };
        Ok(page)
    }

    async fn create(&self, entity: Entity) -> Result<(), RepositoryError> {
        self.store.lock().await.insert(entity.id(), entity);
        Ok(())
    }

    async fn get_by_id(&self, id: &ID) -> Result<Option<Entity>, RepositoryError> {
        Ok(self.store.lock().await.get(id).cloned())
    }

    async fn delete(&self, id: &ID) -> Result<(), RepositoryError> {
        self.store.lock().await.remove(id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::persistence::in_memory_repository::{FilterableAttributes, InMemoryRepository};
    use crate::persistence::repository::{HasId, ListParameters, PageNumber, PageSize, Repository};

    type StubId = i32;

    #[derive(Clone)]
    struct StubEntity {
        id: StubId,
    }

    impl StubEntity {
        fn new(id: StubId) -> Self {
            Self { id }
        }
    }

    impl HasId<StubId> for StubEntity {
        fn id(&self) -> StubId {
            self.id
        }
    }

    impl FilterableAttributes for StubEntity {
        type Output = Option<String>;

        fn get_field_value(&self, _field: &str) -> Self::Output {
            None
        }
    }

    #[tokio::test]
    async fn get_by_id_returns_result() {
        let repository: InMemoryRepository<StubId, StubEntity> = InMemoryRepository::new();
        let stub_entity = StubEntity::new(1);
        repository
            .create(stub_entity)
            .await
            .expect("Failed to create entity");

        let result = repository
            .get_by_id(&1)
            .await
            .expect("Failed to retrieve entity");

        assert!(result.is_some_and(|e| e.id() == 1));
    }

    #[tokio::test]
    async fn get_by_id_returns_none_if_not_found() {
        let repository: InMemoryRepository<StubId, StubEntity> = InMemoryRepository::new();

        let result = repository
            .get_by_id(&1)
            .await
            .expect("Failed to retrieve entity");

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn delete_by_id_removes_entity() {
        let repository: InMemoryRepository<StubId, StubEntity> = InMemoryRepository::new();
        let stub_entity = StubEntity::new(1);
        repository
            .create(stub_entity.clone())
            .await
            .expect("Failed to create entity");
        repository
            .delete(&stub_entity.id)
            .await
            .expect("Failed to create entity");

        let result = repository
            .get_by_id(&1)
            .await
            .expect("Failed to retrieve entity");

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn list_yields_a_collection_of_give_size_with_an_offset() {
        let repository: InMemoryRepository<StubId, StubEntity> = InMemoryRepository::new();
        for iteration in 1..10 {
            let stub_entity = StubEntity::new(iteration);
            repository
                .create(stub_entity)
                .await
                .expect("Failed to create entity");
        }

        let list_parameters = ListParameters {
            page_number: PageNumber(1),
            page_size: PageSize(2),
            filters: None,
            order_by: None,
            ordering: None,
        };
        let page = repository
            .list(list_parameters)
            .await
            .expect("Failed to create entity");

        assert_eq!(page.items.clone().len(), 2);
    }
}

/// Used by in-memory repositories to perform filtering
pub trait FilterableAttributes {
    type Output;

    fn get_field_value(&self, field: &str) -> Self::Output;
}
