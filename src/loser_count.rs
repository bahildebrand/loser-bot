use libsql::Connection;
use serenity::all::{ChannelId, Context, GuildId, UserId};
use tracing::error;

use crate::db::increment_loser_count;

pub async fn check_and_call_out_loser(
    ctx: Context,
    guild_id: GuildId,
    conn: Connection,
    channel_id: u64,
    joining_user: Option<UserId>,
    check_channel: ChannelId,
) {
    let remaining: Vec<serenity::model::id::UserId> = {
        let guild = match ctx.cache.guild(guild_id) {
            Some(g) => g,
            None => return,
        };
        guild
            .voice_states
            .values()
            .filter(|vs| vs.channel_id == Some(check_channel))
            .map(|vs| vs.user_id)
            .collect()
    };

    let target_user = match joining_user {
        Some(u) if remaining.contains(&u) && remaining.len() == 1 => u,
        None if remaining.len() == 1 => remaining[0],
        _ => return,
    };

    let loser_channel = serenity::model::id::ChannelId::new(channel_id);
    let msg = format!(
        "<@{}> lmaooo you're the only one here you fucking loser 💀",
        target_user
    );
    if let Err(e) = loser_channel.say(&ctx.http, &msg).await {
        error!("Failed to send loser message: {:?}", e);
    }

    if let Err(e) = increment_loser_count(conn, &target_user.to_string()).await {
        error!(
            "Failed to increment loser count for {}: {:?}",
            target_user, e
        );
    }
}
