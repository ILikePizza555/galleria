pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_table;
mod m20220710_122414_rename_channel_id;
mod m20220710_125055_discord_ids_unique_indecies;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_table::Migration),
            Box::new(m20220710_122414_rename_channel_id::Migration),
            Box::new(m20220710_125055_discord_ids_unique_indecies::Migration),
        ]
    }
}
