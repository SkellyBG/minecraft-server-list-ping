use std::{sync::LazyLock, time::Duration};

use anyhow::Error;
use clap::Parser;
use poise::serenity_prelude as serenity;

mod ping;
use crate::ping::minecraft_ping;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long)]
    guild_id: u64,
    #[arg(long)]
    server_address: String,
    #[arg(long)]
    channel_id: u64,
}

static SERVER_ADDRESS: LazyLock<String> = LazyLock::new(|| Args::parse().server_address);

struct Data {} // User data, which is stored and accessible in all command invocations
type Context<'a> = poise::Context<'a, Data, Error>;

/// Show who's online on the minecraft server
#[poise::command(slash_command)]
async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    let response = minecraft_ping(&SERVER_ADDRESS)?;
    let mut response: Vec<String> = response.into_iter().collect();
    response.sort_unstable();
    ctx.say(format!(
        "The following player(s) are online: {}",
        response.join(", ")
    ))
    .await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let guild_id = args.guild_id;

    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![ping()],
            ..Default::default()
        })
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                // poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                poise::builtins::register_in_guild(
                    ctx,
                    &framework.options().commands,
                    guild_id.into(),
                )
                .await?;
                {
                    let ctx = ctx.clone();
                    tokio::spawn(async move { spawn_poll(ctx).await });
                }
                Ok(Data {})
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;

    client.unwrap().start().await.unwrap();
}

async fn spawn_poll(ctx: poise::serenity_prelude::Context) {
    let channel_id = serenity::ChannelId::new(Args::parse().channel_id);
    let mut current_players = minecraft_ping(&SERVER_ADDRESS).unwrap();
    loop {
        let new_players = minecraft_ping(&SERVER_ADDRESS).unwrap();

        for new_player in new_players.iter() {
            if !current_players.contains(new_player) && new_player != "Anonymous Player" {
                let _ = channel_id
                    .say(ctx.clone(), format!("{} joined the server!", new_player))
                    .await;
            }
        }

        for cur_player in current_players.iter() {
            if !new_players.contains(cur_player) && cur_player != "Anonymous Player" {
                let _ = channel_id
                    .say(ctx.clone(), format!("{} left the server!", cur_player))
                    .await;
            }
        }

        current_players = new_players;
        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}
