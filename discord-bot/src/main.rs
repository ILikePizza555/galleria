use std::env;

use serenity::framework::StandardFramework;
use serenity::framework::standard::CommandResult;
use serenity::framework::standard::macros::{group, command};
use serenity::http::CacheHttp;
use serenity::model::channel::Message;
use serenity::{ Client, async_trait, model::gateway::Ready};
use serenity::prelude::GatewayIntents;
use serenity::client::{EventHandler, Context};

#[group]
#[commands(ping)]
struct General;

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http, "Pong!").await?;
    Ok(())
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("~"))
        .group(&GENERAL_GROUP);

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error created client");
    
    // Start the client
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}