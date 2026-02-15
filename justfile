default: dev

sea-orm-generate:
    sea-orm-cli generate entity --database-url sqlite://./data.db -o src-tauri/entity/src/ --verbose --lib --with-serde both

sea-orm-migrate:
    sea-orm-cli migrate init -d src-tauri/migration

create-db:
    rm data.db
    sqlite3 data.db < init.sql

dev:
    bun run tauri dev