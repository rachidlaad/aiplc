use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use codex_plc_mcp_server::backend::BackendOptions;
use codex_plc_mcp_server::backend::BackendTransport;
use codex_plc_mcp_server::backend::build_backend;
use codex_plc_mcp_server::server::PlcToolServer;
use rmcp::ServiceExt;

#[derive(Debug, Parser)]
#[command(about = "Expose Siemens TIA Portal engineering operations as MCP tools.")]
struct Args {
    #[arg(long, default_value = "subprocess", hide = true)]
    backend: BackendTransport,

    #[arg(long)]
    adapter_command: Option<String>,

    #[arg(long = "adapter-arg", allow_hyphen_values = true)]
    adapter_args: Vec<String>,

    #[arg(long, hide = true)]
    simulator_state_path: Option<PathBuf>,
}

pub fn stdio() -> (tokio::io::Stdin, tokio::io::Stdout) {
    (tokio::io::stdin(), tokio::io::stdout())
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with_writer(std::io::stderr)
        .init();

    let args = Args::parse();
    let backend = build_backend(BackendOptions {
        transport: args.backend,
        adapter_command: args.adapter_command,
        adapter_args: args.adapter_args,
        simulator_state_path: args.simulator_state_path,
    })
    .await?;
    let service = PlcToolServer::new(backend);
    let running = service.serve(stdio()).await?;
    running.waiting().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use clap::Parser;
    use pretty_assertions::assert_eq;

    use super::Args;

    #[test]
    fn adapter_args_accept_hyphen_prefixed_forwarded_values() {
        let args = Args::try_parse_from([
            "codex-plc-mcp-server",
            "--backend",
            "subprocess",
            "--adapter-command",
            "adapter.exe",
            "--adapter-arg",
            "--public-api-dir",
            "--adapter-arg",
            r"C:\Program Files\Siemens\Automation\Portal V21\PublicAPI\V21\net48",
        ])
        .expect("args should parse");

        assert_eq!(
            args.adapter_args,
            vec![
                "--public-api-dir".to_string(),
                r"C:\Program Files\Siemens\Automation\Portal V21\PublicAPI\V21\net48".to_string(),
            ]
        );
    }

    #[test]
    fn default_backend_is_live_subprocess() {
        let args = Args::try_parse_from(["codex-plc-mcp-server"]).expect("args should parse");

        assert_eq!(
            args.backend,
            codex_plc_mcp_server::backend::BackendTransport::Subprocess
        );
    }

    #[test]
    fn mock_backend_name_is_not_accepted() {
        let err = Args::try_parse_from(["codex-plc-mcp-server", "--backend", "mock"])
            .expect_err("mock backend mode should be removed");

        assert!(
            err.to_string()
                .contains("mock backend mode has been removed"),
            "{err}"
        );
    }
}
