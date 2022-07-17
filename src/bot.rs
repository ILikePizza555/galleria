use std::sync::Arc;

use anyhow::Result;
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, ActiveValue, ActiveModelTrait, DbErr, InsertResult, TransactionTrait};
use serenity::{async_trait, client::{EventHandler, Context}, model::{channel::{Message, Channel, Attachment, Embed, EmbedThumbnail, EmbedImage}, gateway::Ready, id::ChannelId, event::MessageUpdateEvent}};
use tracing::{info, debug, warn, error, span, Level};
use sql_entities::{gallery, gallery_post};

pub struct Handler {
    pub db_connection: Arc<DatabaseConnection>
}

#[async_trait]
impl EventHandler for Handler{
    async fn message(&self, ctx: Context, msg: Message) {
        // Copy the channel_id for later usage, since msg is moved to the handler methods
        let channel_id = msg.channel_id;
        
        if msg.content == "~ping" {
            send_message(&ctx, &channel_id, "Pong!").await;
        } else if msg.content == "~gallery" {
            if let Err(why) = self.handle_gallery_command(&ctx, msg).await {
                error!("Error executing gallery command: {:?}", why);
                send_message(&ctx, &channel_id, "An error occured while running the command.").await;
            }
        } else {
            if let Err(why) = self.handle_new_message(&ctx, msg).await {
                error!("Error handling new message: {:?}", why);
            }
        }
    }

