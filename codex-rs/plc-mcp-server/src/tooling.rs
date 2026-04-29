use std::borrow::Cow;
use std::sync::Arc;

use rmcp::model::JsonObject;
use rmcp::model::Meta;
use rmcp::model::Tool;
use rmcp::model::ToolAnnotations;
use schemars::JsonSchema;
use schemars::r#gen::SchemaSettings;
use serde_json::json;

use crate::advanced_types::CompareOnlineOfflineParams;
use crate::advanced_types::CompareOnlineOfflineResult;
use crate::advanced_types::ConsistencyCheckParams;
use crate::advanced_types::ConsistencyCheckResult;
use crate::advanced_types::CreateBlockCallParams;
use crate::advanced_types::CreateBlockParams;
use crate::advanced_types::CreatePlcTagParams;
use crate::advanced_types::CreateTagTableParams;
use crate::advanced_types::CreateUdtParams;
use crate::advanced_types::CreateWatchTableParams;
use crate::advanced_types::CrossReferenceParams;
use crate::advanced_types::CrossReferenceResult;
use crate::advanced_types::EditBlockBodyParams;
use crate::advanced_types::EditDbMembersParams;
use crate::advanced_types::EditUdtParams;
use crate::advanced_types::ListDataTypesParams;
use crate::advanced_types::ListDataTypesResult;
use crate::advanced_types::ListHmiObjectsParams;
use crate::advanced_types::ListHmiObjectsResult;
use crate::advanced_types::ListNetworksParams;
use crate::advanced_types::ListNetworksResult;
use crate::advanced_types::ListSafetyObjectsParams;
use crate::advanced_types::ListSafetyObjectsResult;
use crate::advanced_types::ListTechnologyObjectsParams;
use crate::advanced_types::ListTechnologyObjectsResult;
use crate::advanced_types::ListWatchTablesParams;
use crate::advanced_types::ListWatchTablesResult;
use crate::advanced_types::RunSimulationParams;
use crate::advanced_types::RunSimulationResult;
use crate::integration_types::CreateHmiAlarmParams;
use crate::integration_types::CreateSafetyObjectParams;
use crate::integration_types::CreateTechnologyObjectParams;
use crate::integration_types::DownloadToDeviceParams;
use crate::integration_types::GoOnlineParams;
use crate::integration_types::WriteHardwareConfigParams;
use crate::integration_types::WriteNetworkConfigParams;
use crate::types::ApplyEditParams;
use crate::types::CompileParams;
use crate::types::ConnectParams;
use crate::types::ConnectResult;
use crate::types::ExportObjectParams;
use crate::types::ExportObjectResult;
use crate::types::ImportObjectParams;
use crate::types::ListBlocksParams;
use crate::types::ListBlocksResult;
use crate::types::ListTagTablesParams;
use crate::types::ListTagTablesResult;
use crate::types::MutationResult;
use crate::types::OpenProjectParams;
use crate::types::ProjectOverviewParams;
use crate::types::ProjectOverviewResult;
use crate::types::RenameObjectParams;
use crate::types::SetBlockHeaderParams;
use crate::types::SetPlcTagPropertiesParams;

