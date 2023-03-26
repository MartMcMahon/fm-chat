use gptapi::MessageObject;
use serenity::framework::standard::{CommandResult, StandardFramework};
use serenity::http::Typing;
use serenity::model::channel::Message;
use serenity::prelude::*;
use serenity::{
    async_trait,
    utils::{content_safe, ContentSafeOptions},
};
use serenity::{
    framework::standard::{
        macros::{command, group, hook},
        Args,
    },
    model::prelude::ChannelId,
};
use std::{env, sync::Arc};

mod gptapi;

#[command]
async fn say(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    match args.single_quoted::<String>() {
        Ok(x) => {
            let settings = if let Some(guild_id) = msg.guild_id {
                // By default roles, users, and channel mentions are cleaned.

                ContentSafeOptions::default()
                    // We do not want to clean channel mentions as they
                    // do not ping users.
                    .clean_channel(false)
                    // If it's a guild channel, we want mentioned users to be displayed
                    // as their display name.
                    .display_as_member_from(guild_id)
            } else {
                ContentSafeOptions::default()
                    .clean_channel(false)
                    .clean_role(false)
            };

            let content = content_safe(&ctx.cache, x, &settings, &msg.mentions);

            msg.channel_id.say(&ctx.http, &content).await?;

            return Ok(());
        }
        Err(_) => {
            msg.reply(ctx, "An argument is required to run this command.")
                .await?;
            return Ok(());
        }
    };
}
#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Pong!").await?;

    Ok(())
}

#[command]
async fn list(ctx: &Context, msg: &Message) -> CommandResult {
    let typing = msg.channel_id.start_typing(&ctx.http)?;
    let message_list: Vec<MessageObject> = {
        let data_read = ctx.data.read().await;
        let message_list_lock = data_read
            .get::<MessageData>()
            .expect("expected MessageData")
            .clone();
        let data = message_list_lock.read().await;
        let list = data.clone().into_iter();
        list.map(|m| m.clone()).collect()
    };

    let mut res = String::new();
    for m in message_list {
        res.push_str(&m.role);
        res.push_str(": ");
        if &m.content.len() > &18 {
            res.push_str(&m.content[..18]);
        } else {
            res.push_str(&m.content);
        }
        res.push_str("...\n");
    }
    msg.channel_id.say(&ctx.http, &res).await?;
    let _stopped = typing.stop();
    Ok(())
}

#[command]
async fn chat(ctx: &Context, msg: &Message) -> CommandResult {
    let typing = msg.channel_id.start_typing(&ctx.http)?;
    let mut message_list: Vec<MessageObject> = {
        let data_read = ctx.data.read().await;
        let message_list_lock = data_read
            .get::<MessageData>()
            .expect("expected MessageData")
            .clone();
        let data = message_list_lock.read().await;
        let list = data.clone().into_iter();
        list.map(|m| m.clone()).collect()
    };

    let new_user_msg = MessageObject {
        role: "user".to_owned(),
        content: msg.content.clone(),
    };
    message_list.push(new_user_msg.clone());

    let gpt_bot = gptapi::GptBot::new();
    let res = gpt_bot.gpt_req(message_list).await?;

    {
        let data_read = ctx.data.read().await;
        let message_list_lock = data_read
            .get::<MessageData>()
            .expect("expected MessageData")
            .clone();

        let mut data = message_list_lock.write().await;
        data.push(new_user_msg);
        data.push(MessageObject {
            role: "assistant".to_owned(),
            content: res.clone(),
        });
    }

    // let message_list: Vec<MessageData> = {
    //     let data_write = ctx.data.write().await;
    //     let message_objects
    // }

    // let mut data = ctx.data.write().await;
    // let messages = data.get_mut::<MessageData>().expect("Expected messages");
    // messages.push(gptapi::MessageObject {
    //     role: "assistant".to_owned(),
    //     content: res.clone(),
    // });

    msg.channel_id.say(&ctx.http, &res).await?;
    let _stopped = typing.stop();
    Ok(())
}

#[group]
#[commands(ping, say, chat, list)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {}

// #[hook]
// async fn before(ctx: &Context, msg: &Message, command_name: &str) -> bool {
//     let mut data = ctx.data.write().await;
//     let counter = data
//         .get_mut::<MessageData>()
//         .expect("Expected CommandCounter in TypeMap.");
//     let entry = counter.entry(command_name.to_string()).or_insert(0);
//     *entry += 1;

//     true // if `before` returns false, command processing doesn't happen.
// }

#[tokio::main]
async fn main() {
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("~")) // set the bot's prefix to "~"
        // .before(before)
        .group(&GENERAL_GROUP);

    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").expect("token");
    let intents = GatewayIntents::all();
    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    {
        // Open the data lock in write mode, so keys can be inserted to it.
        let mut data = client.data.write().await;
        data.insert::<MessageData>(Arc::new(RwLock::new(vec![MessageObject{role:"system".to_owned(), content:"You are a playful chatbot creating creative samples from prompts sent through discord.".to_owned()}])));
    }

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}

struct MessageData;

impl TypeMapKey for MessageData {
    type Value = Arc<RwLock<Vec<MessageObject>>>;
}
