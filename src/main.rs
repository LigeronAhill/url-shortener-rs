#[tokio::main]
async fn main() -> url_shortener_rs::Result<()> {
    url_shortener_rs::run().await?;
    Ok(())
}
