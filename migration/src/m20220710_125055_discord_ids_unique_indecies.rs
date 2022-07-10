use sea_orm_migration::prelude::*;
use sql_entities::{galleries, gallery_posts};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220710_125055_discord_ids_unique_indecies"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_index(
            Index::create()
            .name("idx_unique_discord_channel_id")
            .table(galleries::Entity)
            .col(galleries::Column::DiscordChannelId)  // Will break if columns are changed
            .unique()
            .to_owned()
        ).await?;

        manager.create_index(
            Index::create()
            .name("idx_unique_discord_message_id")
            .table(gallery_posts::Entity)
            .col(gallery_posts::Column::DiscordMessageId)
            .unique()
            .to_owned()
        ).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_index(
            Index::drop()
            .name("idx_unique_discord_channel_id")
            .table(galleries::Entity)
            .to_owned()
        ).await?;

        manager.drop_index(
            Index::drop()
            .name("idx_unique_discord_message_id")
            .table(gallery_posts::Entity)
            .to_owned()
        ).await
    }
}
