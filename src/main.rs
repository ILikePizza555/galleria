mod bot;
mod web;

use crate::bot::Handler;
use crate::web::galleria_service;

use std::env;
use std::sync::Arc;
use anyhow::Result;
use futures::FutureExt;
use futures::future::try_join;
use sea_orm::{Database, DatabaseConnection};
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

    // Setup DB
    let db_connection_base = Database::connect(db_url).await
        .expect("Could not estable a connection to the database.");
    let db_connection = Arc::new(db_connection_base);

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut discord_client = Client::builder(&token, intents)
        .event_handler(Handler { db_connection: db_connection.clone() })
        .await
        .expect("Error created client");
    
    let web_server = warp::serve(galleria_service(db_connection.clone())).bind(([127, 0, 0, 1], 3030))
        .map(|_| Ok(()));

    if let Err(why) = try_join(discord_client.start(), web_server).await {
        println!("Client error: {:?}", why);
    }
}