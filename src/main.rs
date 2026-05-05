mod db;
mod loser_count;

use std::{env, sync::{Arc, LazyLock}, time::Duration};

use regex::Regex;
use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready, voice::VoiceState},
    prelude::*,
};
use tokio::time::sleep;
use tracing::{error, info};

use crate::loser_count::check_and_call_out_loser;

static IGN_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\bign\b").expect("Invalid IGN regex"));

const IGN_YOUTUBE_PATTERNS: &[&str] = &[
    "youtube.com/@ign",
    "youtube.com/user/ign",
    "youtube.com/c/ign",
    "youtube.com/ign",
];

const USER_LOGOUT_DELAY: Duration = Duration::from_mins(2);

fn is_ign_message(content: &str) -> bool {
    let lower = content.to_lowercase();
    IGN_REGEX.is_match(content) || IGN_YOUTUBE_PATTERNS.iter().any(|p| lower.contains(p))
}

struct Handler {
    loser_channel_id: u64,
    db: Arc<libsql::Database>,
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
            let conn = match self.db.connect() {
                Ok(conn) => conn,
                Err(e) => {
                    error!("Failed to connect to db: {:?}", e);
                    return;
                }
            };
            if let Err(e) = db::increment_count(conn, &msg.author.id.to_string()).await {
                error!(
                    "Failed to increment IGN count for {}: {:?}",
                    msg.author.id, e
                );
            }
        }
    }

    async fn voice_state_update(&self, ctx: Context, old: Option<VoiceState>, new: VoiceState) {
        let guild_id = match new.guild_id {
            Some(id) => id,
            None => return,
        };

        let old_channel = old.as_ref().and_then(|o| o.channel_id);
        let new_channel = new.channel_id;

        // Only handle fresh joins (None -> Some) and leaves (Some -> None).
        // Ignore moves (Some -> Some).
        let (check_channel, joiner) = match (old_channel, new_channel) {
            (None, Some(ch)) => (ch, Some(new.user_id)),
            (Some(ch), None) => (ch, None),
            _ => return,
        };

        let user_left = joiner.is_none();

        if user_left {
            let channel_id = self.loser_channel_id;
            let db = self.db.clone();
            tokio::spawn(async move {
                sleep(USER_LOGOUT_DELAY).await;
                let conn = match db.connect() {
                    Ok(c) => c,
                    Err(e) => {
                        error!("Failed to connect to db: {:?}", e);
                        return;
                    }
                };
                check_and_call_out_loser(ctx, guild_id, conn, channel_id, joiner, check_channel)
                    .await;
            });
        } else {
            let conn = match self.db.connect() {
                Ok(c) => c,
                Err(e) => {
                    error!("Failed to connect to db: {:?}", e);
                    return;
                }
            };
            check_and_call_out_loser(
                ctx,
                guild_id,
                conn,
                self.loser_channel_id,
                joiner,
                check_channel,
            )
            .await;
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

    let db = Arc::new(
        db::connect(&turso_url, &turso_token)
            .await
            .expect("Failed to connect to Turso"),
    );
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
