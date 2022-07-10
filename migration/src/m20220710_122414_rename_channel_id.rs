use sea_orm_migration::prelude::*;
use sql_entities::galleries;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220710_122414_unique_channel_id"
    }
}

impl Migration {
    fn old_channel_name() -> Alias {
        Alias::new("channel_id")
    }

    fn new_channel_name() -> Alias {
        Alias::new("discord_channel_id")
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.alter_table(
            Table::alter()
                .table(galleries::Entity)
                .rename_column(Self::old_channel_name(), Self::new_channel_name())
                .to_owned()
        ).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.alter_table(
            Table::alter()
            .table(galleries::Entity)
            .rename_column(Self::new_channel_name(), Self::old_channel_name())
            .to_owned()
        ).await
    }
}
