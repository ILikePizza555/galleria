use std::sync::Arc;

use anyhow::Result;
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, ActiveValue, ActiveModelTrait, DbErr};
use serenity::{async_trait, client::{EventHandler, Context}, model::{channel::{Message, Channel}, gateway::Ready, id::ChannelId}};
use tracing::{info, debug, warn, error, span, Level};
use sql_entities::{galleries, gallery_posts};

pub struct Handler {
    pub db_connection: Arc<DatabaseConnection>
}

#[async_trait]
impl EventHandler for Handler{
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "~ping" {
            send_message(&ctx, &msg.channel_id, "Pong!").await;
        } else if msg.content == "~gallery" {
            if let Err(why) = self.handle_gallery_command(&ctx, &msg).await {
                error!("Error executing gallery command: {:?}", why);
                send_message(&ctx, &msg.channel_id, "An error occured while running the command.").await;
            }
        } else {
            if let Err(why) = self.handle_new_message(&ctx, &msg).await {
                error!("Error handling new message: {:?}", why);
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

impl Handler {
    async fn handle_gallery_command(&self, ctx: &Context, msg: &Message) -> Result<()> {
        let span = span!(Level::TRACE, "create_gallery");
        let _enter = span.enter();

        debug!("Starting gallery creation.");
        
        // Check if the channel already exists
        if self.find_gallery_from_channel_id(msg.channel_id).await?.is_some() {
            warn!("Gallery for {} already exists.", msg.channel_id.0);
            send_message(&ctx, &msg.channel_id, "A gallery for this channel already exists.").await;
            return Ok(())
        }

        let new_gallery = self.create_gallery(msg.channel(&ctx.http).await?).await?;
        info!("Successfully created a new gallery: {}.", new_gallery.pk);

        // TODO: Grab all previous messages 
        
        Ok(())
    }

    async fn handle_new_message(&self, _ctx: &Context, msg: &Message) -> Result<()> {
        let span = span!(Level::TRACE, "handle_new_message");
        let _enter = span.enter();

        let images: Vec<String> = filter_images(&msg).collect();
        if images.len() == 0 {
            debug!("Message {} has no image attachements or embeds", msg.id.0);
            return Ok(())
        }

        let gallery = self.find_gallery_from_channel_id(msg.channel_id).await?;
        match gallery {
            Some(gallery_model) => {
                info!("Creating new gallery entries for message_id {}", msg.id.0);

                let new_gallery_posts = images.iter()
                    .map(|url| gallery_posts::ActiveModel {
                        gallery: ActiveValue::Set(gallery_model.pk),
                        discord_message_id: ActiveValue::Set(msg.id.0 as i64),
                        link: ActiveValue::Set(url.to_string()),
                        ..Default::default()
                    });
                gallery_posts::Entity::insert_many(new_gallery_posts).exec(self.db_connection.as_ref()).await?;

                info!("Successfully created {} gallery entries.", images.len());
                Ok(())
            },
            None => {
                debug!("No gallery found with associated channel_id {}", msg.channel_id.0);
                Ok(())
            }
        }
    }

    async fn find_gallery_from_channel_id(&self, channel_id: ChannelId) -> Result<Option<galleries::Model>, DbErr> {
        galleries::Entity::find()
            .filter(galleries::Column::DiscordChannelId.eq(channel_id.0 as i64))
            .one(self.db_connection.as_ref())
            .await
    }

    async fn create_gallery(&self, channel: Channel) -> Result<galleries::Model, DbErr> {
        let gallery_active_model = galleries::ActiveModel {
            name: ActiveValue::Set(channel.to_string()),
            discord_channel_id: ActiveValue::Set(channel.id().0 as i64),
            ..Default::default()
        };

        gallery_active_model.insert(self.db_connection.as_ref()).await
    }
}

/// Protected way to send a message to the channel. Logs any errors.
async fn send_message(ctx: &Context, channel_id: &ChannelId, message: impl std::fmt::Display) {
    if let Err(why) = channel_id.say(&ctx.http, message).await {
        error!("Error sending message: {:?}", why);
    }
}

/// Filters images from the embeds or attachements from a message
fn filter_images(msg: &Message) -> impl Iterator<Item = String> + '_ {
    let attachments_iter = msg.attachments.iter()
        .filter(|&a| a.content_type.as_ref().unwrap_or(&String::new()).starts_with("image"))
        .map(|a| a.url.clone());
    
    let embeds_iter = msg.embeds.iter()
        .filter_map(|e| e.image.as_ref().map(
            |i| i.url.clone()
        ));
    
    attachments_iter.chain(embeds_iter)
}