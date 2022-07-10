mod entities;
pub use entities::*;

use sea_orm::{EntityTrait, QueryFilter, ColumnTrait, ConnectionTrait};

// Define extensions on entities here to prevent being overridden
impl galleries::Entity {
    pub async fn find_by_channel_id<C: ConnectionTrait>(db_connection: &C, channel_id: u64) -> Result<Option<galleries::Model>, sea_orm::DbErr> {
        Self::find()
            .filter(galleries::Column::DiscordChannelId.eq(channel_id as i64))
            .one(db_connection)
            .await
    }
}