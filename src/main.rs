use std::env;

use serenity::{
    async_trait,
    model::{gateway::Ready, voice::VoiceState},
    prelude::*,
};
use tracing::{error, info};

struct Handler {
    loser_channel_id: u64,
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _ctx: Context, ready: Ready) {
        info!("{} is connected and ready", ready.user.name);
    }

    async fn voice_state_update(&self, ctx: Context, old: Option<VoiceState>, new: VoiceState) {
        // Only care about join events (user entering a channel, not leaving)
        let Some(channel_id) = new.channel_id else {
            return;
        };

        // Ignore pure moves between channels — only fire on fresh joins
        if let Some(ref prev) = old {
            if prev.channel_id.is_some() {
                return;
            }
        }

        let guild_id = match new.guild_id {
            Some(id) => id,
            None => return,
        };

        // Count members currently in this voice channel via the cache
        let member_count = {
            let guild = match ctx.cache.guild(guild_id) {
                Some(g) => g,
                None => return,
            };
            guild
                .voice_states
                .values()
                .filter(|vs| vs.channel_id == Some(channel_id))
                .count()
        };

        if member_count == 1 {
            let loser_channel = serenity::model::id::ChannelId::new(self.loser_channel_id);
            let msg = format!(
                "<@{}> lmaooo you're the only one here you fucking loser 💀",
                new.user_id
            );
            if let Err(e) = loser_channel.say(&ctx.http, &msg).await {
                error!("Failed to send loser message: {:?}", e);
            }
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN must be set");
    let loser_channel_id: u64 = env::var("LOSER_CHANNEL_ID")
        .expect("LOSER_CHANNEL_ID must be set")
        .parse()
        .expect("LOSER_CHANNEL_ID must be a valid u64");

    let intents = GatewayIntents::GUILDS | GatewayIntents::GUILD_VOICE_STATES;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler { loser_channel_id })
        .await
        .expect("Error creating client");

    if let Err(e) = client.start().await {
        error!("Client error: {:?}", e);
    }
}
