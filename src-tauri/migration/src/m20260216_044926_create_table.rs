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
                    .col(ColumnDef::new(Tasks::LastExitCode).integer())
                    .col(ColumnDef::new(Tasks::LastRunAt).string()) // SQLite 中 TEXT 对应 .string()
                    .to_owned(),
            )
            .await?;

        // 2. 创建索引: idx_tasks_enabled
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

        // 3. 创建索引: idx_tasks_trigger_tag
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
        // 撤销迁移：直接删除表（索引会随之自动删除）
        manager
            .drop_table(Table::drop().table(Tasks::Table).to_owned())
            .await
    }
}

/// 使用枚举定义标识符，避免硬编码字符串
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
    LastExitCode,
    LastRunAt,
}