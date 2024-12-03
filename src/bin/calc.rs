use std::collections::HashMap;

use discord_bot::message_data::MessageData;
use serenity::all::ChannelId;
fn main() {
    let file = std::fs::File::open("messages.json").unwrap();
    let _data: HashMap<ChannelId, Vec<MessageData>> = serde_json::from_reader(file).unwrap();
}
