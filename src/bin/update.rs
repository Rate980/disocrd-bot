use std::{collections::HashMap, env};

use discord_bot::message_data::{ChannelData, EmojiData, JsonData, UserData};
use discord_bot::utils::filename;
use dotenvy::dotenv;
use serenity::{
    all::{ChannelId, Context, EmojiId, EventHandler, GatewayIntents, GuildId, Ready, UserId},
    async_trait, Client,
};

struct Updater {
    guild_id: GuildId,
}

#[async_trait]
impl EventHandler for Updater {
    async fn ready(&self, ctx: Context, _ready: Ready) {
        let guild_id = self.guild_id;
        let file = std::fs::File::open(filename(guild_id)).unwrap();
        let mut data: JsonData = serde_json::from_reader(file).unwrap();
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

        let channels: HashMap<ChannelId, ChannelData> = guild
            .channels(&ctx.http)
            .await
            .unwrap()
            .into_iter()
            .map(|(id, c)| (id, c.into()))
            .collect();

        data.members = members;
        data.emojis = emojis;
        data.channels = channels;

        let mut file = std::fs::File::create(filename(guild_id)).unwrap();
        serde_json::to_writer(&mut file, &data).unwrap();
        println!("Done");
    }
}

#[tokio::main]
async fn main() {
    dotenv().unwrap();
    let token = env::var("TOKEN").unwrap();
    let guild_id = env::var("GUILD_ID").unwrap().parse::<GuildId>().unwrap();
    let intents = GatewayIntents::all() - GatewayIntents::GUILD_MESSAGE_TYPING;
    println!("Token: {}", token);

    let mut client = Client::builder(token, intents)
        .event_handler(Updater { guild_id })
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
