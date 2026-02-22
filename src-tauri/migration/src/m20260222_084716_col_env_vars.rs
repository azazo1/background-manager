use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 使用 alter_table 向现有 tasks 表添加列
        manager
            .alter_table(
                Table::alter()
                    .table(Tasks::Table)
                    .add_column(
                        // 添加 TEXT 类型，存储环境变量的 JSON 字符串，非空，默认值为 '{}'
                        ColumnDef::new(Tasks::EnvVars)
                            .text()
                            .not_null()
                            .default("{}"),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 回滚操作：删除 env_vars 列
        manager
            .alter_table(
                Table::alter()
                    .table(Tasks::Table)
                    .drop_column(Tasks::EnvVars)
                    .to_owned(),
            )
            .await
    }
}

/// 定义表名和新列名
#[derive(DeriveIden)]
enum Tasks {
    Table,
    EnvVars,
}

