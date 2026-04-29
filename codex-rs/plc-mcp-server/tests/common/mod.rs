use std::ffi::OsString;
use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;
use codex_utils_cargo_bin::CargoBinError;
use pretty_assertions::assert_eq;
use serde::de::DeserializeOwned;
use serde_json::Value as JsonValue;
use serde_json::json;
use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncWriteExt;
use tokio::io::BufReader;
use tokio::process::Child;
use tokio::process::ChildStdin;
use tokio::process::ChildStdout;
use tokio::process::Command;

pub fn server_bin() -> Result<PathBuf, CargoBinError> {
    codex_utils_cargo_bin::cargo_bin("codex-plc-mcp-server")
}

pub struct McpTestClient {
    #[allow(dead_code)]
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    next_id: u64,
}

impl McpTestClient {
    pub async fn spawn(args: Vec<OsString>) -> Result<Self> {
        let mut command = Command::new(server_bin()?);
        command.args(args);
        command.kill_on_drop(true);
        command.stdin(std::process::Stdio::piped());
        command.stdout(std::process::Stdio::piped());
        command.stderr(std::process::Stdio::piped());

        let mut child = command.spawn().context("failed to spawn MCP server")?;
        if let Some(stderr) = child.stderr.take() {
            let mut stderr = BufReader::new(stderr).lines();
            tokio::spawn(async move {
                while let Ok(Some(line)) = stderr.next_line().await {
                    eprintln!("[codex-plc-mcp-server] {line}");
                }
            });
        }

        let stdin = child.stdin.take().context("stdin unavailable")?;
        let stdout = child.stdout.take().context("stdout unavailable")?;
        Ok(Self {
            child,
            stdin,
            stdout: BufReader::new(stdout),
            next_id: 1,
        })
    }

    pub async fn initialize(&mut self) -> Result<()> {
        let _ = self
            .send_request(
                "initialize",
                json!({
                    "protocolVersion": "2025-06-18",
                    "capabilities": {},
                    "clientInfo": {
                        "name": "codex-plc-mcp-test",
                        "version": "0.0.0-test",
                    },
                }),
            )
            .await?;
        self.send_notification("notifications/initialized", json!({}))
            .await?;
        Ok(())
    }

    pub async fn list_tools(&mut self) -> Result<Vec<String>> {
        let response = self.send_request("tools/list", json!({})).await?;
        let tools = response["tools"]
            .as_array()
            .context("tools/list did not return a tools array")?;
        Ok(tools
            .iter()
            .filter_map(|tool| tool["name"].as_str().map(ToOwned::to_owned))
            .collect())
    }

    pub async fn call_tool<T>(&mut self, name: &str, arguments: JsonValue) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let response = self.call_tool_raw(name, arguments).await?;
        assert_eq!(response["isError"], json!(false));
        let structured = response
            .get("structuredContent")
            .cloned()
            .context("tool result missing structuredContent")?;
        Ok(serde_json::from_value(structured)?)
    }

    pub async fn call_tool_raw(&mut self, name: &str, arguments: JsonValue) -> Result<JsonValue> {
        self.send_request(
            "tools/call",
            json!({
                "name": name,
                "arguments": arguments,
            }),
        )
        .await
    }

    async fn send_request(&mut self, method: &str, params: JsonValue) -> Result<JsonValue> {
        let id = self.next_id;
        self.next_id += 1;
        self.write_message(&json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        }))
        .await?;

        loop {
            let message = self.read_message().await?;
            if message.get("id") != Some(&json!(id)) {
                continue;
            }
            if let Some(error) = message.get("error") {
                anyhow::bail!("MCP request failed: {error}");
            }
            return message
                .get("result")
                .cloned()
                .context("response did not include result");
        }
    }

    async fn send_notification(&mut self, method: &str, params: JsonValue) -> Result<()> {
        self.write_message(&json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
        }))
        .await
    }

    async fn write_message(&mut self, message: &JsonValue) -> Result<()> {
        let body = serde_json::to_vec(message)?;
        self.stdin.write_all(&body).await?;
        self.stdin.write_all(b"\n").await?;
        self.stdin.flush().await?;
        Ok(())
    }

    async fn read_message(&mut self) -> Result<JsonValue> {
        let mut line = String::new();
        self.stdout.read_line(&mut line).await?;
        let trimmed = line.trim();
        anyhow::ensure!(!trimmed.is_empty(), "received empty JSON-RPC line");
        Ok(serde_json::from_str(trimmed)?)
    }
}
