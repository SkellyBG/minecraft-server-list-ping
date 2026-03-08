use std::sync::OnceLock;

use anyhow::Error;
use clap::Parser;
use poise::serenity_prelude as serenity;

mod ping;
use crate::ping::minecraft_ping;

static SERVER_IP: OnceLock<String> = OnceLock::new();

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long)]
    guild_id: u64,
    #[arg(long)]
    server_address: String,
}

struct Data {} // User data, which is stored and accessible in all command invocations
type Context<'a> = poise::Context<'a, Data, Error>;

/// Show who's online on the minecraft server
#[poise::command(slash_command)]
async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    let response = minecraft_ping(SERVER_IP.get().unwrap())?;
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
    let server_address = args.server_address;
    let _ = SERVER_IP.set(server_address);

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
                Ok(Data {})
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;
    client.unwrap().start().await.unwrap();
}