pub const TOOL_CONNECT: &str = "tia_portal_connect";
pub const TOOL_OPEN_PROJECT: &str = "tia_portal_open_project";
pub const TOOL_PROJECT_OVERVIEW: &str = "tia_portal_project_overview";
pub const TOOL_LIST_BLOCKS: &str = "tia_portal_list_blocks";
pub const TOOL_LIST_TAG_TABLES: &str = "tia_portal_list_tag_tables";
pub const TOOL_LIST_DATA_TYPES: &str = "tia_portal_list_data_types";
pub const TOOL_EXPORT_OBJECT: &str = "tia_portal_export_object";
pub const TOOL_IMPORT_OBJECT: &str = "tia_portal_import_object";
pub const TOOL_APPLY_EDIT: &str = "tia_portal_apply_edit";
pub const TOOL_RENAME_OBJECT: &str = "tia_portal_rename_object";
pub const TOOL_SET_BLOCK_HEADER: &str = "tia_portal_set_block_header";
pub const TOOL_SET_PLC_TAG_PROPERTIES: &str = "tia_portal_set_plc_tag_properties";
pub const TOOL_CREATE_UDT: &str = "tia_portal_create_udt";
pub const TOOL_EDIT_UDT: &str = "tia_portal_edit_udt";
pub const TOOL_CREATE_BLOCK: &str = "tia_portal_create_block";
pub const TOOL_EDIT_BLOCK_BODY: &str = "tia_portal_edit_block_body";
pub const TOOL_CREATE_BLOCK_CALL: &str = "tia_portal_create_block_call";
pub const TOOL_EDIT_DB_MEMBERS: &str = "tia_portal_edit_db_members";
pub const TOOL_CREATE_PLC_TAG: &str = "tia_portal_create_plc_tag";
pub const TOOL_CREATE_TAG_TABLE: &str = "tia_portal_create_tag_table";
pub const TOOL_LIST_TECHNOLOGY_OBJECTS: &str = "tia_portal_list_technology_objects";
pub const TOOL_LIST_WATCH_TABLES: &str = "tia_portal_list_watch_tables";
pub const TOOL_CREATE_WATCH_TABLE: &str = "tia_portal_create_watch_table";
pub const TOOL_LIST_NETWORKS: &str = "tia_portal_list_networks";
pub const TOOL_LIST_HMI_OBJECTS: &str = "tia_portal_list_hmi_objects";
pub const TOOL_LIST_SAFETY_OBJECTS: &str = "tia_portal_list_safety_objects";
pub const TOOL_WRITE_HARDWARE_CONFIG: &str = "tia_portal_write_hardware_config";
pub const TOOL_WRITE_NETWORK_CONFIG: &str = "tia_portal_write_network_config";
pub const TOOL_CREATE_HMI_ALARM: &str = "tia_portal_create_hmi_alarm";
pub const TOOL_CREATE_TECHNOLOGY_OBJECT: &str = "tia_portal_create_technology_object";
pub const TOOL_CREATE_SAFETY_OBJECT: &str = "tia_portal_create_safety_object";
pub const TOOL_CROSS_REFERENCE: &str = "tia_portal_cross_reference";
pub const TOOL_CONSISTENCY_CHECK: &str = "tia_portal_consistency_check";
pub const TOOL_COMPARE_ONLINE_OFFLINE: &str = "tia_portal_compare_online_offline";
pub const TOOL_RUN_SIMULATION: &str = "tia_portal_run_simulation";
pub const TOOL_GO_ONLINE: &str = "tia_portal_go_online";
pub const TOOL_DOWNLOAD_TO_DEVICE: &str = "tia_portal_download_to_device";
pub const TOOL_COMPILE: &str = "tia_portal_compile";

pub const ALL_TOOL_NAMES: &[&str] = &[
    TOOL_CONNECT,
    TOOL_OPEN_PROJECT,
    TOOL_PROJECT_OVERVIEW,
    TOOL_LIST_BLOCKS,
    TOOL_LIST_TAG_TABLES,
    TOOL_LIST_DATA_TYPES,
    TOOL_EXPORT_OBJECT,
    TOOL_IMPORT_OBJECT,
    TOOL_RENAME_OBJECT,
    TOOL_SET_BLOCK_HEADER,
    TOOL_SET_PLC_TAG_PROPERTIES,
    TOOL_APPLY_EDIT,
    TOOL_CREATE_UDT,
    TOOL_EDIT_UDT,
    TOOL_CREATE_BLOCK,
    TOOL_EDIT_BLOCK_BODY,
    TOOL_CREATE_BLOCK_CALL,
    TOOL_EDIT_DB_MEMBERS,
    TOOL_CREATE_PLC_TAG,
    TOOL_CREATE_TAG_TABLE,
    TOOL_LIST_TECHNOLOGY_OBJECTS,
    TOOL_LIST_WATCH_TABLES,
    TOOL_CREATE_WATCH_TABLE,
    TOOL_LIST_NETWORKS,
    TOOL_LIST_HMI_OBJECTS,
    TOOL_LIST_SAFETY_OBJECTS,
    TOOL_WRITE_HARDWARE_CONFIG,
    TOOL_WRITE_NETWORK_CONFIG,
    TOOL_CREATE_HMI_ALARM,
    TOOL_CREATE_TECHNOLOGY_OBJECT,
    TOOL_CREATE_SAFETY_OBJECT,
    TOOL_CROSS_REFERENCE,
    TOOL_CONSISTENCY_CHECK,
    TOOL_COMPARE_ONLINE_OFFLINE,
    TOOL_RUN_SIMULATION,
    TOOL_GO_ONLINE,
    TOOL_DOWNLOAD_TO_DEVICE,
    TOOL_COMPILE,
];

