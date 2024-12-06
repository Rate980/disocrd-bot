use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};
use serenity::all::{Emoji as SerenityEmoji, EmojiId, ReactionType};

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
pub struct EmojiData {
    pub emoji_id: EmojiId,
    pub alias: String,
    pub image_url: String,
}
impl Display for EmojiData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "<:{}:{}>", self.alias, self.emoji_id)
    }
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
