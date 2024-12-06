use serde::{Deserialize, Serialize};
use serenity::all::{ChannelId, ChannelType, GuildChannel, PermissionOverwrite};
use std::fmt::{Display, Formatter};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ChannelData {
    pub channel_id: ChannelId,
    pub name: String,
    pub channel_type: ChannelType,
    pub permission_overwrites: Vec<PermissionOverwrite>,
}

impl Display for ChannelData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
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
