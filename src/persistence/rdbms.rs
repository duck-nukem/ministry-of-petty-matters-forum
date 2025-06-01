use crate::persistence::repository::{HasId, ListParameters, Page, Repository};
use crate::views::pagination::Ordering;
use async_trait::async_trait;
use sea_orm::sea_query::Expr;
use sea_orm::{Condition, DatabaseConnection, DeriveColumn, EntityTrait, EnumIter, Select};
use sea_orm::{
    FromQueryResult, IntoActiveModel, Order, PrimaryKeyTrait, QueryFilter, QueryOrder, QuerySelect,
};

pub trait ModelDatabaseInterface<E: EntityTrait, M, Id> {
    fn filter_from_params(list_parameters: &ListParameters) -> Condition;
    fn order_by_from_params(list_parameters: &ListParameters) -> (E::Column, Order);
    fn model_from_record(record: E::Model) -> M;
    fn model_to_record(model: M) -> E::ActiveModel;
    fn unwrap_id(id: &Id) -> <<E>::PrimaryKey as PrimaryKeyTrait>::ValueType;
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
enum Counter {
    Count,
}

#[allow(clippy::cast_sign_loss)]
pub async fn fetch_filtered_rows<T, R>(
    db: &DatabaseConnection,
    condition: Condition,
    list_parameters: &ListParameters,
    ordering: (T::Column, Order),
    select: Select<T>,
) -> crate::error::Result<(u64, Vec<R>)>
where
    T: EntityTrait<Model = R>,
    R: Send + Sync + FromQueryResult,
{
    let resulting_rows = select.filter(condition);
    // Workaround: .count() is ambiguous, it wants to use an iterable count
    let count = resulting_rows
        .clone()
        .select_only()
        .column_as(Expr::val(1).count(), "count")
        .into_values::<_, Counter>()
        .one(db)
        .await?;
    let final_count: i64 = count.unwrap_or_default();
    let data = resulting_rows
        .offset(Some(list_parameters.calculate_offset() as u64))
        .limit(Some(list_parameters.calculate_limit() as u64))
        .order_by(ordering.0, ordering.1)
        .all(db)
        .await?;

    Ok((final_count as u64, data))
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
        let (count, data) = fetch_filtered_rows(
            &self.db,
            DbRecord::filter_from_params(&list_parameters),
            &list_parameters,
            DbRecord::order_by_from_params(&list_parameters),
            DbRecord::find(),
        )
        .await?;

        Ok(Page {
            items: data.into_iter().map(DbRecord::model_from_record).collect(),
            size: list_parameters.page_size,
            current_page_number: list_parameters.page_number,
            total_count: count,
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
        match DbRecord::find_by_id(DbRecord::unwrap_id(id))
            .one(&self.db)
            .await
        {
            Ok(Some(record)) => Ok(Some(DbRecord::model_from_record(record))),
            Ok(None) | Err(_) => Ok(None),
        }
    }

    async fn delete(&self, id: &Id) -> crate::error::Result<()> {
        DbRecord::delete_by_id(DbRecord::unwrap_id(id))
            .exec(&self.db)
            .await?;

        Ok(())
    }
}
