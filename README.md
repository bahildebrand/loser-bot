# loser_bot

tags you when you're the only person in a voice channel

## setup

copy `.env.example` to `.env` and fill it in:

```
DISCORD_TOKEN=your_token
LOSER_CHANNEL_ID=channel_to_post_in
```

get your token at [discord.com/developers/applications](https://discord.com/developers/applications). channel ID requires developer mode on in discord settings, then right-click the channel.

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
fly secrets set DISCORD_TOKEN=$DISCORD_TOKEN LOSER_CHANNEL_ID=$LOSER_CHANNEL_ID
fly deploy
```

## bot permissions

needs `Send Messages` and `View Channels` in whatever channel you point it at.
