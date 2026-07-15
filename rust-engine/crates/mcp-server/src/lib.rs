//! Standard MCP transport for safe Monogatari project authoring.

#![forbid(unsafe_code)]

pub mod cli;
mod package_transport;
mod project_lease;
pub mod protocol;
mod provenance;
mod server;
mod validation;

use anyhow::Context;
use rmcp::ServiceExt;

pub use server::MonogatariMcpServer;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerOptions {
    pub project_root: std::path::PathBuf,
    pub allow_write: bool,
    pub package_output_dir: Option<std::path::PathBuf>,
}

/// Serve MCP over stdio. No application output may be written to stdout.
pub async fn serve_stdio(options: ServerOptions) -> anyhow::Result<()> {
    let server = MonogatariMcpServer::new_with_package_output(
        options.project_root,
        options.allow_write,
        options.package_output_dir,
    )
    .map_err(anyhow::Error::msg)?;
    server
        .serve(rmcp::transport::stdio())
        .await
        .context("failed to initialize MCP stdio transport")?
        .waiting()
        .await
        .context("MCP stdio transport stopped with an error")?;
    Ok(())
}
