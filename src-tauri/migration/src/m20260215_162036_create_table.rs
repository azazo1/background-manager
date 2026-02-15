use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 1. 创建任务主表
        manager
            .create_table(
                Table::create()
                    .table(Tasks::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Tasks::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Tasks::Name).string().not_null())
                    .col(ColumnDef::new(Tasks::Program).string().not_null())
                    .col(
                        ColumnDef::new(Tasks::Args)
                            .string()
                            .not_null()
                            .default("[]"),
                    )
                    .col(ColumnDef::new(Tasks::Stdin).string())
                    .col(ColumnDef::new(Tasks::Stdout).string())
                    .col(ColumnDef::new(Tasks::Stderr).string())
                    .col(
                        ColumnDef::new(Tasks::Enabled)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(ColumnDef::new(Tasks::TriggerTag).string().not_null())
                    .col(ColumnDef::new(Tasks::TriggerContent).string())
                    .to_owned(),
            )
            .await?;

        // 2. 创建索引：idx_tasks_enabled
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_tasks_enabled")
                    .table(Tasks::Table)
                    .col(Tasks::Enabled)
                    .to_owned(),
            )
            .await?;

        // 3. 创建索引：idx_tasks_trigger_tag
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_tasks_trigger_tag")
                    .table(Tasks::Table)
                    .col(Tasks::TriggerTag)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 撤销迁移时删除表（索引会随表自动删除）
        manager
            .drop_table(Table::drop().table(Tasks::Table).to_owned())
            .await
    }
}

/// 定义表名和列名的标识符
#[derive(DeriveIden)]
enum Tasks {
    Table,
    Id,
    Name,
    Program,
    Args,
    Stdin,
    Stdout,
    Stderr,
    Enabled,
    TriggerTag,
    TriggerContent,
}
