use crate::error::Result;
use crate::persistence::repository::{ListParameters, Page, Repository};
use async_trait::async_trait;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct InMemoryRepository<ID, Entity> {
    store: Arc<Mutex<HashMap<ID, Entity>>>,
}

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

pub trait HasId<ID> {
    fn id(&self) -> ID;
}

#[async_trait]
impl<ID, Entity> Repository<ID, Entity> for InMemoryRepository<ID, Entity>
where
    ID: Send + Sync + Eq + Hash + Clone,
    Entity: Send + Sync + Clone + HasId<ID>,
{
    async fn list(&self, list_parameters: ListParameters) -> Result<Page<Entity>> {
        let offset = (list_parameters.page_number.0 - 1) * list_parameters.page_size.0;
        let collection = self.store.lock().await;
        let key_value_pairs = collection.values();
        let page = Page {
            page_number: list_parameters.page_number,
            page_size: list_parameters.page_size.clone(),
            total_count: key_value_pairs.len(),
            items: key_value_pairs
                .skip(offset)
                .take(list_parameters.page_size.0)
                .cloned()
                .collect(),
        };
        Ok(page)
    }

    async fn save(&self, entity: Entity) -> Result<()> {
        self.store.lock().await.insert(entity.id(), entity);
        Ok(())
    }

    async fn get_by_id(&self, id: &ID) -> Result<Option<Entity>> {
        Ok(self.store.lock().await.get(id).cloned())
    }

    async fn delete(&self, id: &ID) -> Result<()> {
        self.store.lock().await.remove(id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::persistence::in_memory_repository::{HasId, InMemoryRepository};
    use crate::persistence::repository::{ListParameters, PageNumber, PageSize, Repository};

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

    #[tokio::test]
    async fn get_by_id_returns_result() {
        let repository: InMemoryRepository<StubId, StubEntity> = InMemoryRepository::new();
        let stub_entity = StubEntity::new(1);
        repository
            .save(stub_entity)
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
            .save(stub_entity.clone())
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
                .save(stub_entity)
                .await
                .expect("Failed to create entity");
        }

        let list_parameters = ListParameters {
            page_number: PageNumber(1),
            page_size: PageSize(2),
        };
        let page = repository
            .list(list_parameters)
            .await
            .expect("Failed to create entity");

        assert_eq!(page.items.clone().len(), 2);
    }
}
