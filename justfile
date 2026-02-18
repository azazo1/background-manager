default: dev

sea-orm-generate:
    sea-orm-cli generate entity --database-url sqlite://./data.db -o src-tauri/entity/src/ --verbose --lib --with-serde both

# sea-orm-migrate:
#     sea-orm-cli migrate init -d src-tauri/migration

sea-orm-migration name:
    cd src-tauri && sea-orm-cli migrate generate {{name}}

create-db:
    rm -f data.db
    sqlite3 data.db < init.sql

dev:
    bun run tauri dev