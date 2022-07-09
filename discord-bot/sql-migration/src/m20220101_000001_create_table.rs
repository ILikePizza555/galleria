use sea_orm_migration::prelude::*;

#[derive(Iden)]
pub enum Galleries {
    Table,
    Pk,
    Name,
    ChannelId
}

#[derive(Iden)]
pub enum GalleryPosts {
    Table,
    Pk,
    Gallery,
    DiscordMessageId,
    Link
}

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220101_000001_create_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Galleries::Table)
                    .col(
                        ColumnDef::new(Galleries::Pk)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key()
                    )
                    .col(ColumnDef::new(Galleries::Name).text().not_null())
                    .col(ColumnDef::new(Galleries::ChannelId).big_unsigned().not_null())
                    .to_owned()
            )
            .await?;
        
        manager.create_table(
            Table::create()
                .table(GalleryPosts::Table)
                .col(
                    ColumnDef::new(GalleryPosts::Pk)
                        .integer()
                        .not_null()
                        .auto_increment()
                        .primary_key()
                )
                .col(ColumnDef::new(GalleryPosts::Gallery).integer().not_null())
                .col(ColumnDef::new(GalleryPosts::DiscordMessageId).big_unsigned().not_null())
                .col(ColumnDef::new(GalleryPosts::Link).text().not_null())
                .foreign_key(
                    ForeignKeyCreateStatement::new()
                        .name("fk_gallery_posts")
                        .from_tbl(GalleryPosts::Table)
                        .from_col(GalleryPosts::Gallery)
                        .to_tbl(Galleries::Table)
                        .to_col(Galleries::Pk)
                )
                .to_owned()
        ).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(GalleryPosts::Table).to_owned()).await?;

        manager.drop_table(Table::drop().table(Galleries::Table).to_owned()).await
    }
}
