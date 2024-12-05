use std::collections::HashMap;
use std::env;
use std::hash::Hash;

use discord_bot::message_data::{EmojiData, JsonData, MessageData, UserData};
use dotenvy::dotenv;
use serenity::all::{
    ChannelId, EmojiId, GuildId, MessageId, PermissionOverwriteType, Result, UserId,
};
use serenity::{
    all::{CacheHttp, Context, EventHandler, GatewayIntents, GetMessages, GuildChannel, Message},
    async_trait, Client,
};

#[allow(non_upper_case_globals)]
const _x128_GUILD_ID: u64 = 854616415323815936;
#[allow(non_upper_case_globals)]
const _IoT_GUILD_ID: u64 = 1095933862657405038;

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
        let guild_id: GuildId = _x128_GUILD_ID.into();
        // let guild_id = GuildId::new(986597459323150376);
        //854616415323815936
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
        for (_, channel) in channels.iter() {
            if is_private_archive_channel(channel, guild_id) {
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

        let channels = channels.into_iter().map(|(id, c)| (id, c.into())).collect();
        let data = JsonData::new(guild_id, members, channels, emojis, messages);
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
        // let guild_id = GuildId::new(986597459323150376);
        let message_id = MessageId::new(1311603619300114464);
        let channel_id = ChannelId::new(596760218755399685);
        let message = channel_id.message(&ctx.http, message_id).await.unwrap();
        for reaction in message.reactions.iter() {
            let reaction_type = &reaction.reaction_type;
            println!("{:?}", reaction.reaction_type);
            let users = channel_id
                .reaction_users(&ctx.http, message_id, reaction_type.clone(), None, None)
                .await
                .unwrap();
            println!("{:?}", users);
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv().unwrap();
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
