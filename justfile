default: dev

sea-orm-generate:
    sea-orm-cli generate entity --database-url sqlite://./data.db -o src-tauri/src/entity -v --with-serde both

sea-orm-migrate:
    cd src-tauri && sea-orm-cli migrate init

create-db:
    rm data.db
    sqlite3 data.db < init.sql

dev:
    bun run tauri dev