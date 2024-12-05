use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serenity::all::{
    ChannelId, ChannelType, Emoji as SerenityEmoji, EmojiId, GuildChannel, GuildId, Member,
    Message, MessageId, PermissionOverwrite, ReactionType, UserId,
};

#[derive(Serialize, Deserialize, Debug, Hash, Eq, PartialEq, Clone)]
#[serde(untagged)]
#[non_exhaustive]
pub enum Emoji {
    Custom(EmojiId),
    Unicode(String),
}

impl From<EmojiId> for Emoji {
    fn from(id: EmojiId) -> Self {
        Emoji::Custom(id)
    }
}

impl From<String> for Emoji {
    fn from(string: String) -> Self {
        // if string.chars().count() != 1 {
        //     println!("Emoji string is not a single character: {}", string);
        // }
        Self::Unicode(string)
    }
}

impl From<SerenityEmoji> for Emoji {
    fn from(value: SerenityEmoji) -> Self {
        value.id.into()
    }
}

impl From<ReactionType> for Emoji {
    fn from(value: ReactionType) -> Self {
        match value {
            ReactionType::Custom { id, .. } => id.into(),
            ReactionType::Unicode(c) => c.into(),
            _ => unimplemented!(),
        }
    }
}

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

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct UserData {
    pub user_id: UserId,
    pub username: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
}

impl UserData {
    pub fn new(
        user_id: UserId,
        username: String,
        display_name: String,
        avatar_url: Option<String>,
    ) -> Self {
        Self {
            user_id,
            username,
            display_name,
            avatar_url,
        }
    }
}

impl From<Member> for UserData {
    fn from(member: Member) -> Self {
        Self {
            user_id: member.user.id,
            username: member.user.name.clone(),
            display_name: member.display_name().to_string(),
            avatar_url: member.user.avatar_url(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct EmojiData {
    pub emoji_id: EmojiId,
    pub alias: String,
    pub image_url: String,
}

impl From<SerenityEmoji> for EmojiData {
    fn from(emoji: SerenityEmoji) -> Self {
        Self {
            emoji_id: emoji.id,
            alias: emoji.name.clone(),
            image_url: emoji.url(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ChannelData {
    pub channel_id: ChannelId,
    pub name: String,
    pub channel_type: ChannelType,
    pub permission_overwrites: Vec<PermissionOverwrite>,
}

impl ChannelData {
    pub fn new(
        channel_id: ChannelId,
        name: String,
        channel_type: ChannelType,
        permission_overwrites: Vec<PermissionOverwrite>,
    ) -> Self {
        Self {
            channel_id,
            name,
            channel_type,
            permission_overwrites,
        }
    }
}

impl From<GuildChannel> for ChannelData {
    fn from(channel: GuildChannel) -> Self {
        Self {
            channel_id: channel.id,
            channel_type: channel.kind,
            permission_overwrites: channel.permission_overwrites,
            name: channel.name,
        }
    }
}

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
