mod message_data;

use std::collections::{HashMap, HashSet};
use std::env;
use std::hash::Hash;

use dotenvy::dotenv;
use serenity::all::{
    ChannelId, ChannelType, GuildId, Http, MessageId, PermissionOverwriteType, ReactionType,
    Result, User, UserId,
};
use serenity::model::{channel, guild};
use serenity::{
    all::{CacheHttp, Context, EventHandler, GatewayIntents, GetMessages, GuildChannel, Message},
    async_trait, Client,
};

#[allow(dead_code)]
async fn get_messages(
    cache: impl CacheHttp,
    channel: GuildChannel,
    start: Option<MessageId>,
    filter: Option<impl Fn(&Message) -> bool + Copy>,
    stop: Option<impl Fn(&Message) -> bool + Copy>,
) -> Result<Vec<Message>> {
    let mut messages: Vec<Message> = Vec::<Message>::new();
    println!("Channel: {}", channel.name);
    if !channel.is_text_based() {
        return Ok(messages);
    }
    let mut last_message_id = if start.is_some() {
        start.unwrap()
    } else {
        match channel.last_message_id {
            Some(x) => x,
            None => return Ok(messages),
        }
    };

    messages.push(channel.message(&cache, last_message_id).await?);
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

async fn get_reactions(
    http: impl AsRef<Http>,
    message: &Message,
    reaction_type: ReactionType,
) -> Result<Vec<User>> {
    let mut users = Vec::<User>::new();
    let mut last_user_id: Option<_> = None;
    let limit = 100;
    loop {
        let new_users = message
            .reaction_users(&http, reaction_type.clone(), Some(limit), last_user_id)
            .await?;
        let length = new_users.len();
        users.extend(new_users);
        if (length as u8) < limit {
            break;
        }
        last_user_id = Some(users.last().unwrap().id);
    }
    Ok(users)
}

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
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!ping" {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                println!("Error sending message: {:?}", why);
            }
        }
    }

    async fn ready(&self, ctx: Context, _ready: serenity::model::gateway::Ready) {
        let guild_id: GuildId = 1095933862657405038.into();
        let guild = guild_id.to_partial_guild(&ctx.http).await.unwrap();
        println!("Guild: {}", guild.name);
        let channels = guild.channels(&ctx.http).await.unwrap();
        let mut messages = Vec::<Message>::new();
        let now = chrono::Utc::now();
        let a_year_ago = now - chrono::Duration::days(365);
        let filter = |message: &Message| !message.author.bot;
        for (_, channel) in channels {
            if is_private_archive_channel(&channel, guild_id) {
                continue;
            }
            messages.extend(
                get_messages(
                    &ctx.http,
                    channel,
                    None,
                    Some(filter),
                    None::<fn(&Message) -> bool>,
                )
                .await
                .unwrap(),
            );
        }
        let mut member_message_counts: HashMap<UserId, usize> = HashMap::new();
        let mut use_emoji_counts: HashMap<ReactionType, usize> = HashMap::new();
        let mut mention_counts: HashMap<UserId, usize> = HashMap::new();
        let mut reaction_counts: HashMap<UserId, usize> = HashMap::new();
        for message in messages {
            let author_id = message.author.id;
            add_or_insert(&mut member_message_counts, author_id);

            let mut reaction_set = HashSet::<UserId>::new();
            for reaction in message.reactions.iter() {
                add_or_insert(&mut use_emoji_counts, reaction.reaction_type.clone());
                for user in get_reactions(&ctx.http, &message, reaction.reaction_type.clone())
                    .await
                    .unwrap()
                {
                    reaction_set.insert(user.id);
                }
            }
            for user_id in reaction_set {
                add_or_insert(&mut reaction_counts, user_id);
            }
            for memtions in message.mentions.iter() {
                add_or_insert(&mut mention_counts, memtions.id);
            }
        }
        let mut member_message_counts: Vec<_> = member_message_counts.into_iter().collect();
        member_message_counts.sort_by(|a, b| b.1.cmp(&a.1));
        for x in member_message_counts.iter() {
            println!("{}: {}", x.0, x.1);
        }
    }
}

#[allow(dead_code)]
struct Handler2;

#[async_trait]
impl EventHandler for Handler2 {
    async fn ready(&self, ctx: Context, _ready: serenity::model::gateway::Ready) {
        let guild_id = GuildId::new(524972650548953126);
        let channel_id = ChannelId::new(1298833695888900216);
        let guild = ctx.http.get_guild(guild_id).await.unwrap();
        for (_, channel) in guild.channels(&ctx.http).await.unwrap() {
            println!(
                "Channel: {}, Private:{}",
                channel.name,
                is_private_archive_channel(&channel, guild_id),
            );
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
        .event_handler(Handler2)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
