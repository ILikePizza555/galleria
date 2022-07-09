use anyhow::Result;
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, ActiveValue, ActiveModelTrait};
use serenity::{async_trait, client::{EventHandler, Context}, model::{channel::Message, gateway::Ready, id::ChannelId}};
use tracing::{info, warn, error};
use sql_entities::galleries as gallery;

pub struct Handler {
    pub db_connection: DatabaseConnection
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "~ping" {
            self.send_message(&ctx, &msg.channel_id, "Pong!").await;
        }

        if msg.content == "~gallery" {
            if let Err(why) = self.create_gallery(&ctx, &msg).await {
                error!("Error executing gallery command: {:?}", why);
                self.send_message(&ctx, &msg.channel_id, "An error occured while running the command.").await;
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

impl Handler {
    async fn send_message(&self, ctx: &Context, channel_id: &ChannelId, message: impl std::fmt::Display) {
        if let Err(why) = channel_id.say(&ctx.http, message).await {
            error!("Error sending message: {:?}", why);
        }
    }

    async fn create_gallery(&self, ctx: &Context, msg: &Message) -> Result<()> {
        info!("Starting gallery creation.");
        
        // Check if the channel already exists
        let channel_id = *msg.channel_id.as_u64();
        let gallery_check = gallery::Entity::find()
            .filter(gallery::Column::ChannelId.eq(channel_id as i64))
            .one(&self.db_connection)
            .await?;

        if gallery_check.is_some() {
            warn!("Gallery for {} already exists.", channel_id);
            self.send_message(&ctx, &msg.channel_id, "A gallery for this channel already exists.").await;
            return Ok(())
        }

        info!("Creating new gallery.");

        let channel = msg.channel(&ctx.http).await?;
        let new_gallery_model = gallery::ActiveModel {
            name: ActiveValue::Set(channel.to_string()),
            channel_id: ActiveValue::Set(channel_id as i64),
            ..Default::default()
        };
        let new_gallery = new_gallery_model.insert(&self.db_connection).await?;

        // TODO: Grab all previous messages 
        
        Ok(())
    }
}