pub const LIVE_SUBPROCESS_TOOL_NAMES: &[&str] = ALL_TOOL_NAMES;

pub fn tool_definitions() -> Vec<Tool> {
    tool_definitions_for(ALL_TOOL_NAMES)
}

pub fn tool_definitions_for(tool_names: &[&str]) -> Vec<Tool> {
    all_tool_definitions()
        .into_iter()
        .filter(|tool| {
            tool_names
                .iter()
                .any(|tool_name| *tool_name == tool.name.as_ref())
        })
        .collect()
}

fn all_tool_definitions() -> Vec<Tool> {
    vec![
        read_only_tool::<ConnectParams, ConnectResult>(
            TOOL_CONNECT,
            "Attach to or launch a local Siemens TIA Portal session.",
            &[],
        ),
        read_only_tool::<OpenProjectParams, ProjectOverviewResult>(
            TOOL_OPEN_PROJECT,
            "Open a local TIA Portal project and return the top-level engineering structure.",
            &[],
        ),
        read_only_tool::<ProjectOverviewParams, ProjectOverviewResult>(
            TOOL_PROJECT_OVERVIEW,
            "Inspect the currently opened TIA Portal project, devices, and PLC software roots.",
            &[],
        ),
        read_only_tool::<ListBlocksParams, ListBlocksResult>(
            TOOL_LIST_BLOCKS,
            "Enumerate PLC blocks and block metadata for a selected PLC software object.",
            &[],
        ),
        read_only_tool::<ListTagTablesParams, ListTagTablesResult>(
            TOOL_LIST_TAG_TABLES,
            "Enumerate PLC tag tables and optionally the tags they contain.",
            &[],
        ),
        read_only_tool::<ListDataTypesParams, ListDataTypesResult>(
            TOOL_LIST_DATA_TYPES,
            "Enumerate PLC data types, including user-defined types, for a selected PLC software object.",
            &[],
        ),
        read_only_tool::<ExportObjectParams, ExportObjectResult>(
            TOOL_EXPORT_OBJECT,
            "Export a supported TIA object and optionally return the exported text.",
            &[],
        ),
        mutating_tool::<ImportObjectParams, MutationResult>(
            TOOL_IMPORT_OBJECT,
            "Import a supported TIA object into a target group, then verify the imported result.",
            &["source_file_path"],
        ),
        mutating_tool::<RenameObjectParams, MutationResult>(
            TOOL_RENAME_OBJECT,
            "Rename a supported engineering object and verify the new name by read-back. Prefer this over tia_portal_apply_edit for simple renames.",
            &[],
        ),
        mutating_tool::<SetBlockHeaderParams, MutationResult>(
            TOOL_SET_BLOCK_HEADER,
            "Set one or more PLC block header fields on a selected block and verify them by read-back. Prefer this over tia_portal_apply_edit for block header changes.",
            &[],
        ),
        mutating_tool::<SetPlcTagPropertiesParams, MutationResult>(
            TOOL_SET_PLC_TAG_PROPERTIES,
            "Set one or more PLC tag properties on a selected tag and verify them by read-back. Prefer this over tia_portal_apply_edit for PLC tag edits.",
            &[],
        ),
        mutating_tool::<ApplyEditParams, MutationResult>(
            TOOL_APPLY_EDIT,
            "Fallback generic edit tool for supported TIA objects. Prefer tia_portal_rename_object, tia_portal_set_block_header, or tia_portal_set_plc_tag_properties when they match the requested change. The operation field must be a structured object, never free text.",
            &[],
        ),
        mutating_tool::<CreateUdtParams, MutationResult>(
            TOOL_CREATE_UDT,
            "Create a PLC user-defined data type and verify the created definition.",
            &[],
        ),
        mutating_tool::<EditUdtParams, MutationResult>(
            TOOL_EDIT_UDT,
            "Edit a PLC user-defined data type definition and verify the resulting members.",
            &[],
        ),
        mutating_tool::<CreateBlockParams, MutationResult>(
            TOOL_CREATE_BLOCK,
            "Create a PLC block or DB and verify the created engineering object.",
            &[],
        ),
        mutating_tool::<EditBlockBodyParams, MutationResult>(
            TOOL_EDIT_BLOCK_BODY,
            "Replace a block logic body and verify the persisted program text metadata.",
            &[],
        ),
        mutating_tool::<CreateBlockCallParams, MutationResult>(
            TOOL_CREATE_BLOCK_CALL,
            "Create a block call, including optional instance DB handling and parameter bindings, then verify the updated caller.",
            &[],
        ),
        mutating_tool::<EditDbMembersParams, MutationResult>(
            TOOL_EDIT_DB_MEMBERS,
            "Edit members of a global or instance DB and verify the resulting structure.",
            &[],
        ),
        mutating_tool::<CreatePlcTagParams, MutationResult>(
            TOOL_CREATE_PLC_TAG,
            "Create a PLC tag in a selected tag table and verify the created tag properties.",
            &[],
        ),
        mutating_tool::<CreateTagTableParams, MutationResult>(
            TOOL_CREATE_TAG_TABLE,
            "Create a PLC tag table and verify the created engineering object.",
            &[],
        ),
        read_only_tool::<ListTechnologyObjectsParams, ListTechnologyObjectsResult>(
            TOOL_LIST_TECHNOLOGY_OBJECTS,
            "Enumerate technology objects such as motion-related engineering objects for a PLC software root.",
            &[],
        ),
        read_only_tool::<ListWatchTablesParams, ListWatchTablesResult>(
            TOOL_LIST_WATCH_TABLES,
            "Enumerate watch tables and their expressions for a PLC software root.",
            &[],
        ),
        mutating_tool::<CreateWatchTableParams, MutationResult>(
            TOOL_CREATE_WATCH_TABLE,
            "Create a watch table or diagnostics helper and verify the created expressions.",
            &[],
        ),
        read_only_tool::<ListNetworksParams, ListNetworksResult>(
            TOOL_LIST_NETWORKS,
            "Enumerate project networks and connected engineering objects.",
            &[],
        ),
        read_only_tool::<ListHmiObjectsParams, ListHmiObjectsResult>(
            TOOL_LIST_HMI_OBJECTS,
            "Enumerate HMI engineering objects available in the current TIA project.",
            &[],
        ),
        read_only_tool::<ListSafetyObjectsParams, ListSafetyObjectsResult>(
            TOOL_LIST_SAFETY_OBJECTS,
            "Enumerate safety-related engineering objects for the current project or PLC scope.",
            &[],
        ),
        mutating_tool::<WriteHardwareConfigParams, MutationResult>(
            TOOL_WRITE_HARDWARE_CONFIG,
            "Apply a safe targeted hardware-configuration edit to a selected device and verify the changed fields.",
            &[],
        ),
        mutating_tool::<WriteNetworkConfigParams, MutationResult>(
            TOOL_WRITE_NETWORK_CONFIG,
            "Apply a safe targeted network-configuration edit to a selected network and verify the changed fields.",
            &[],
        ),
        mutating_tool::<CreateHmiAlarmParams, MutationResult>(
            TOOL_CREATE_HMI_ALARM,
            "Create an HMI alarm under a selected HMI object and verify the created definition.",
            &[],
        ),
        mutating_tool::<CreateTechnologyObjectParams, MutationResult>(
            TOOL_CREATE_TECHNOLOGY_OBJECT,
            "Create a technology object under a PLC software root and verify the created engineering object.",
            &[],
        ),
        mutating_tool::<CreateSafetyObjectParams, MutationResult>(
            TOOL_CREATE_SAFETY_OBJECT,
            "Create a safety engineering object under a PLC software root and verify the created engineering object.",
            &[],
        ),
        read_only_tool::<CrossReferenceParams, CrossReferenceResult>(
            TOOL_CROSS_REFERENCE,
            "Inspect references to a PLC engineering object across blocks, tags, and diagnostics helpers.",
            &[],
        ),
        read_only_tool::<ConsistencyCheckParams, ConsistencyCheckResult>(
            TOOL_CONSISTENCY_CHECK,
            "Run deterministic consistency checks against the current project, PLC, or object scope.",
            &[],
        ),
        read_only_tool::<CompareOnlineOfflineParams, CompareOnlineOfflineResult>(
            TOOL_COMPARE_ONLINE_OFFLINE,
            "Compare online and offline engineering state for a selected scope and return structured differences.",
            &[],
        ),
        mutating_tool::<RunSimulationParams, RunSimulationResult>(
            TOOL_RUN_SIMULATION,
            "Run a scoped simulation or commissioning dry-run and return structured observations.",
            &[],
        ),
        mutating_tool::<GoOnlineParams, MutationResult>(
            TOOL_GO_ONLINE,
            "Put a selected device into an online monitoring or commissioning state and verify the resulting session state.",
            &[],
        ),
        mutating_tool::<DownloadToDeviceParams, MutationResult>(
            TOOL_DOWNLOAD_TO_DEVICE,
            "Run a controlled download workflow to a selected device and verify the transferred scope and resulting online state.",
            &[],
        ),
        mutating_tool::<CompileParams, crate::types::CompileResultEnvelope>(
            TOOL_COMPILE,
            "Compile the current TIA project or a specific engineering object and return the compiler result tree. Valid scope.type values are current_project or object. To compile one PLC software root, pass that PLC software root object id with scope { type: object, object_id: ... }.",
            &[],
        ),
    ]
}

