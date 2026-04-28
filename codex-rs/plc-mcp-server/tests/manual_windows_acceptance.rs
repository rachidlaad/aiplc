#![cfg(windows)]

use std::env;
use std::ffi::OsString;

use codex_plc_mcp_server::tooling::TOOL_COMPILE;
use codex_plc_mcp_server::tooling::TOOL_CONNECT;
use codex_plc_mcp_server::tooling::TOOL_EXPORT_OBJECT;
use codex_plc_mcp_server::tooling::TOOL_IMPORT_OBJECT;
use codex_plc_mcp_server::tooling::TOOL_LIST_BLOCKS;
use codex_plc_mcp_server::tooling::TOOL_OPEN_PROJECT;
use codex_plc_mcp_server::tooling::TOOL_SET_BLOCK_HEADER;
use codex_plc_mcp_server::types::CompileResultEnvelope;
use codex_plc_mcp_server::types::ConnectResult;
use codex_plc_mcp_server::types::ExportObjectResult;
use codex_plc_mcp_server::types::ListBlocksResult;
use codex_plc_mcp_server::types::MutationResult;
use codex_plc_mcp_server::types::ProjectOverviewResult;
use pretty_assertions::assert_eq;
use serde_json::json;
use tempfile::tempdir;

mod common;

use common::McpTestClient;

fn required_env(key: &str) -> anyhow::Result<String> {
    env::var(key).map_err(|_| anyhow::anyhow!("missing required environment variable {key}"))
}

fn optional_env(key: &str) -> Option<String> {
    env::var(key).ok().filter(|value| !value.trim().is_empty())
}

async fn new_real_client() -> anyhow::Result<McpTestClient> {
    let adapter_command = required_env("AIPLC_TIA_ADAPTER_COMMAND")?;
    let mut args = vec![
        OsString::from("--backend"),
        OsString::from("subprocess"),
        OsString::from("--adapter-command"),
        OsString::from(adapter_command),
    ];
    if let Some(public_api_dir) = optional_env("CODEX_TIA_PUBLICAPI_DIR") {
        args.push(OsString::from("--adapter-arg"));
        args.push(OsString::from("--public-api-dir"));
        args.push(OsString::from("--adapter-arg"));
        args.push(OsString::from(public_api_dir));
    }
    if let Some(portal_version) = optional_env("CODEX_TIA_PORTAL_VERSION") {
        args.push(OsString::from("--adapter-arg"));
        args.push(OsString::from("--portal-version"));
        args.push(OsString::from("--adapter-arg"));
        args.push(OsString::from(portal_version));
    }

    let mut client = McpTestClient::spawn(args).await?;
    client.initialize().await?;
    Ok(client)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[ignore = "requires Windows, TIA Portal, and a locally approved project copy"]
async fn windows_acceptance_connect_export_edit_compile_and_restore() -> anyhow::Result<()> {
    let mut client = new_real_client().await?;
    let temp_dir = tempdir()?;
    let project_path = required_env("CODEX_TIA_PROJECT_PATH")?;
    let block_name = required_env("CODEX_TIA_BLOCK_NAME")?;
    let plc_name = optional_env("CODEX_TIA_PLC_SOFTWARE_NAME");

    let _: ConnectResult = client
        .call_tool(
            TOOL_CONNECT,
            json!({
                "connection_mode": "auto",
                "ui_mode": "with_ui",
                "portal_version": optional_env("CODEX_TIA_PORTAL_VERSION"),
            }),
        )
        .await?;

    let overview: ProjectOverviewResult = client
        .call_tool(
            TOOL_OPEN_PROJECT,
            json!({
                "project_path": project_path,
            }),
        )
        .await?;

    let plc = if let Some(plc_name) = plc_name {
        overview
            .plc_software
            .iter()
            .find(|plc| plc.object.name == plc_name)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("PLC software {plc_name} was not found"))?
    } else if overview.plc_software.len() == 1 {
        overview
            .plc_software
            .first()
            .cloned()
            .expect("single PLC present")
    } else {
        let candidates = overview
            .plc_software
            .iter()
            .map(|plc| plc.object.name.clone())
            .collect::<Vec<_>>();
        return Err(anyhow::anyhow!(
            "multiple PLC software roots found; set CODEX_TIA_PLC_SOFTWARE_NAME to one of: {}",
            candidates.join(", ")
        ));
    };

    let blocks: ListBlocksResult = client
        .call_tool(
            TOOL_LIST_BLOCKS,
            json!({
                "plc_software_id": plc.object.object_id,
                "traversal_mode": "recursive",
            }),
        )
        .await?;
    let block = blocks
        .blocks
        .iter()
        .find(|block| block.object.name == block_name)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("block {block_name} was not found"))?;

    let before_export: ExportObjectResult = client
        .call_tool(
            TOOL_EXPORT_OBJECT,
            json!({
                "object_id": block.object.object_id,
                "destination_path": temp_dir.path().join("backup.xml"),
                "read_mode": "include_text",
            }),
        )
        .await?;
    let marker = format!("CodexAcceptance-{}", std::process::id());

    let edit_result: MutationResult = client
        .call_tool(
            TOOL_SET_BLOCK_HEADER,
            json!({
                "object_id": block.object.object_id,
                "header_author": marker,
            }),
        )
        .await?;
    assert_eq!(edit_result.verification.verified, true);

    let compile_after_edit: CompileResultEnvelope = client
        .call_tool(
            TOOL_COMPILE,
            json!({
                "scope": {
                    "type": "object",
                    "object_id": block.object.object_id,
                },
            }),
        )
        .await?;
    assert_eq!(compile_after_edit.result.error_count, 0);

    let after_edit_export: ExportObjectResult = client
        .call_tool(
            TOOL_EXPORT_OBJECT,
            json!({
                "object_id": block.object.object_id,
                "destination_path": temp_dir.path().join("after-edit.xml"),
                "read_mode": "include_text",
            }),
        )
        .await?;
    let after_edit_text = after_edit_export
        .content_text
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("expected export text for verification"))?;
    assert!(
        after_edit_text.contains(&marker),
        "edited export should contain marker {marker}"
    );

    let restore_result: MutationResult = client
        .call_tool(
            TOOL_IMPORT_OBJECT,
            json!({
                "target_group_object_id": plc.block_group_object_id,
                "source_file_path": before_export.export_path,
                "conflict_mode": "override",
            }),
        )
        .await?;
    assert_eq!(restore_result.verification.verified, true);

    let compile_after_restore: CompileResultEnvelope = client
        .call_tool(
            TOOL_COMPILE,
            json!({
                "scope": {
                    "type": "object",
                    "object_id": block.object.object_id,
                },
            }),
        )
        .await?;
    assert_eq!(compile_after_restore.result.error_count, 0);

    let after_restore_export: ExportObjectResult = client
        .call_tool(
            TOOL_EXPORT_OBJECT,
            json!({
                "object_id": block.object.object_id,
                "destination_path": temp_dir.path().join("after-restore.xml"),
                "read_mode": "include_text",
            }),
        )
        .await?;
    let after_restore_text = after_restore_export
        .content_text
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("expected export text for restore verification"))?;
    assert!(
        !after_restore_text.contains(&marker),
        "restored export should no longer contain marker {marker}"
    );

    Ok(())
}
