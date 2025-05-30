use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for crate::m20250530_124142_add_comments::Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared(
            "CREATE TABLE comments (
    id UUID PRIMARY KEY,
    topic_id UUID NOT NULL REFERENCES topics(id) ON DELETE CASCADE,
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
        db.execute_unprepared("DROP TABLE comments;").await?;

        Ok(())
    }
}
