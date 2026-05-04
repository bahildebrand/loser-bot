mod db;

use std::{env, sync::LazyLock};

use regex::Regex;
use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready, voice::VoiceState},
    prelude::*,
};
use tracing::{error, info};

static IGN_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\bign\b").expect("Invalid IGN regex"));

const IGN_YOUTUBE_PATTERNS: &[&str] = &[
    "youtube.com/@ign",
    "youtube.com/user/ign",
    "youtube.com/c/ign",
    "youtube.com/ign",
];

fn is_ign_message(content: &str) -> bool {
    let lower = content.to_lowercase();
    IGN_REGEX.is_match(content) || IGN_YOUTUBE_PATTERNS.iter().any(|p| lower.contains(p))
}

struct Handler {
    loser_channel_id: u64,
    db: libsql::Database,
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _ctx: Context, ready: Ready) {
        info!("{} is connected and ready", ready.user.name);
    }

    async fn message(&self, _ctx: Context, msg: Message) {
        if msg.author.bot {
            return;
        }
        if is_ign_message(&msg.content) {
            if let Err(e) = db::increment_count(&self.db, &msg.author.id.to_string()).await {
                error!(
                    "Failed to increment IGN count for {}: {:?}",
                    msg.author.id, e
                );
            }
        }
    }

    async fn voice_state_update(&self, ctx: Context, old: Option<VoiceState>, new: VoiceState) {
        let Some(channel_id) = new.channel_id else {
            return;
        };

        if let Some(ref prev) = old {
            if prev.channel_id.is_some() {
                return;
            }
        }

        let guild_id = match new.guild_id {
            Some(id) => id,
            None => return,
        };

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

    let turso_url = env::var("TURSO_URL").expect("TURSO_URL must be set");
    let turso_token = env::var("TURSO_AUTH_TOKEN").expect("TURSO_AUTH_TOKEN must be set");

    let db = db::connect(&turso_url, &turso_token)
        .await
        .expect("Failed to connect to Turso");
    db::run_migrations(&db)
        .await
        .expect("Failed to run migrations");

    let intents = GatewayIntents::GUILDS
        | GatewayIntents::GUILD_VOICE_STATES
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler {
            loser_channel_id,
            db,
        })
        .await
        .expect("Error creating client");

    if let Err(e) = client.start().await {
        error!("Client error: {:?}", e);
    }
}
