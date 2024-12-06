use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serenity::all::{ChannelId, Message, MessageId, UserId};
use std::collections::HashMap;

use super::Emoji;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct MessageData {
    pub channel_id: ChannelId,
    pub message_id: MessageId,
    pub author_id: UserId,
    pub mentions: Vec<UserId>,
    pub reactions: HashMap<Emoji, u64>,
    pub used_emojis: Vec<Emoji>,
    pub send_time: DateTime<Utc>,
    pub edit_time: Option<DateTime<Utc>>,
    pub attachment_count: usize,
    pub num_characters: usize,
    pub is_pinned: bool,
}
impl From<Message> for MessageData {
    fn from(message: Message) -> Self {
        let mut reactions = HashMap::<Emoji, u64>::new();
        let mut used_emojis = Vec::<Emoji>::new();
        for reaction in &message.reactions {
            let emoji: Emoji = reaction.reaction_type.clone().into();
            reactions.insert(emoji.clone(), reaction.count);
            used_emojis.push(emoji);
        }
        Self {
            channel_id: message.channel_id,
            message_id: message.id,
            mentions: message.mentions.iter().map(|mention| mention.id).collect(),
            author_id: message.author.id,
            reactions,
            used_emojis,
            send_time: *message.timestamp,
            edit_time: message.edited_timestamp.map(|timestamp| *timestamp),
            attachment_count: message.attachments.len(),
            num_characters: message.content.chars().count(),
            is_pinned: message.pinned,
        }
    }
}
