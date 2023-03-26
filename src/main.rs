use gptapi::MessageObject;
use serenity::framework::standard::{
    macros::{command, group, hook},
    Args,
};
use serenity::framework::standard::{CommandResult, StandardFramework};
use serenity::model::channel::Message;
use serenity::prelude::*;
use serenity::{
    async_trait,
    utils::{content_safe, ContentSafeOptions},
};
use std::env;

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

struct MessageData;
impl TypeMapKey for MessageData {
    type Value = Vec<gptapi::MessageObject>;
}

#[command]
async fn chat(ctx: &Context, msg: &Message) -> CommandResult {
    // {
    //     let data = ctx.data.read().await;
    //     let messages = data.get::<MessageData>().expect("Expected messages");

    //     for m in messages {
    //         println!("{:?}", m.content);
    //     }
    // }

    let gpt_bot = gptapi::GptBot::new();
    let res = gpt_bot.gpt_req(msg.content.clone()).await?;
    msg.channel_id.say(&ctx.http, &res).await?;

    // let mut data = ctx.data.write().await;
    // let messages = data.get_mut::<MessageData>().expect("Expected messages");
    // messages.push(gptapi::MessageObject {
    //     role: "assistant".to_owned(),
    //     content: res.clone(),
    // });

    Ok(())
}

#[group]
#[commands(ping, say, chat)]
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

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}
