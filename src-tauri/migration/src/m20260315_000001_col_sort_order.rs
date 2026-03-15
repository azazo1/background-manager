use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Tasks::Table)
                    .add_column(
                        ColumnDef::new(Tasks::SortOrder)
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared("UPDATE tasks SET sort_order = id WHERE sort_order = 0")
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_tasks_sort_order")
                    .table(Tasks::Table)
                    .col(Tasks::SortOrder)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .if_exists()
                    .name("idx_tasks_sort_order")
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Tasks::Table)
                    .drop_column(Tasks::SortOrder)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Tasks {
    Table,
    SortOrder,
}
