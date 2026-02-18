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
                    .add_column(ColumnDef::new(Tasks::WorkingDir).string())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 回滚操作：在 SQLite 中，旧版本可能不支持直接 DROP COLUMN
        // 但 SeaORM 会尽力通过重建表的方式帮你完成
        manager
            .alter_table(
                Table::alter()
                    .table(Tasks::Table)
                    .drop_column(Tasks::WorkingDir)
                    .to_owned(),
            )
            .await
    }
}

/// 只需要定义涉及到的表名和新列名
#[derive(DeriveIden)]
enum Tasks {
    Table,
    WorkingDir,
}
