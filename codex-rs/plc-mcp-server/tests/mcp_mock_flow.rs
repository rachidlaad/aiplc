use std::ffi::OsString;

use codex_plc_mcp_server::advanced_types::CompareOnlineOfflineResult;
use codex_plc_mcp_server::advanced_types::ConsistencyCheckResult;
use codex_plc_mcp_server::advanced_types::CrossReferenceResult;
use codex_plc_mcp_server::advanced_types::ListDataTypesResult;
use codex_plc_mcp_server::advanced_types::ListHmiObjectsResult;
use codex_plc_mcp_server::advanced_types::ListNetworksResult;
use codex_plc_mcp_server::advanced_types::ListSafetyObjectsResult;
use codex_plc_mcp_server::advanced_types::ListTechnologyObjectsResult;
use codex_plc_mcp_server::advanced_types::ListWatchTablesResult;
use codex_plc_mcp_server::advanced_types::RunSimulationResult;
use codex_plc_mcp_server::tooling::LIVE_SUBPROCESS_TOOL_NAMES;
use codex_plc_mcp_server::tooling::TOOL_APPLY_EDIT;
use codex_plc_mcp_server::tooling::TOOL_COMPARE_ONLINE_OFFLINE;
use codex_plc_mcp_server::tooling::TOOL_COMPILE;
use codex_plc_mcp_server::tooling::TOOL_CONNECT;
use codex_plc_mcp_server::tooling::TOOL_CONSISTENCY_CHECK;
use codex_plc_mcp_server::tooling::TOOL_CREATE_BLOCK;
use codex_plc_mcp_server::tooling::TOOL_CREATE_BLOCK_CALL;
use codex_plc_mcp_server::tooling::TOOL_CREATE_HMI_ALARM;
use codex_plc_mcp_server::tooling::TOOL_CREATE_PLC_TAG;
use codex_plc_mcp_server::tooling::TOOL_CREATE_SAFETY_OBJECT;
use codex_plc_mcp_server::tooling::TOOL_CREATE_TAG_TABLE;
use codex_plc_mcp_server::tooling::TOOL_CREATE_TECHNOLOGY_OBJECT;
use codex_plc_mcp_server::tooling::TOOL_CREATE_UDT;
use codex_plc_mcp_server::tooling::TOOL_CREATE_WATCH_TABLE;
use codex_plc_mcp_server::tooling::TOOL_CROSS_REFERENCE;
use codex_plc_mcp_server::tooling::TOOL_DOWNLOAD_TO_DEVICE;
use codex_plc_mcp_server::tooling::TOOL_EDIT_BLOCK_BODY;
use codex_plc_mcp_server::tooling::TOOL_EDIT_DB_MEMBERS;
use codex_plc_mcp_server::tooling::TOOL_EDIT_UDT;
use codex_plc_mcp_server::tooling::TOOL_EXPORT_OBJECT;
use codex_plc_mcp_server::tooling::TOOL_GO_ONLINE;
use codex_plc_mcp_server::tooling::TOOL_IMPORT_OBJECT;
use codex_plc_mcp_server::tooling::TOOL_LIST_BLOCKS;
use codex_plc_mcp_server::tooling::TOOL_LIST_DATA_TYPES;
use codex_plc_mcp_server::tooling::TOOL_LIST_HMI_OBJECTS;
use codex_plc_mcp_server::tooling::TOOL_LIST_NETWORKS;
use codex_plc_mcp_server::tooling::TOOL_LIST_SAFETY_OBJECTS;
use codex_plc_mcp_server::tooling::TOOL_LIST_TAG_TABLES;
use codex_plc_mcp_server::tooling::TOOL_LIST_TECHNOLOGY_OBJECTS;
use codex_plc_mcp_server::tooling::TOOL_LIST_WATCH_TABLES;
use codex_plc_mcp_server::tooling::TOOL_OPEN_PROJECT;
use codex_plc_mcp_server::tooling::TOOL_PROJECT_OVERVIEW;
use codex_plc_mcp_server::tooling::TOOL_RENAME_OBJECT;
use codex_plc_mcp_server::tooling::TOOL_RUN_SIMULATION;
use codex_plc_mcp_server::tooling::TOOL_SET_BLOCK_HEADER;
use codex_plc_mcp_server::tooling::TOOL_SET_PLC_TAG_PROPERTIES;
use codex_plc_mcp_server::tooling::TOOL_WRITE_HARDWARE_CONFIG;
use codex_plc_mcp_server::tooling::TOOL_WRITE_NETWORK_CONFIG;
use codex_plc_mcp_server::types::CompileResultEnvelope;
use codex_plc_mcp_server::types::ConnectResult;
use codex_plc_mcp_server::types::ExportObjectResult;
use codex_plc_mcp_server::types::ListBlocksResult;
use codex_plc_mcp_server::types::ListTagTablesResult;
use codex_plc_mcp_server::types::MutationResult;
use codex_plc_mcp_server::types::ProjectOverviewResult;
use pretty_assertions::assert_eq;
use serde_json::Value as JsonValue;
use serde_json::json;
use tempfile::tempdir;

