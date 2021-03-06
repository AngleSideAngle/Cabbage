#![allow(unused_imports)]

use std::collections::{HashMap, HashSet};
use std::{env, string};
use std::fmt::Write;
use std::sync::Arc;

use serenity::async_trait;
use serenity::builder::CreateEmbed;
use serenity::client::bridge::gateway::{ShardId, ShardManager};
use serenity::framework::standard::buckets::{LimitedFor, RevertBucket};
use serenity::framework::standard::macros::{check, command, group, help, hook};
use serenity::framework::standard::{
    help_commands,
    Args,
    CommandGroup,
    CommandOptions,
    CommandResult,
    DispatchError,
    HelpOptions,
    Reason,
    StandardFramework,
};
use serenity::http::Http;
use serenity::model::channel::{Channel, Message, Embed};
use serenity::model::event::ResumedEvent;
use serenity::model::gateway::{GatewayIntents, Ready};
use serenity::model::id::UserId;
use serenity::model::permissions::Permissions;
use serenity::prelude::*;
use serenity::utils::{content_safe, ContentSafeOptions, MessageBuilder};
use tokio::sync::Mutex;

struct MessageLogger;
struct CabbageableLogger;
struct UserRegistery;

impl TypeMapKey for MessageLogger {
    type Value = Arc<RwLock<HashMap<u64, String>>>;
}

impl TypeMapKey for CabbageableLogger {
    type Value = Arc<RwLock<HashMap<u64, bool>>>;
}

impl TypeMapKey for UserRegistery {
    type Value = Arc<RwLock<HashMap<u64, Vec<String>>>>;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("Connected as {}", ready.user.name);
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
        println!("resumed");
    }
}

#[group]
#[commands(test, last, cabbage, register)]
struct Game;

#[hook]
async fn before(ctx: &Context, msg: &Message, command_name: &str) -> bool {
    true
}

#[hook]
async fn normal_message(ctx: &Context, msg: &Message) {
    // if msg.content.to_lowercase().contains("cabbage") {
    //     if let Some(guild) = msg.guild(&ctx.cache) {
    //         if let Some(target) = guild.member_named(user) {
    //             let lock = {
    //                 let data = ctx.data.read().await;
    //                 data.get::<MessageLogger>().unwrap().clone()
    //             };
        
    //             let logger = lock.read().await;
    //             let self_entry = logger.get(msg.author.id.as_u64());
    //             let self_last = self_entry.unwrap_or(&String::new());
    //             let target_entry = logger.get(target.user.id.as_u64());
    //             let target_last = target_entry.unwrap_or(&String::new());
    //             drop(logger);

    //             if self_last == &target.user.name && target_last.eq_ignore_ascii_case("minimuffin") {}
    //             let message = MessageBuilder::new()
    //                 .mention(&msg.author)
    //                 .push(" cabbaged ")
    //                 .mention(target)
    //                 .build();

    //             msg.reply_ping(&ctx, message).await?;
    //         }
    //     }
    // }

    // update data
    let lock = {
        let data = ctx.data.read().await;
        data.get::<MessageLogger>().unwrap().clone()
    };

    {
        let mut logger = lock.write().await;
        let entry = logger.entry(msg.author.id.as_u64().clone()).or_insert("".to_string());
        *entry = msg.content.to_string();
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("Failed to load .env file");

    tracing_subscriber::fmt::init();

    let token = env::var("TOKEN").expect("Expected a token in the environment");
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
            .owners(owners))
        // .before(f)
        // .after(f)
        .normal_message(normal_message)
        // .help(h)
        .group(&GAME_GROUP);
        // .group(group);

    let intents = GatewayIntents::all();
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .framework(fw)
        .await
        .unwrap();

    // add data to bot
    {
        let mut data = client.data.write().await;
        data.insert::<MessageLogger>(Arc::new(RwLock::new(HashMap::default())));
        data.insert::<CabbageableLogger>(Arc::new(RwLock::new(HashMap::default())));
        data.insert::<UserRegistery>(Arc::new(RwLock::new(HashMap::default())));
    }

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

#[command]
async fn register(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let lock = {
        let data = ctx.data.read().await;
        data.get::<UserRegistery>().unwrap().clone()
    };
    
    
    {
        let mut user_registry = lock.write().await;
        let entry = user_registry.entry(msg.author.id.as_u64().clone()).or_insert(Vec::new());
        entry.push(args.rest().to_string());
        msg.channel_id.send_message(&ctx, |m|
            m.embed(|e| e.title("registered").description(&entry[0]))).await?;
    }

    Ok(())
}

#[command]
async fn test(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    msg.reply(&ctx, "sussy").await?;
    Ok(())
}

#[command]
async fn last(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let lock = {
        let data = ctx.data.read().await;
        data.get::<MessageLogger>().unwrap().clone()
    };
    
    {
        let logger = lock.read().await;
        let entry = logger.get(msg.author.id.as_u64());
        msg.reply(&ctx, entry.unwrap_or(&"NA".to_string())).await?;
    }

    Ok(())
}

#[command]
async fn cabbage(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    // let user = args.rest();
    // if let Some(guild) = msg.guild(&ctx.cache) {
    //     if let Some(target) = guild.member_named(user) {
    //         let lock = {
    //             let data = ctx.data.read().await;
    //             data.get::<MessageLogger>().unwrap().clone()
    //         };
    
    //         let logger = lock.read().await;
    //         let self_entry = logger.get(msg.author.id.as_u64());
    //         let self_last = self_entry.unwrap_or(&String::new());
    //         let target_entry = logger.get(target.user.id.as_u64());
    //         let target_last = target_entry.unwrap_or(&String::new());
    //         drop(logger);

    //         if self_last == &target.user.name && target_last.eq_ignore_ascii_case("minimuffin") {}
    //         let message = MessageBuilder::new()
    //             .mention(&msg.author)
    //             .push(" cabbaged ")
    //             .mention(target)
    //             .build();

    //         msg.reply_ping(&ctx, message).await?;
    //     }
    // }
    Ok(())
}