    async fn message_update(&self, ctx: Context, event: MessageUpdateEvent) {
        if let Err(why) = self.handle_message_update(&ctx, event).await {
            error!("Error handling message update: {:?}", why);
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

impl Handler {
    async fn handle_gallery_command(&self, ctx: &Context, msg: Message) -> Result<()> {
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

    async fn handle_new_message(&self, _ctx: &Context, msg: Message) -> Result<()> {
        let span = span!(Level::TRACE, "handle_new_message");
        let _enter = span.enter();
        debug!("handle_new message() - Message: {:?}", msg);

        // Optimization: Return if no attachements or embeds before querying the database
        if msg.attachments.len() == 0 && msg.embeds.len() == 0 {
            debug!("Message {} has no embeds or attachments.", msg.id.0);
            return Ok(())
        }

        let gallery_model = match self.find_gallery_from_channel_id(msg.channel_id).await? {
            Some(gallery_model) => gallery_model,
            None => {
                debug!("No gallery found with associated channel_id {}", msg.channel_id.0);
                return Ok(())
            }
        };

        // Grab all attachments and embeds into posts
        let new_posts = attachments_to_db(msg.attachments.into_iter(), &gallery_model, msg.id.0)
            .chain(embeds_to_db(msg.embeds.into_iter(), &gallery_model, msg.id.0))
            .collect::<Vec<gallery_post::ActiveModel>>();

        gallery_post::Entity::insert_many(new_posts).exec(self.db_connection.as_ref()).await?;

        Ok(())
    }

    async fn handle_message_update(&self, _ctx: &Context, event: MessageUpdateEvent) -> Result<()> {
        let span = span!(Level::TRACE, "handle_message_update");
        let _enter = span.enter();
        debug!("handle_message_update() - MessageUpdateEvent: {:?}", event);

        let gallery_model = match self.find_gallery_from_channel_id(event.channel_id).await? {
            Some(gallery_model) => gallery_model,
            None => {
                debug!("No gallery found with associated channel_id {}", event.channel_id.0);
                return Ok(());
            }
        };

        // Handle the update by removing all rows associated with the message and re-adding them.
        // Probably not very efficient, but I don't expect more than a few embeds per message.
        let attachments = event.attachments.unwrap_or_default();
        let embeds = event.embeds.unwrap_or_default();
        let new_posts = attachments_to_db(attachments.into_iter(), &gallery_model, event.id.0)
            .chain(embeds_to_db(embeds.into_iter(), &gallery_model, event.id.0))
            .collect::<Vec<gallery_post::ActiveModel>>();

        self.db_connection.transaction::<_, (), DbErr>(|txn| {
            Box::pin(async move {
                let del_result = gallery_post::Entity::delete_many()
                    .filter(gallery_post::Column::DiscordMessageId.eq(event.id.0 as i64))
                    .exec(txn)
                    .await?;
                
                debug!("Removed {} rows.", del_result.rows_affected);
                
                if new_posts.len() > 0 {
                    gallery_post::Entity::insert_many(new_posts).exec(txn).await?;
                } else {
                    debug!("No new posts to insert.");
                }
                
                Ok(())
            })
        }).await?;

        Ok(())
    }

    async fn find_gallery_from_channel_id(&self, channel_id: ChannelId) -> Result<Option<gallery::Model>, DbErr> {
        gallery::Entity::find()
            .filter(gallery::Column::DiscordChannelId.eq(channel_id.0 as i64))
            .one(self.db_connection.as_ref())
            .await
    }

    async fn create_gallery(&self, channel: Channel) -> Result<gallery::Model, DbErr> {
        let gallery_active_model = gallery::ActiveModel {
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

// Converts an iterator of Attachment objects to an iterator of gallery_post::ActiveModel objects. 
fn attachments_to_db<'r>(
    attachments: impl Iterator<Item = Attachment> + 'r,
    gallery: &'r gallery::Model,
    discord_message_id: u64
) -> impl Iterator<Item = gallery_post::ActiveModel> + 'r {
    attachments.filter(attachment_is_image).map(move |a| gallery_post::ActiveModel {
        gallery: ActiveValue::Set(gallery.pk),
        discord_message_id: ActiveValue::Set(discord_message_id as i64),
        media_url: ActiveValue::Set(Some(a.url)),
        media_width: ActiveValue::Set(a.width.and_then(|i| i32::try_from(i).ok())),
        media_height: ActiveValue::Set(a.height.and_then(|i| i32::try_from(i).ok())),
        ..Default::default()
    })
}

fn embeds_to_db<'r>(
    embeds: impl Iterator<Item = Embed> + 'r,
    gallery: &'r gallery::Model,
    discord_message_id: u64
) -> impl Iterator<Item = gallery_post::ActiveModel> + 'r {
    embeds.filter_map(move |e|
        if e.image.is_none() && e.thumbnail.is_none() {
            None
        } else {
            let (image_url, image_width, image_height) = transpose_embed_image(e.image);
            let (thumbnail_url, thumbnail_width, thumbnail_height) = tranpose_embed_thumbnail(e.thumbnail);
            
            Some(gallery_post::ActiveModel {
                gallery: ActiveValue::Set(gallery.pk),
                discord_message_id: ActiveValue::Set(discord_message_id as i64),
                source_url: ActiveValue::Set(e.url),
                media_url: ActiveValue::Set(image_url),
                media_width: ActiveValue::Set(image_width),
                media_height: ActiveValue::Set(image_height),
                thumbnail_url: ActiveValue::Set(thumbnail_url),
                thumbnail_width: ActiveValue::Set(thumbnail_width),
                thumbnail_height: ActiveValue::Set(thumbnail_height),
                ..Default::default()
            })
        }
    )
}

fn attachment_is_image(a: &Attachment) -> bool {
    a.content_type.as_ref().map(|s| s.starts_with("image")).unwrap_or(false)
}

fn tranpose_embed_thumbnail(thumbnail: Option<EmbedThumbnail>) -> (Option<String>, Option<i32>, Option<i32>) {
    thumbnail.map(|t| (
        Some(t.url),
        t.width.and_then(|w| i32::try_from(w).ok()),
        t.height.and_then(|h| i32::try_from(h).ok())
    ))
    .unwrap_or_default()
}

fn transpose_embed_image(image: Option<EmbedImage>) -> (Option<String>, Option<i32>, Option<i32>) {
    image.map(|i| (
        Some(i.url),
        i.width.and_then(|w| i32::try_from(w).ok()),
        i.height.and_then(|h| i32::try_from(h).ok())
    ))
    .unwrap_or_default()
}