use std::env;
use std::io::ErrorKind;

use anyhow::Result;
use sea_orm::{Database, DatabaseConnection};
use serenity::model::channel::Message;
use serenity::{ Client, async_trait, model::gateway::Ready};
use serenity::prelude::GatewayIntents;
use serenity::client::{EventHandler, Context};

struct Handler {
    db_connection: DatabaseConnection
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "~ping" {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                println!("Error sending message: {:?}", why);
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

fn load() -> Result<(String, String)> {
    // Load the dotenv file, but ignore not found errors. 
    dotenv::dotenv()
        .map(|ok| Some(ok))
        .or_else(|err| match err {
            dotenv::Error::Io(io_error) =>
                if io_error.kind() == std::io::ErrorKind::NotFound {
                    Ok(None)
                } else {
                    Err(dotenv::Error::Io(io_error))
                }
            _ => Err(err)
        })?;

    let token = env::var("DISCORD_TOKEN")?;
    let db_url = env::var("DATABASE_URL")?;

    Ok((token, db_url ))
}

#[tokio::main]
async fn main() {
    let (token, db_url) = load().unwrap();

    let db_connection = Database::connect(db_url).await
        .expect("Could not estable a connection to the database.");

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler { db_connection })
        .await
        .expect("Error created client");
    
    // Start the client
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

