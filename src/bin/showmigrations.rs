use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    rustauth::commands::migrations::run_showmigrations(&args).await
}
