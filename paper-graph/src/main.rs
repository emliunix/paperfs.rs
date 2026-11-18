#[tokio::main]
async fn main() -> anyhow::Result<()> {
    paper_graph::cli::run().await
}
