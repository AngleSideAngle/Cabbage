use std::collections::HashSet;
use std::env;
use std::sync::Arc;

use serenity::async_trait;
use serenity::client::bridge::gateway::ShardManager;
use serenity::framework::standard::macros::group;
use serenity::framework::StandardFramework;
use serenity::http::Http;
use serenity::model::event::ResumedEvent;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use tracing::{error, info};

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        info!("Connected as {}", ready.user.name);
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
        info!("resumed");
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("Failed to load .env file");

    tracing_subscriber::fmt::init();

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let http = Http::new(&token);

    let (owners, bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            if let Some(team) = info.team {
                owners.insert(team.owner_user_id);
            } else {
                owners.insert(info.owner.id);
            }
            match http.get_current_user().await {
                Ok(bot_id) => (owners, bot_id.id),
                Err(why) => panic!("Could not access the bot id: {:?}", why),
            }
        },
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    let fw = StandardFramework::new()
        .configure(
            |c| c
            .with_whitespace(true)
            .on_mention(Some(bot_id))
            .prefix("+")
            .owners(owners));
        // .before(f)
        // .after(f)
        // .normal_message(f)
        // .help(h)
        // .group(group);

    let intents = GatewayIntents::all();
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .framework(fw)
        .await
        .unwrap();

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