fn read_only_tool<I, O>(name: &'static str, description: &'static str, file_params: &[&str]) -> Tool
where
    I: JsonSchema,
    O: JsonSchema,
{
    let mut tool = base_tool::<I, O>(name, description, file_params);
    tool.annotations = Some(ToolAnnotations::new().read_only(true));
    tool
}

fn mutating_tool<I, O>(name: &'static str, description: &'static str, file_params: &[&str]) -> Tool
where
    I: JsonSchema,
    O: JsonSchema,
{
    let mut tool = base_tool::<I, O>(name, description, file_params);
    tool.annotations = Some(ToolAnnotations {
        destructive_hint: Some(true),
        idempotent_hint: None,
        open_world_hint: Some(false),
        read_only_hint: Some(false),
        title: None,
    });
    tool
}

fn base_tool<I, O>(name: &'static str, description: &'static str, file_params: &[&str]) -> Tool
where
    I: JsonSchema,
    O: JsonSchema,
{
    let mut tool = Tool::new(
        Cow::Borrowed(name),
        Cow::Borrowed(description),
        schema_object::<I>(),
    );
    tool.output_schema = Some(schema_object::<O>());
    if !file_params.is_empty() {
        let meta_value = json!({ "openai/fileParams": file_params });
        let serde_json::Value::Object(map) = meta_value else {
            unreachable!("tool meta should serialize to an object");
        };
        tool.meta = Some(Meta(map));
    }
    tool
}

