use anyhow::Result;
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, ActiveValue, ActiveModelTrait};
use serenity::{async_trait, client::{EventHandler, Context}, model::{channel::Message, gateway::Ready, id::ChannelId}};
use tracing::{info, debug, warn, error, span, Level};
use sql_entities::galleries as gallery;
use sql_entities::gallery_posts as gallery_post;

pub struct Handler<'db> {
    pub db_connection: &'db DatabaseConnection
}

#[async_trait]
impl <'db> EventHandler for Handler<'db> {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "~ping" {
            send_message(&ctx, &msg.channel_id, "Pong!").await;
        } else if msg.content == "~gallery" {
            if let Err(why) = self.create_gallery(&ctx, &msg).await {
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
    async fn get_gallery_by_channel_id(&self, channel_id: u64) -> Result<Option<gallery::Model>> {
        Ok(gallery::Entity::find()
            .filter(gallery::Column::ChannelId.eq(channel_id as i64))
            .one(&self.db_connection)
            .await?)
    }

    async fn create_gallery(&self, ctx: &Context, msg: &Message) -> Result<()> {
        let span = span!(Level::TRACE, "create_gallery");
        let _enter = span.enter();

        debug!("Starting gallery creation.");
        
        // Check if the channel already exists
        let channel_id = msg.channel_id.0;
        let gallery_check = self.get_gallery_by_channel_id(channel_id).await?;

        if gallery_check.is_some() {
            warn!("Gallery for {} already exists.", channel_id);
            send_message(&ctx, &msg.channel_id, "A gallery for this channel already exists.").await;
            return Ok(())
        }

        info!("Creating new gallery.");

        let channel = msg.channel(&ctx.http).await?;
        let new_gallery_model = gallery::ActiveModel {
            name: ActiveValue::Set(channel.to_string()),
            channel_id: ActiveValue::Set(channel_id as i64),
            ..Default::default()
        };
        let _new_gallery = new_gallery_model.insert(&self.db_connection).await?;

        info!("Successfully created a new gallery.");

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

        let gallery = self.get_gallery_by_channel_id(msg.channel_id.0).await?;
        match gallery {
            Some(gallery_model) => {
                info!("Creating new gallery entries for message_id {}", msg.id.0);

                let new_gallery_posts = images.iter()
                    .map(|url| gallery_post::ActiveModel {
                        gallery: ActiveValue::Set(gallery_model.pk),
                        discord_message_id: ActiveValue::Set(msg.id.0 as i64),
                        link: ActiveValue::Set(url.to_string()),
                        ..Default::default()
                    });
                gallery_post::Entity::insert_many(new_gallery_posts).exec(&self.db_connection).await?;

                info!("Successfully created {} gallery entries.", images.len());
                Ok(())
            },
            None => {
                debug!("No gallery found with associated channel_id {}", msg.channel_id.0);
                Ok(())
            }
        }
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