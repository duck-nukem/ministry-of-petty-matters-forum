# Database

## Migrations

1. Install `cargo install sea-orm-cli --features sqlx-postgres` (check `Cargo.toml` for the latest version).
2. Set the connection string via `export DATABASE_URL="postgresql://postgres:password@localhost:5432/postgres"`
3. `sea-orm-cli migrate generate --name <migration_name>` to create a new migration.
4. `sea-orm-cli migrate up` to apply all pending migrations