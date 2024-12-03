use std::collections::HashMap;
use std::env;
use std::hash::Hash;

use discord_bot::message_data::{EmojiData, JsonData, MessageData, UserData};
use dotenvy::dotenv;
use serenity::all::{ChannelId, EmojiId, GuildId, PermissionOverwriteType, Result, UserId};
use serenity::{
    all::{CacheHttp, Context, EventHandler, GatewayIntents, GetMessages, GuildChannel, Message},
    async_trait, Client,
};

#[allow(dead_code)]
async fn get_messages<F: Fn(&Message) -> bool + Copy, F1: Fn(&Message) -> bool + Copy>(
    cache: impl CacheHttp,
    channel: GuildChannel,
    filter: Option<F>,
    stop: Option<F1>,
) -> Result<Vec<Message>> {
    let mut messages: Vec<Message> = Vec::<Message>::new();
    println!("Channel: {}", channel.name);
    if !channel.is_text_based() {
        return Ok(messages);
    }
    let mut last_message_id = match channel.last_message_id {
        Some(x) => x,
        None => return Ok(messages),
    };
    if let Ok(last_message) = channel.message(&cache, last_message_id).await {
        messages.push(last_message);
    }
    loop {
        let get_messages = GetMessages::new().before(last_message_id).limit(100);
        let new_messages = channel.messages(&cache, get_messages).await?;
        if new_messages.is_empty() {
            return Ok(messages);
        }
        last_message_id = new_messages.last().unwrap().id;
        for message in new_messages {
            if filter.is_some() && !filter.unwrap()(&message) {
                continue;
            }
            if stop.is_some() && stop.unwrap()(&message) {
                return Ok(messages);
            }
            messages.push(message);
        }
    }
}

#[allow(dead_code)]
fn add_or_insert<K: Eq + Hash>(map: &mut HashMap<K, usize>, key: K) {
    map.entry(key).and_modify(|x| *x += 1).or_insert(1);
}

fn is_private_archive_channel(channel: &GuildChannel, guild_id: GuildId) -> bool {
    for permission_overwrite in channel.permission_overwrites.iter() {
        match permission_overwrite.kind {
            PermissionOverwriteType::Role(role_id) => {
                if role_id.get() != guild_id.get() {
                    continue;
                }
            }
            _ => continue,
        }
        return permission_overwrite.deny.view_channel();
    }
    false
}

#[allow(dead_code)]
struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, _ready: serenity::model::gateway::Ready) {
        // let guild_id: GuildId = 1095933862657405038.into();
        // let guild_id = GuildId::new(1095933862657405038);
        let guild_id = GuildId::new(986597459323150376);
        let guild = guild_id.to_partial_guild(&ctx.http).await.unwrap();
        println!("Guild: {}", guild.name);

        let members: HashMap<UserId, UserData> = guild
            .members(&ctx.http, None, None)
            .await
            .unwrap()
            .into_iter()
            .map(|m| (m.user.id, m.into()))
            .collect();

        let emojis: HashMap<EmojiId, EmojiData> = guild
            .emojis(&ctx.http)
            .await
            .unwrap()
            .into_iter()
            .map(|e| (e.id, e.into()))
            .collect();

        let channels = guild.channels(&ctx.http).await.unwrap();
        let mut messages = HashMap::<ChannelId, Vec<MessageData>>::new();
        for (_, channel) in channels {
            if is_private_archive_channel(&channel, guild_id) {
                continue;
            }
            let message_dates = get_messages(
                &ctx.http,
                channel.clone(),
                None::<fn(&Message) -> bool>,
                None::<fn(&Message) -> bool>,
            )
            .await
            .unwrap()
            .into_iter()
            .map(|m| m.into())
            .collect();
            messages.insert(channel.id, message_dates);
        }

        let data = JsonData::new(guild_id, members, emojis, messages);
        let mut file = std::fs::File::create("messages.json").unwrap();
        serde_json::to_writer(&mut file, &data).unwrap();
        println!("Done");
    }
}

#[allow(dead_code)]
struct Handler2;

#[async_trait]
impl EventHandler for Handler2 {
    async fn ready(&self, ctx: Context, _ready: serenity::model::gateway::Ready) {
        let guild_id = GuildId::new(986597459323150376);
        // let message_id = MessageId::new(1311985495781281872);
        let channel_id = ChannelId::new(986616694325805086);
        let channel = guild_id
            .channels(&ctx.http)
            .await
            .unwrap()
            .get(&channel_id)
            .unwrap()
            .clone();

        let messages = get_messages(
            &ctx.http,
            channel.clone(),
            None::<fn(&Message) -> bool>,
            None::<fn(&Message) -> bool>,
        )
        .await;
        println!("Messages: {:?}", messages);
        if let Ok(messages) = messages {
            messages.iter().for_each(|message| {
                println!("Message: {}", message.content);
            });
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let token = env::var("TOKEN").unwrap();
    println!("Token: {token}");
    let intents = GatewayIntents::all() - GatewayIntents::GUILD_MESSAGE_TYPING;
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
