use sea_orm::{EntityOrSelect, FromQueryResult, QueryFilter, QuerySelect};
use sea_orm::{Condition, DatabaseConnection, DeriveColumn, EntityTrait, EnumIter, Select};
use sea_orm::sea_query::Expr;

#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
enum Counter {
    Count,
}

pub async fn fetch_filtered_rows<T, R>(
    db: &DatabaseConnection,
    condition: Condition, 
    select: Select<T>,
) -> crate::error::Result<(u64, Vec<R>)>
where
    T: EntityTrait<Model = R>,
    R: Send + Sync + FromQueryResult,
{
    let resulting_rows = select.filter(condition);
    // Workaround: .count() is ambiguous, it wants to use an iterable count
    let count = resulting_rows.clone()
        .select()
        .column_as(Expr::col("*").count(), "count")
        .into_values::<_, Counter>()
        .one(db)
        .await?;
    let final_count: u64 = count.unwrap_or_default();
    let data = resulting_rows
        .all(db)
        .await?;
    
    Ok((final_count, data))
}