fn schema_object<T>() -> Arc<JsonObject>
where
    T: JsonSchema,
{
    let schema = SchemaSettings::draft2019_09()
        .with(|settings| {
            settings.inline_subschemas = true;
            settings.option_add_null_type = false;
        })
        .into_generator()
        .into_root_schema_for::<T>();
    let schema = match serde_json::to_value(schema) {
        Ok(schema) => schema,
        Err(err) => panic!("tool schema should serialize: {err}"),
    };
    match schema {
        serde_json::Value::Object(map) => Arc::new(map),
        _ => unreachable!("tool schema should serialize to a JSON object"),
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn import_tool_marks_source_file_param_for_local_file_resolution() {
        let tool = tool_definitions()
            .into_iter()
            .find(|tool| tool.name == TOOL_IMPORT_OBJECT)
            .expect("import tool should exist");

        assert_eq!(
            tool.meta,
            Some(Meta(
                serde_json::json!({ "openai/fileParams": ["source_file_path"] })
                    .as_object()
                    .expect("object")
                    .clone()
            ))
        );
    }

    #[test]
    fn connect_tool_is_marked_read_only() {
        let tool = tool_definitions()
            .into_iter()
            .find(|tool| tool.name == TOOL_CONNECT)
            .expect("connect tool should exist");

        assert_eq!(
            tool.annotations.expect("annotations").read_only_hint,
            Some(true)
        );
    }

    #[test]
    fn advanced_contract_includes_project_authoring_and_validation_tools() {
        let tool_names = tool_definitions()
            .into_iter()
            .map(|tool| tool.name.to_string())
            .collect::<Vec<_>>();

        assert_eq!(
            tool_names,
            ALL_TOOL_NAMES
                .iter()
                .map(|tool_name| (*tool_name).to_string())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn live_subprocess_tools_expose_full_contract() {
        let tool_names = tool_definitions_for(LIVE_SUBPROCESS_TOOL_NAMES)
            .into_iter()
            .map(|tool| tool.name.to_string())
            .collect::<Vec<_>>();

        assert_eq!(
            tool_names,
            LIVE_SUBPROCESS_TOOL_NAMES
                .iter()
                .map(|tool_name| (*tool_name).to_string())
                .collect::<Vec<_>>()
        );
    }
}
