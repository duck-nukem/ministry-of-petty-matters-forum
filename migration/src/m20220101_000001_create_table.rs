use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared(
            "CREATE TABLE topics (
    id UUID PRIMARY KEY,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    upvotes_count INTEGER NOT NULL,
    downvotes_count INTEGER NOT NULL,
    created_by TEXT NOT NULL,
    creation_time TIMESTAMPTZ NOT NULL,
    last_updated_time TIMESTAMPTZ
);",
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared("DROP TABLE topics;").await?;
        
        Ok(())
    }
}
