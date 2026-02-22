pub use sea_orm_migration::prelude::*;

mod m20260216_044926_create_table;
mod m20260218_042706_col_no_console;
mod m20260218_141207_col_working_dir;
mod m20260222_084716_col_env_vars;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260216_044926_create_table::Migration),
            Box::new(m20260218_042706_col_no_console::Migration),
            Box::new(m20260218_141207_col_working_dir::Migration),
            Box::new(m20260222_084716_col_env_vars::Migration),
        ]
    }
}
