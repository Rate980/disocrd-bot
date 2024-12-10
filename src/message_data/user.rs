use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};
use serenity::all::{Member, UserId};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct UserData {
    pub user_id: UserId,
    pub username: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    #[serde(default)]
    pub is_bot: bool,
}

impl UserData {
    pub fn new(
        user_id: UserId,
        username: String,
        display_name: String,
        avatar_url: Option<String>,
        is_bot: bool,
    ) -> Self {
        Self {
            user_id,
            username,
            display_name,
            avatar_url,
            is_bot,
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
            is_bot: member.user.bot,
        }
    }
}

impl Display for UserData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name)
    }
}

impl Default for UserData {
    fn default() -> Self {
        Self {
            user_id: UserId::default(),
            username: "Unknown".to_string(),
            display_name: "Unknown".to_string(),
            avatar_url: None,
            is_bot: false,
        }
    }
}
