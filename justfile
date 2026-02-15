default: dev

sea-orm-generate:
    sea-orm-cli generate entity --database-url sqlite://./data.db -o src-tauri/entity -v --with-serde both

create-db:
    sqlite3 data.db < init.sql

dev:
    bun run tauri dev