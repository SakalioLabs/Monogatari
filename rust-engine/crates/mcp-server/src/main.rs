use std::process::ExitCode;

use monogatari_mcp::cli::{parse_args, CliAction, HELP};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> ExitCode {
    init_tracing();
    match parse_args(std::env::args_os().skip(1)) {
        Ok(CliAction::Help) => {
            eprintln!("{HELP}");
            ExitCode::SUCCESS
        }
        Ok(CliAction::Version) => {
            eprintln!("monogatari-mcp {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        Ok(CliAction::Run(options)) => match monogatari_mcp::serve_stdio(options).await {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("monogatari-mcp: {error:#}");
                ExitCode::FAILURE
            }
        },
        Err(error) => {
            eprintln!("monogatari-mcp: {error}\n\n{HELP}");
            ExitCode::from(2)
        }
    }
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("monogatari_mcp=info,rmcp=warn"));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .with_target(false)
        .try_init();
}
