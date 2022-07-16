use sea_orm_migration::{prelude::*, sea_orm::{ConnectionTrait, Statement}};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220101_000001_create_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let gallery_table_sql = r#"
            CREATE TABLE "gallery" (
                "pk" UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
                "name" TEXT NOT NULL,
                "discord_channel_id" BIGINT NOT NULL,
                "date_created" TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
            );
        "#;

        let posts_table_sql = r#"
            CREATE TABLE "gallery_post" (
                "pk" UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
                "gallery" UUID NOT NULL,
                "discord_message_id" BIGINT NOT NULL,
                "source_url" TEXT,
                "media_url" TEXT,
                "media_width" INTEGER,
                "media_height" INTEGER,
                "thumbnail_url" TEXT,
                "thumbnail_width" INTEGER,
                "thumbnail_height" INTEGER,
                "date_created" TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                CONSTRAINT fk_gallery FOREIGN KEY("gallery") REFERENCES "gallery"("pk")
                    ON DELETE CASCADE
                    ON UPDATE CASCADE
            );
        "#;

        let gallery_index_sql = r#"CREATE INDEX "idx_gallery_discord_channel_id" ON gallery(discord_channel_id);"#;
        let posts_index_sql = r#"CREATE INDEX "idk_gallery_post_discord_message_id" ON gallery_post(discord_message_id);"#;

        let gallery_table_stmt = Statement::from_string(manager.get_database_backend(), gallery_table_sql.to_owned());
        let posts_table_stmt = Statement::from_string(manager.get_database_backend(), posts_table_sql.to_owned());
        let gallery_index_stmt = Statement::from_string(manager.get_database_backend(), gallery_index_sql.to_owned());
        let posts_index_stmt = Statement::from_string(manager.get_database_backend(), posts_index_sql.to_owned());

        manager.get_connection().execute(gallery_table_stmt).await?;
        manager.get_connection().execute(posts_table_stmt).await?;
        manager.get_connection().execute(gallery_index_stmt).await?;
        manager.get_connection().execute(posts_index_stmt).await.map(|_| ())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(Alias::new("gallery")).to_owned()).await?;

        manager.drop_table(Table::drop().table(Alias::new("gallery_post")).to_owned()).await
    }
}
