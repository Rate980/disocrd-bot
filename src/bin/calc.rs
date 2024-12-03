use std::collections::HashMap;

use discord_bot::message_data::MessageData;
use serde_json;
use serenity::all::ChannelId;
fn main() {
    let mut file = std::fs::File::open("messages.json").unwrap();
    let messages: HashMap<ChannelId, Vec<MessageData>> = serde_json::from_reader(file).unwrap();
}