mod common;

use common::McpTestClient;

async fn new_mock_client() -> anyhow::Result<McpTestClient> {
    let mut client = McpTestClient::spawn(vec![
        OsString::from("--backend"),
        OsString::from("simulator"),
    ])
    .await?;
    client.initialize().await?;
    Ok(client)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn mcp_mock_server_lists_expected_tools() -> anyhow::Result<()> {
    let mut client = new_mock_client().await?;
    let tool_names = client.list_tools().await?;
    assert_eq!(
        tool_names,
        vec![
            TOOL_CONNECT.to_string(),
            TOOL_OPEN_PROJECT.to_string(),
            TOOL_PROJECT_OVERVIEW.to_string(),
            TOOL_LIST_BLOCKS.to_string(),
            TOOL_LIST_TAG_TABLES.to_string(),
            TOOL_LIST_DATA_TYPES.to_string(),
            TOOL_EXPORT_OBJECT.to_string(),
            TOOL_IMPORT_OBJECT.to_string(),
            TOOL_RENAME_OBJECT.to_string(),
            TOOL_SET_BLOCK_HEADER.to_string(),
            TOOL_SET_PLC_TAG_PROPERTIES.to_string(),
            TOOL_APPLY_EDIT.to_string(),
            TOOL_CREATE_UDT.to_string(),
            TOOL_EDIT_UDT.to_string(),
            TOOL_CREATE_BLOCK.to_string(),
            TOOL_EDIT_BLOCK_BODY.to_string(),
            TOOL_CREATE_BLOCK_CALL.to_string(),
            TOOL_EDIT_DB_MEMBERS.to_string(),
            TOOL_CREATE_PLC_TAG.to_string(),
            TOOL_CREATE_TAG_TABLE.to_string(),
            TOOL_LIST_TECHNOLOGY_OBJECTS.to_string(),
            TOOL_LIST_WATCH_TABLES.to_string(),
            TOOL_CREATE_WATCH_TABLE.to_string(),
            TOOL_LIST_NETWORKS.to_string(),
            TOOL_LIST_HMI_OBJECTS.to_string(),
            TOOL_LIST_SAFETY_OBJECTS.to_string(),
            TOOL_WRITE_HARDWARE_CONFIG.to_string(),
            TOOL_WRITE_NETWORK_CONFIG.to_string(),
            TOOL_CREATE_HMI_ALARM.to_string(),
            TOOL_CREATE_TECHNOLOGY_OBJECT.to_string(),
            TOOL_CREATE_SAFETY_OBJECT.to_string(),
            TOOL_CROSS_REFERENCE.to_string(),
            TOOL_CONSISTENCY_CHECK.to_string(),
            TOOL_COMPARE_ONLINE_OFFLINE.to_string(),
            TOOL_RUN_SIMULATION.to_string(),
            TOOL_GO_ONLINE.to_string(),
            TOOL_DOWNLOAD_TO_DEVICE.to_string(),
            TOOL_COMPILE.to_string(),
        ]
    );
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn mcp_subprocess_server_lists_full_live_contract() -> anyhow::Result<()> {
    let adapter_command = common::server_bin()?;
    let mut client = McpTestClient::spawn(vec![
        OsString::from("--backend"),
        OsString::from("subprocess"),
        OsString::from("--adapter-command"),
        adapter_command.into_os_string(),
    ])
    .await?;
    client.initialize().await?;

    let tool_names = client.list_tools().await?;
    assert_eq!(
        tool_names,
        LIVE_SUBPROCESS_TOOL_NAMES
            .iter()
            .map(|tool_name| (*tool_name).to_string())
            .collect::<Vec<_>>()
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn mcp_mock_flow_supports_inspect_edit_import_and_compile() -> anyhow::Result<()> {
    let mut client = new_mock_client().await?;
    let temp_dir = tempdir()?;

    let connect: ConnectResult = client
        .call_tool(
            TOOL_CONNECT,
            json!({
                "connection_mode": "launch",
                "ui_mode": "without_ui",
                "portal_version": "V21",
            }),
        )
        .await?;
    assert_eq!(connect.backend, "simulator");
    assert_eq!(connect.project_open, true);

    let overview: ProjectOverviewResult = client
        .call_tool(
            TOOL_OPEN_PROJECT,
            json!({
                "project_path": r"C:\Samples\SamplePackagingLine\SamplePackagingLine.ap21",
            }),
        )
        .await?;
    assert_eq!(
        overview.project.project_path,
        r"C:\Samples\SamplePackagingLine\SamplePackagingLine.ap21"
    );

    let plc = overview
        .plc_software
        .first()
        .cloned()
        .expect("mock project should expose one PLC");
    let blocks: ListBlocksResult = client
        .call_tool(
            TOOL_LIST_BLOCKS,
            json!({
                "plc_software_id": plc.object.object_id,
                "traversal_mode": "recursive",
            }),
        )
        .await?;
    let motor_fb = blocks
        .blocks
        .iter()
        .find(|block| block.object.name == "MotorFB")
        .cloned()
        .expect("mock project should expose MotorFB");

    let tag_tables: ListTagTablesResult = client
        .call_tool(
            TOOL_LIST_TAG_TABLES,
            json!({
                "plc_software_id": plc.object.object_id,
                "traversal_mode": "recursive",
                "detail_level": "include_tags",
            }),
        )
        .await?;
    assert_eq!(tag_tables.tag_tables.len(), 1);
    assert_eq!(
        tag_tables.tag_tables[0]
            .tags
            .as_ref()
            .expect("include_tags should return tags")
            .len(),
        2
    );

    let initial_export: ExportObjectResult = client
        .call_tool(
            TOOL_EXPORT_OBJECT,
            json!({
                "object_id": motor_fb.object.object_id,
                "destination_path": temp_dir.path().join("motor_fb-before.mock.json"),
                "read_mode": "include_text",
            }),
        )
        .await?;
    assert_eq!(initial_export.verification.verified, true);
    let initial_doc: JsonValue =
        serde_json::from_str(initial_export.content_text.as_deref().expect("export text"))?;
    assert_eq!(initial_doc["name"], json!("MotorFB"));

    let edit_result: MutationResult = client
        .call_tool(
            TOOL_SET_BLOCK_HEADER,
            json!({
                "object_id": motor_fb.object.object_id,
                "header_author": "AIPLC Test",
                "header_version": "2.1.1",
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
                    "object_id": motor_fb.object.object_id,
                },
            }),
        )
        .await?;
    assert_eq!(compile_after_edit.result.error_count, 0);

    let after_edit_export: ExportObjectResult = client
        .call_tool(
            TOOL_EXPORT_OBJECT,
            json!({
                "object_id": motor_fb.object.object_id,
                "destination_path": temp_dir.path().join("motor_fb-after-edit.mock.json"),
                "read_mode": "include_text",
            }),
        )
        .await?;
    let after_edit_doc: JsonValue = serde_json::from_str(
        after_edit_export
            .content_text
            .as_deref()
            .expect("export text"),
    )?;
    assert_eq!(after_edit_doc["header_author"], json!("AIPLC Test"));
    assert_eq!(after_edit_doc["header_version"], json!("2.1.1"));

    let mut imported_doc = after_edit_doc;
    imported_doc["header_author"] = json!("Imported By Test");
    imported_doc["block_body"] = json!(
        "FUNCTION_BLOCK MotorFB\nVAR_INPUT\n    Start : Bool;\nEND_VAR\n\n    MotorRunning := Start;\nEND_FUNCTION_BLOCK"
    );
    let import_path = temp_dir.path().join("motor_fb-import.mock.json");
    std::fs::write(&import_path, serde_json::to_string_pretty(&imported_doc)?)?;

    let import_result: MutationResult = client
        .call_tool(
            TOOL_IMPORT_OBJECT,
            json!({
                "target_group_object_id": plc.block_group_object_id,
                "source_file_path": import_path,
                "conflict_mode": "override",
            }),
        )
        .await?;
    assert_eq!(import_result.verification.verified, true);

    let compile_after_import: CompileResultEnvelope = client
        .call_tool(
            TOOL_COMPILE,
            json!({
                "scope": {
                    "type": "object",
                    "object_id": motor_fb.object.object_id,
                },
            }),
        )
        .await?;
    assert_eq!(compile_after_import.result.error_count, 0);

    let after_import_export: ExportObjectResult = client
        .call_tool(
            TOOL_EXPORT_OBJECT,
            json!({
                "object_id": motor_fb.object.object_id,
                "destination_path": temp_dir.path().join("motor_fb-after-import.mock.json"),
                "read_mode": "include_text",
            }),
        )
        .await?;
    let after_import_doc: JsonValue = serde_json::from_str(
        after_import_export
            .content_text
            .as_deref()
            .expect("export text"),
    )?;
    assert_eq!(after_import_doc["header_author"], json!("Imported By Test"));
    assert_eq!(
        after_import_doc["block_body"],
        json!(
            "FUNCTION_BLOCK MotorFB\nVAR_INPUT\n    Start : Bool;\nEND_VAR\n\n    MotorRunning := Start;\nEND_FUNCTION_BLOCK"
        )
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn mcp_mock_compile_surfaces_block_errors_after_import() -> anyhow::Result<()> {
    let mut client = new_mock_client().await?;
    let temp_dir = tempdir()?;

    let _: ConnectResult = client
        .call_tool(
            TOOL_CONNECT,
            json!({
                "connection_mode": "auto",
                "ui_mode": "without_ui",
            }),
        )
        .await?;
    let overview: ProjectOverviewResult =
        client.call_tool(TOOL_PROJECT_OVERVIEW, json!({})).await?;
    let plc = overview
        .plc_software
        .first()
        .cloned()
        .expect("mock project should expose one PLC");

    let bad_import_path = temp_dir.path().join("motor_fb-bad.mock.json");
    std::fs::write(
        &bad_import_path,
        serde_json::to_string_pretty(&json!({
            "object_kind": "block",
            "name": "MotorFB",
            "header_author": "Compile Error Test",
            "header_family": "Drives",
            "header_name": "Motor",
            "header_version": "9.9.9",
            "block_body": "COMPILE_ERROR",
        }))?,
    )?;

    let _: MutationResult = client
        .call_tool(
            TOOL_IMPORT_OBJECT,
            json!({
                "target_group_object_id": plc.block_group_object_id,
                "source_file_path": bad_import_path,
                "conflict_mode": "override",
            }),
        )
        .await?;

    let compile: CompileResultEnvelope = client
        .call_tool(
            TOOL_COMPILE,
            json!({
                "scope": {
                    "type": "current_project",
                },
            }),
        )
        .await?;
    assert_eq!(compile.result.error_count, 1);
    assert_eq!(compile.result.messages.len(), 1);
    assert_eq!(
        compile.result.messages[0].description.as_deref(),
        Some("Mock compile failed because block body contains COMPILE_ERROR")
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn mcp_mock_flow_supports_authoring_diagnostics_and_validation() -> anyhow::Result<()> {
    let mut client = new_mock_client().await?;

    let _: ConnectResult = client
        .call_tool(
            TOOL_CONNECT,
            json!({
                "connection_mode": "launch",
                "ui_mode": "without_ui",
                "portal_version": "V21",
            }),
        )
        .await?;

    let overview: ProjectOverviewResult =
        client.call_tool(TOOL_PROJECT_OVERVIEW, json!({})).await?;
    let plc = overview
        .plc_software
        .first()
        .cloned()
        .expect("mock project should expose one PLC");
    let plc_id = plc.object.object_id.clone();

    let initial_data_types: ListDataTypesResult = client
        .call_tool(
            TOOL_LIST_DATA_TYPES,
            json!({
                "plc_software_id": plc_id,
            }),
        )
        .await?;
    assert_eq!(initial_data_types.data_types.len(), 1);

    let create_udt: MutationResult = client
        .call_tool(
            TOOL_CREATE_UDT,
            json!({
                "plc_software_id": plc_id,
                "name": "ActuatorStatusUDT",
                "comment": "Created by mock e2e flow",
                "members": [
                    {
                        "name": "Commanded",
                        "data_type_name": "Bool",
                        "comment": "Requested state",
                        "initial_value": "FALSE"
                    },
                    {
                        "name": "Feedback",
                        "data_type_name": "Bool",
                        "comment": "Actual state",
                        "initial_value": "FALSE"
                    }
                ]
            }),
        )
        .await?;
    assert_eq!(create_udt.verification.verified, true);
    let actuator_udt_id = create_udt.touched_objects[0].object.object_id.clone();

    let edit_udt: MutationResult = client
        .call_tool(
            TOOL_EDIT_UDT,
            json!({
                "object_id": actuator_udt_id,
                "comment": "Updated by mock e2e flow",
                "members": [
                    {
                        "name": "Commanded",
                        "data_type_name": "Bool",
                        "comment": "Requested state",
                        "initial_value": "FALSE"
                    },
                    {
                        "name": "Feedback",
                        "data_type_name": "Bool",
                        "comment": "Actual state",
                        "initial_value": "FALSE"
                    },
                    {
                        "name": "Interlocked",
                        "data_type_name": "Bool",
                        "comment": "Interlock status",
                        "initial_value": "FALSE"
                    }
                ]
            }),
        )
        .await?;
    assert_eq!(edit_udt.verification.verified, true);

    let create_block: MutationResult = client
        .call_tool(
            TOOL_CREATE_BLOCK,
            json!({
                "plc_software_id": plc_id,
                "block_kind": "fb",
                "name": "ConveyorFB",
                "language": "scl",
                "header_author": "Uxarion",
                "header_family": "AIPLCDemo",
                "header_name": "Conveyor",
                "header_version": "1.0.0",
            }),
        )
        .await?;
    assert_eq!(create_block.verification.verified, true);
    let conveyor_fb_id = create_block.touched_objects[0].object.object_id.clone();

    let edit_block_body: MutationResult = client
        .call_tool(
            TOOL_EDIT_BLOCK_BODY,
            json!({
                "object_id": conveyor_fb_id.clone(),
                "language": "scl",
                "comment": "Conveyor authoring flow",
                "block_body": "FUNCTION_BLOCK ConveyorFB\nVAR_INPUT\n    Start : Bool;\nEND_VAR\nIF Start THEN\n    MotorRun := TRUE;\nEND_IF;\nEND_FUNCTION_BLOCK"
            }),
        )
        .await?;
    assert_eq!(edit_block_body.verification.verified, true);

    let blocks_before_call: ListBlocksResult = client
        .call_tool(
            TOOL_LIST_BLOCKS,
            json!({
                "plc_software_id": plc_id.clone(),
                "traversal_mode": "recursive",
            }),
        )
        .await?;
    let main_ob = blocks_before_call
        .blocks
        .iter()
        .find(|block| block.object.name == "MainOB1")
        .cloned()
        .expect("MainOB1 should exist");
    let conveyor_db = blocks_before_call
        .blocks
        .iter()
        .find(|block| block.object.name == "ConveyorStateDB")
        .cloned()
        .expect("ConveyorStateDB should exist");

    let create_block_call: MutationResult = client
        .call_tool(
            TOOL_CREATE_BLOCK_CALL,
            json!({
                "caller_block_id": main_ob.object.object_id.clone(),
                "callee_block_id": conveyor_fb_id.clone(),
                "instance_db_name": "ConveyorFB_Instance",
                "comment": "Inserted by mock e2e flow",
                "parameter_bindings": [
                    {
                        "parameter": "Start",
                        "expression": "Start"
                    }
                ]
            }),
        )
        .await?;
    assert_eq!(create_block_call.verification.verified, true);
    assert_eq!(create_block_call.touched_objects.len(), 2);

    let edit_db_members: MutationResult = client
        .call_tool(
            TOOL_EDIT_DB_MEMBERS,
            json!({
                "object_id": conveyor_db.object.object_id.clone(),
                "replace_existing": false,
                "members": [
                    {
                        "name": "Actuator",
                        "data_type_name": "ActuatorStatusUDT",
                        "comment": "Structured actuator state",
                        "initial_value": null
                    }
                ]
            }),
        )
        .await?;
    assert_eq!(edit_db_members.verification.verified, true);

    let create_tag_table: MutationResult = client
        .call_tool(
            TOOL_CREATE_TAG_TABLE,
            json!({
                "plc_software_id": plc_id.clone(),
                "name": "CommissioningTags",
            }),
        )
        .await?;
    assert_eq!(create_tag_table.verification.verified, true);
    let commissioning_table_id = create_tag_table.touched_objects[0].object.object_id.clone();

    let rename_tag_table: MutationResult = client
        .call_tool(
            TOOL_RENAME_OBJECT,
            json!({
                "object_id": commissioning_table_id.clone(),
                "new_name": "CommissioningTags_Stage",
            }),
        )
        .await?;
    assert_eq!(rename_tag_table.verification.verified, true);

    let create_tag: MutationResult = client
        .call_tool(
            TOOL_CREATE_PLC_TAG,
            json!({
                "tag_table_object_id": commissioning_table_id,
                "name": "Reset",
                "data_type_name": "Bool",
                "logical_address": "%I0.1",
                "external_accessible": true,
                "external_visible": true,
                "external_writable": false
            }),
        )
        .await?;
    assert_eq!(create_tag.verification.verified, true);
    let reset_tag_id = create_tag.touched_objects[0].object.object_id.clone();

    let set_tag_properties: MutationResult = client
        .call_tool(
            TOOL_SET_PLC_TAG_PROPERTIES,
            json!({
                "object_id": reset_tag_id,
                "name": "ResetCmd",
                "logical_address": "%I0.2",
                "external_writable": true
            }),
        )
        .await?;
    assert_eq!(set_tag_properties.verification.verified, true);

    let create_watch_table: MutationResult = client
        .call_tool(
            TOOL_CREATE_WATCH_TABLE,
            json!({
                "plc_software_id": plc_id.clone(),
                "name": "DiagnosticsWatch",
                "expressions": [
                    {
                        "expression": "Start",
                        "comment": "Command input"
                    },
                    {
                        "expression": "MotorRun",
                        "comment": "Motor output"
                    }
                ]
            }),
        )
        .await?;
    assert_eq!(create_watch_table.verification.verified, true);

    let data_types_after: ListDataTypesResult = client
        .call_tool(
            TOOL_LIST_DATA_TYPES,
            json!({
                "plc_software_id": plc_id.clone(),
            }),
        )
        .await?;
    assert_eq!(data_types_after.data_types.len(), 2);

    let technology_objects: ListTechnologyObjectsResult = client
        .call_tool(
            TOOL_LIST_TECHNOLOGY_OBJECTS,
            json!({
                "plc_software_id": plc_id.clone(),
            }),
        )
        .await?;
    assert_eq!(technology_objects.technology_objects.len(), 1);

    let watch_tables: ListWatchTablesResult = client
        .call_tool(
            TOOL_LIST_WATCH_TABLES,
            json!({
                "plc_software_id": plc_id,
            }),
        )
        .await?;
    assert_eq!(watch_tables.watch_tables.len(), 2);

    let networks: ListNetworksResult = client.call_tool(TOOL_LIST_NETWORKS, json!({})).await?;
    assert_eq!(networks.networks.len(), 1);

    let hmi_objects: ListHmiObjectsResult =
        client.call_tool(TOOL_LIST_HMI_OBJECTS, json!({})).await?;
    assert_eq!(hmi_objects.hmi_objects.len(), 1);

    let safety_objects: ListSafetyObjectsResult = client
        .call_tool(
            TOOL_LIST_SAFETY_OBJECTS,
            json!({
                "plc_software_id": plc_id.clone(),
            }),
        )
        .await?;
    assert_eq!(safety_objects.safety_objects.len(), 1);

    let cross_reference: CrossReferenceResult = client
        .call_tool(
            TOOL_CROSS_REFERENCE,
            json!({
                "object_id": conveyor_fb_id,
            }),
        )
        .await?;
    assert!(
        cross_reference.references.iter().any(
            |reference| reference.relation == "called_by" && reference.object.name == "MainOB1"
        )
    );

    let consistency: ConsistencyCheckResult = client
        .call_tool(
            TOOL_CONSISTENCY_CHECK,
            json!({
                "scope": {
                    "type": "current_project",
                }
            }),
        )
        .await?;
    assert_eq!(consistency.issue_count, 0);

    let compare: CompareOnlineOfflineResult = client
        .call_tool(
            TOOL_COMPARE_ONLINE_OFFLINE,
            json!({
                "scope": {
                    "type": "current_project",
                }
            }),
        )
        .await?;
    assert_eq!(compare.status, "in_sync");
    assert_eq!(compare.differences.len(), 0);

    let simulation: RunSimulationResult = client
        .call_tool(
            TOOL_RUN_SIMULATION,
            json!({
                "plc_software_id": plc_id,
                "duration_cycles": 3,
            }),
        )
        .await?;
    assert_eq!(simulation.status, "completed");
    assert_eq!(simulation.observations.len(), 6);

    let compile: CompileResultEnvelope = client
        .call_tool(
            TOOL_COMPILE,
            json!({
                "scope": {
                    "type": "current_project",
                },
            }),
        )
        .await?;
    assert_eq!(compile.result.error_count, 0);

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn mcp_mock_flow_supports_plant_integration_and_download_workflows() -> anyhow::Result<()> {
    let mut client = new_mock_client().await?;

    let _: ConnectResult = client
        .call_tool(
            TOOL_CONNECT,
            json!({
                "connection_mode": "launch",
                "ui_mode": "without_ui",
                "portal_version": "V21",
            }),
        )
        .await?;

    let overview: ProjectOverviewResult =
        client.call_tool(TOOL_PROJECT_OVERVIEW, json!({})).await?;
    let device = overview
        .devices
        .first()
        .cloned()
        .expect("mock project should expose one device");
    let plc = overview
        .plc_software
        .first()
        .cloned()
        .expect("mock project should expose one PLC");
    let device_id = device.object.object_id.clone();
    let plc_id = plc.object.object_id.clone();

    let tag_tables: ListTagTablesResult = client
        .call_tool(
            TOOL_LIST_TAG_TABLES,
            json!({
                "plc_software_id": plc_id.clone(),
                "traversal_mode": "recursive",
                "detail_level": "include_tags",
            }),
        )
        .await?;
    let start_tag = tag_tables
        .tag_tables
        .iter()
        .flat_map(|table| table.tags.clone().unwrap_or_default())
        .find(|tag| tag.object.name == "Start")
        .expect("mock project should expose Start tag");

    let hmi_objects_before: ListHmiObjectsResult =
        client.call_tool(TOOL_LIST_HMI_OBJECTS, json!({})).await?;
    let line_panel = hmi_objects_before
        .hmi_objects
        .iter()
        .find(|hmi| hmi.object.name == "LinePanel_1")
        .cloned()
        .expect("mock project should expose one primary HMI");
    let line_panel_id = line_panel.object.object_id.clone();

    let networks_before: ListNetworksResult =
        client.call_tool(TOOL_LIST_NETWORKS, json!({})).await?;
    let network = networks_before
        .networks
        .first()
        .cloned()
        .expect("mock project should expose one network");
    let network_id = network.object.object_id.clone();

    let write_hardware: MutationResult = client
        .call_tool(
            TOOL_WRITE_HARDWARE_CONFIG,
            json!({
                "device_object_id": device_id.clone(),
                "operation": {
                    "type": "rename_device",
                    "new_name": "PackagingStation_A",
                }
            }),
        )
        .await?;
    assert_eq!(write_hardware.verification.verified, true);

    let overview_after_hardware: ProjectOverviewResult =
        client.call_tool(TOOL_PROJECT_OVERVIEW, json!({})).await?;
    assert_eq!(
        overview_after_hardware.devices[0].object.name,
        "PackagingStation_A"
    );
    assert_eq!(
        overview_after_hardware.plc_software[0].device_name,
        "PackagingStation_A"
    );

    let rename_network: MutationResult = client
        .call_tool(
            TOOL_WRITE_NETWORK_CONFIG,
            json!({
                "network_object_id": network_id.clone(),
                "operation": {
                    "type": "rename_network",
                    "new_name": "PROFINET_MainLine",
                }
            }),
        )
        .await?;
    assert_eq!(rename_network.verification.verified, true);

    let set_network_participants: MutationResult = client
        .call_tool(
            TOOL_WRITE_NETWORK_CONFIG,
            json!({
                "network_object_id": network_id.clone(),
                "operation": {
                    "type": "set_connected_objects",
                    "connected_object_ids": [
                        device_id.clone(),
                        plc_id.clone(),
                        line_panel_id.clone()
                    ],
                }
            }),
        )
        .await?;
    assert_eq!(set_network_participants.verification.verified, true);

    let networks_after: ListNetworksResult =
        client.call_tool(TOOL_LIST_NETWORKS, json!({})).await?;
    assert_eq!(networks_after.networks[0].object.name, "PROFINET_MainLine");
    assert_eq!(networks_after.networks[0].connected_object_ids.len(), 3);

    let create_hmi_alarm: MutationResult = client
        .call_tool(
            TOOL_CREATE_HMI_ALARM,
            json!({
                "hmi_object_id": line_panel_id.clone(),
                "name": "MotorStartFault",
                "trigger_tag": start_tag.object.object_id.clone(),
                "severity": "error",
                "message": "Motor start timeout",
            }),
        )
        .await?;
    assert_eq!(create_hmi_alarm.verification.verified, true);

    let hmi_objects_after: ListHmiObjectsResult =
        client.call_tool(TOOL_LIST_HMI_OBJECTS, json!({})).await?;
    assert_eq!(hmi_objects_after.hmi_objects.len(), 2);
    assert!(
        hmi_objects_after
            .hmi_objects
            .iter()
            .any(|hmi| hmi.object.name == "MotorStartFault" && hmi.hmi_type == "alarm_error")
    );

    let create_technology_object: MutationResult = client
        .call_tool(
            TOOL_CREATE_TECHNOLOGY_OBJECT,
            json!({
                "plc_software_id": plc_id.clone(),
                "name": "Axis_Infeed",
                "technology_type": "motion_axis",
                "bound_axis": "InfeedAxis",
            }),
        )
        .await?;
    assert_eq!(create_technology_object.verification.verified, true);
    let axis_infeed_id = create_technology_object.touched_objects[0]
        .object
        .object_id
        .clone();

    let technology_objects: ListTechnologyObjectsResult = client
        .call_tool(
            TOOL_LIST_TECHNOLOGY_OBJECTS,
            json!({
                "plc_software_id": plc_id.clone(),
            }),
        )
        .await?;
    assert_eq!(technology_objects.technology_objects.len(), 2);

    let create_safety_object: MutationResult = client
        .call_tool(
            TOOL_CREATE_SAFETY_OBJECT,
            json!({
                "plc_software_id": plc_id.clone(),
                "name": "GuardDoorGroup",
                "safety_type": "guard_monitor",
            }),
        )
        .await?;
    assert_eq!(create_safety_object.verification.verified, true);
    let guard_door_group_id = create_safety_object.touched_objects[0]
        .object
        .object_id
        .clone();

    let safety_objects: ListSafetyObjectsResult = client
        .call_tool(
            TOOL_LIST_SAFETY_OBJECTS,
            json!({
                "plc_software_id": plc_id.clone(),
            }),
        )
        .await?;
    assert_eq!(safety_objects.safety_objects.len(), 2);

    let go_online: MutationResult = client
        .call_tool(
            TOOL_GO_ONLINE,
            json!({
                "device_object_id": device_id.clone(),
                "mode": "commissioning",
            }),
        )
        .await?;
    assert_eq!(go_online.verification.verified, true);

    let download: MutationResult = client
        .call_tool(
            TOOL_DOWNLOAD_TO_DEVICE,
            json!({
                "device_object_id": device_id.clone(),
                "object_ids": [
                    plc_id.clone(),
                    axis_infeed_id,
                    guard_door_group_id
                ],
                "download_mode": "software_only",
                "post_download_online_action": "go_online",
            }),
        )
        .await?;
    assert_eq!(download.verification.verified, true);

    let compare: CompareOnlineOfflineResult = client
        .call_tool(
            TOOL_COMPARE_ONLINE_OFFLINE,
            json!({
                "scope": {
                    "type": "current_project",
                }
            }),
        )
        .await?;
    assert_eq!(compare.status, "in_sync");

    let simulation: RunSimulationResult = client
        .call_tool(
            TOOL_RUN_SIMULATION,
            json!({
                "plc_software_id": plc_id,
                "duration_cycles": 2,
            }),
        )
        .await?;
    assert_eq!(simulation.status, "completed");
    assert_eq!(simulation.observations.len(), 4);

    let compile: CompileResultEnvelope = client
        .call_tool(
            TOOL_COMPILE,
            json!({
                "scope": {
                    "type": "current_project",
                },
            }),
        )
        .await?;
    assert_eq!(compile.result.error_count, 0);

    Ok(())
}
