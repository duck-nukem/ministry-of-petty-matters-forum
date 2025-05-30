use sea_orm::{FromQueryResult, Order, QueryFilter, QueryOrder, QuerySelect};
use sea_orm::{Condition, DatabaseConnection, DeriveColumn, EntityTrait, EnumIter, Select};
use sea_orm::sea_query::Expr;
use crate::persistence::repository::ListParameters;

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
    let count = resulting_rows.clone()
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
