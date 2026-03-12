use crate::app;
use color_eyre::eyre::Result;

mod core;
mod server;

pub async fn run(args: app::Args) -> Result<()> {
    let log_file = core::setup_logging(args.log_dir.clone(), args.log_json, &args.log_rotation)?;
    server::start(args).await.inspect_err(|_| {
        eprintln!();
        eprintln!("Logs: {}", log_file.display());
        eprintln!();
    })
}
