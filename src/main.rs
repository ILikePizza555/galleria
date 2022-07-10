mod bot;

use crate::bot::Handler;

use std::env;
use anyhow::Result;
use sea_orm::{Database};
use serenity::Client;
use serenity::prelude::GatewayIntents;

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

    tracing_subscriber::fmt::init();

    let db_connection = Database::connect(db_url).await
        .expect("Could not estable a connection to the database.");

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler { db_connection: &db_connection })
        .await
        .expect("Error created client");
    
    // Start the client
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

