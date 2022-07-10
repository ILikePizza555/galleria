use std::sync::Arc;

use anyhow::{Result, Context as ErrorContext};
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, ActiveValue, ActiveModelTrait, DbErr, InsertResult};
use serenity::{async_trait, client::{EventHandler, Context}, model::{channel::{Message, Channel, Attachment, Embed}, gateway::Ready, id::ChannelId, event::MessageUpdateEvent}};
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

    async fn message_update(&self, ctx: Context, event: MessageUpdateEvent) {
        if let Err(why) = self.handle_message_update(&ctx, &event).await {
            error!("Error handling message update: {:?}", why);
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

        let image_urls: Vec<String> = filter_image_urls_from_message(&msg).collect();
        if image_urls.len() == 0 {
            debug!("Message {} has no image attachements or embeds", msg.id.0);
            return Ok(())
        }

        let gallery = self.find_gallery_from_channel_id(msg.channel_id).await?;
        match gallery {
            Some(gallery_model) => {
                self.create_gallery_posts(&gallery_model, msg.id.0, image_urls).await?;
                Ok(())
            },
            None => {
                debug!("No gallery found with associated channel_id {}", msg.channel_id.0);
                Ok(())
            }
        }
    }

    async fn handle_message_update(&self, _ctx: &Context, event: &MessageUpdateEvent) -> Result<()> {
        let span = span!(Level::TRACE, "handle_message_update");
        let _enter = span.enter();

        let image_urls: Vec<String> = filter_image_urls(
            event.attachments.as_ref().unwrap_or(&Vec::new()).iter(),
            event.embeds.as_ref().unwrap_or(&Vec::new()).iter()
        ).collect();

        if image_urls.len() == 0 {
            return Ok(())
        }
        
        let gallery = self.find_gallery_from_channel_id(event.channel_id)
            .await
            .context("Failed to query database for galleries with channel id.")?;

        match gallery {
            Some(gallery_model) => {
                self.create_gallery_posts(&gallery_model, event.id.0, image_urls).await?;
                Ok(())
            }
            None => {
                debug!("No gallery found with associated channel_id {}", event.channel_id.0);
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

    async fn create_gallery_posts(&self, gallery: &galleries::Model, message_id: u64, image_urls: Vec<String>) -> Result<InsertResult<gallery_posts::ActiveModel>> {
        let new_gallery_posts = image_urls.iter()
            .map(|url| gallery_posts::ActiveModel {
                gallery: ActiveValue::Set(gallery.pk),
                discord_message_id: ActiveValue::Set(message_id as i64),
                link: ActiveValue::Set(url.to_string()),
                ..Default::default()
            });
        
        gallery_posts::Entity::insert_many(new_gallery_posts)
            .exec(self.db_connection.as_ref())
            .await
            .context("Gallery_post insert_many query failed.")
    }
}

/// Protected way to send a message to the channel. Logs any errors.
async fn send_message(ctx: &Context, channel_id: &ChannelId, message: impl std::fmt::Display) {
    if let Err(why) = channel_id.say(&ctx.http, message).await {
        error!("Error sending message: {:?}", why);
    }
}

fn filter_image_urls_from_message(message: &Message) -> impl Iterator<Item = String> + '_ {
    filter_image_urls(message.attachments.iter(), message.embeds.iter())
}

/// Filters images from the embeds or attachements from a message
fn filter_image_urls<'l, A, E>(a: A, e: E) -> impl Iterator<Item = String> + 'l
where
    A: Iterator<Item = &'l Attachment> + 'l,
    E: Iterator<Item = &'l Embed> + 'l
{
    let attachments_iter = a
        .filter_map(
            |a| a.content_type.as_ref().and_then(
                |content_type| if content_type.starts_with("image") {
                    Some(a.url.clone())
                } else {
                    None
                }
            )
        );
    
    let embeds_iter = e
        .filter_map(|e| e.image.as_ref().map(
            |i| i.url.clone()
        ));
    
    attachments_iter.chain(embeds_iter)
}