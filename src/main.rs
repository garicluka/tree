use anyhow::Result;
use tree::app::App;

#[tokio::main]
async fn main() -> Result<()> {
    let mut app = App::new()?;
    app.run().await?;

    Ok(())
}