use sea_orm::DatabaseConnection;
use serenity::{async_trait, client::{EventHandler, Context}, model::{channel::Message, gateway::Ready}};
use tracing::error;

pub struct Handler {
    pub db_connection: DatabaseConnection
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "~ping" {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                error!("Error sending message: {:?}", why);
            }
        }

        
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}