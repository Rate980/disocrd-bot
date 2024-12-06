mod channels;
mod emoji;
mod message;
mod user;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serenity::all::{ChannelId, EmojiId, GuildId, UserId};

pub use channels::ChannelData;
pub use emoji::Emoji;
pub use emoji::EmojiData;
pub use message::MessageData;
pub use user::UserData;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct JsonData {
    pub guild_id: GuildId,
    pub members: HashMap<UserId, UserData>,
    pub channels: HashMap<ChannelId, ChannelData>,
    pub emojis: HashMap<EmojiId, EmojiData>,
    pub messages: HashMap<ChannelId, Vec<MessageData>>,
}

impl JsonData {
    pub fn new(
        guild_id: GuildId,
        members: HashMap<UserId, UserData>,
        channels: HashMap<ChannelId, ChannelData>,
        emojis: HashMap<EmojiId, EmojiData>,
        messages: HashMap<ChannelId, Vec<MessageData>>,
    ) -> Self {
        Self {
            guild_id,
            members,
            channels,
            emojis,
            messages,
        }
    }
}
