use std::collections::HashMap;

use anyhow::Ok;
use serde::{Deserialize, Serialize};
use serenity::all::{ChannelId, EmojiId, MessageId, UserId};

#[derive(Serialize, Deserialize, Debug, Hash, Eq, PartialEq)]
#[serde(untagged)]
enum Emoji {
    Custom(EmojiId),
    Unicode(char),
}

impl From<EmojiId> for Emoji {
    fn from(id: EmojiId) -> Self {
        Emoji::Custom(id)
    }
}

impl From<char> for Emoji {
    fn from(c: char) -> Self {
        Emoji::Unicode(c)
    }
}

impl TryFrom<String> for Emoji {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.len() > 1 {
            return Err(anyhow::anyhow!("Emoji string too long"));
        }
        if value.is_empty() {
            return Err(anyhow::anyhow!("Emoji string too short"));
        }
        Ok(value.chars().next().unwrap().into())
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct MessageData {
    pub channel_id: ChannelId,
    pub message_id: MessageId,
    pub mentions: Vec<UserId>,
    pub reactions: HashMap<Emoji, Vec<UserId>>,
    pub used_emojis: Vec<Emoji>,
}
