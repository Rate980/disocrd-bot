use serenity::all::{GuildId, Http, Message, ReactionType, Result, User};

#[allow(dead_code)]
pub async fn get_reactions(
    http: impl AsRef<Http>,
    message: &Message,
    reaction_type: ReactionType,
) -> Result<Vec<User>> {
    let mut users = Vec::<User>::new();
    let mut last_user_id: Option<_> = None;
    let limit = 100;
    loop {
        let new_users = message
            .reaction_users(&http, reaction_type.clone(), Some(limit), last_user_id)
            .await?;
        let length = new_users.len();
        users.extend(new_users);
        if (length as u8) < limit {
            break;
        }
        last_user_id = Some(users.last().unwrap().id);
    }
    Ok(users)
}

#[inline]
pub fn filename(guild_id: GuildId) -> String {
    format!("outputs/{}.json", guild_id)
}
