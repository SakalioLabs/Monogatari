//! Standard MCP transport for safe Monogatari project authoring.

#![forbid(unsafe_code)]

pub mod cli;
pub mod protocol;
mod server;
mod validation;

use anyhow::Context;
use rmcp::ServiceExt;

pub use server::MonogatariMcpServer;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerOptions {
    pub project_root: std::path::PathBuf,
    pub allow_write: bool,
}

/// Serve MCP over stdio. No application output may be written to stdout.
pub async fn serve_stdio(options: ServerOptions) -> anyhow::Result<()> {
    let server = MonogatariMcpServer::new(options.project_root, options.allow_write)
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
