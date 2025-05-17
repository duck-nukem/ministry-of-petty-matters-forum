# Database

## Migrations

1. Install `cargo install sqlx-cli --no-default-features --features sqlite`
2. Run `sqlx migrate add <migration_name>` to create a new migration file.
3. Edit the migration file in `migrations` directory to add your SQL commands.
4. Run `sqlx migrate run` to apply the migration to the database.
