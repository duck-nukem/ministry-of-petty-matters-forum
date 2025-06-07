use crate::persistence::repository::{HasId, ListParameters, Page, Repository};
use crate::views::pagination::Ordering;
use async_trait::async_trait;
use sea_orm::sea_query::Expr;
use sea_orm::{Condition, DatabaseConnection, DeriveColumn, EntityTrait, EnumIter};
use sea_orm::{IntoActiveModel, Order, PrimaryKeyTrait, QueryFilter, QueryOrder, QuerySelect};

pub trait ModelDatabaseInterface<E: EntityTrait, M, Id> {
    fn filter_from_params(list_parameters: &ListParameters) -> Condition;
    fn order_by_from_params(list_parameters: &ListParameters) -> (E::Column, Order);
    fn model_from_record(record: E::Model) -> M;
    fn model_to_record(model: M) -> E::ActiveModel;
    fn id_to_primary_key(id: &Id) -> <<E>::PrimaryKey as PrimaryKeyTrait>::ValueType;
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
enum Counter {
    Count,
}

impl From<Ordering> for Order {
    fn from(o: Ordering) -> Self {
        match o {
            Ordering::Ascending => Self::Asc,
            Ordering::Descending => Self::Desc,
        }
    }
}

pub struct RdbmsRepository<E> {
    db: DatabaseConnection,
    _marker: std::marker::PhantomData<E>,
}

impl<E> RdbmsRepository<E> {
    pub const fn new(db: DatabaseConnection) -> Self {
        Self {
            db,
            _marker: std::marker::PhantomData,
        }
    }
}

#[async_trait]
impl<DbRecord, Id, ModelType> Repository<Id, ModelType> for RdbmsRepository<DbRecord>
where
    Id: Send + Sync + Clone,
    ModelType: Send + Sync + Clone + HasId<Id> + 'static,
    DbRecord: Send
        + Sync
        + EntityTrait<Model: IntoActiveModel<<DbRecord as EntityTrait>::ActiveModel>>
        + ModelDatabaseInterface<DbRecord, ModelType, Id>,
    <DbRecord as EntityTrait>::Model: Send + Sync,
    <DbRecord as EntityTrait>::ActiveModel: Send + 'static,
{
    #[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
    async fn list(&self, list_parameters: ListParameters) -> crate::error::Result<Page<ModelType>> {
        let resulting_rows =
            DbRecord::find().filter(DbRecord::filter_from_params(&list_parameters));
        // Workaround: .count() is ambiguous, it wants to use an iterable count
        let count: Option<i64> = resulting_rows
            .clone()
            .select_only()
            .column_as(Expr::val(1).count(), "count")
            .into_values::<_, Counter>()
            .one(&self.db)
            .await?;
        let (order_by_column, order_direction) = DbRecord::order_by_from_params(&list_parameters);
        let data = resulting_rows
            .offset(Some(list_parameters.calculate_offset() as u64))
            .limit(Some(list_parameters.calculate_limit() as u64))
            .order_by(order_by_column, order_direction)
            .all(&self.db)
            .await?;

        Ok(Page {
            items: data.into_iter().map(DbRecord::model_from_record).collect(),
            size: list_parameters.page_size,
            current_page_number: list_parameters.page_number,
            total_count: count.unwrap_or_default() as u64,
        })
    }

    #[allow(clippy::cast_possible_wrap)]
    async fn save(&self, entity: ModelType) -> crate::error::Result<()> {
        let active_model: DbRecord::ActiveModel = DbRecord::model_to_record(entity.clone());

        if let Some(_id_already_exists) = &self.get_by_id(&entity.id()).await? {
            DbRecord::update(active_model).exec(&self.db).await?;
        } else {
            DbRecord::insert(active_model).exec(&self.db).await?;
        }

        Ok(())
    }

    #[allow(clippy::cast_sign_loss)]
    async fn get_by_id(&self, id: &Id) -> crate::error::Result<Option<ModelType>> {
        match DbRecord::find_by_id(DbRecord::id_to_primary_key(id))
            .one(&self.db)
            .await
        {
            Ok(Some(record)) => Ok(Some(DbRecord::model_from_record(record))),
            Ok(None) | Err(_) => Ok(None),
        }
    }

    async fn delete(&self, id: &Id) -> crate::error::Result<()> {
        DbRecord::delete_by_id(DbRecord::id_to_primary_key(id))
            .exec(&self.db)
            .await?;

        Ok(())
    }
}
