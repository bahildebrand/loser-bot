# loser_bot

tags you when you're the only person in a voice channel. also tracks how many times each person mentions IGN or posts an IGN youtube link.

## setup

copy `.env.example` to `.env` and fill it in:

```
DISCORD_TOKEN=your_token
LOSER_CHANNEL_ID=channel_to_post_in
TURSO_URL=libsql://your-db-name.turso.io
TURSO_AUTH_TOKEN=your_turso_auth_token
```

get your token at [discord.com/developers/applications](https://discord.com/developers/applications). channel ID requires developer mode on in discord settings, then right-click the channel.

for turso, create a db and grab the credentials:

```bash
turso db create loser-bot
turso db show loser-bot   # get the URL
turso db tokens create loser-bot
```

## run

```bash
cargo run
```

or with docker:

```bash
docker build -t loser_bot .
docker run -d --env-file .env --restart unless-stopped loser_bot
```

## deploy (fly.io)

```bash
fly launch
fly secrets set DISCORD_TOKEN=$DISCORD_TOKEN LOSER_CHANNEL_ID=$LOSER_CHANNEL_ID TURSO_URL=$TURSO_URL TURSO_AUTH_TOKEN=$TURSO_AUTH_TOKEN
fly scale count 1
fly deploy
```

## querying the db

```bash
turso db shell loser-bot "SELECT * FROM ign_counts ORDER BY count DESC"
```

## bot permissions

needs `Send Messages` and `View Channels` in whatever channel you point it at. also enable **Message Content Intent** in the developer portal under Bot > Privileged Gateway Intents.
