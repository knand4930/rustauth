use anyhow::Result;

fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    rustauth::commands::startapp::run(&args)
}
