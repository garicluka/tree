use tree::{app::App, types::Result};

#[tokio::main]
async fn main() -> Result {
    let mut app = App::new()?;
    app.run().await?;

    Ok(())
}
