#[tokio::main]
async fn main() -> anyhow::Result<()> {
    backend::bootstrap::run().await
}
