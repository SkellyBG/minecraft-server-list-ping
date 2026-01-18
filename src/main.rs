mod ping;
use anyhow::Error;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let players = ping::ping()?;

    dbg!(players);

    Ok(())
}
