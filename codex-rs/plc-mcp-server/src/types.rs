use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value as JsonValue;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum PortalConnectionMode {
    #[default]
    Auto,
    Attach,
    Launch,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SessionOrigin {
    Attached,
    Launched,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum TiaUiMode {
    #[default]
    WithUi,
    WithoutUi,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum TraversalMode {
    DirectChildren,
    #[default]
    Recursive,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum TagTableDetailLevel {
    #[default]
    TablesOnly,
    IncludeTags,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ExportReadMode {
    MetadataOnly,
    #[default]
    IncludeText,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ImportConflictMode {
    #[default]
    None,
    Override,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CompileScope {
    CurrentProject,
    Object { object_id: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct BackendError {
    pub code: String,
    pub message: String,
    pub details: Option<JsonValue>,
}

impl BackendError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: None,
        }
    }

    pub fn with_details(
        code: impl Into<String>,
        message: impl Into<String>,
        details: JsonValue,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: Some(details),
        }
    }
}

impl std::fmt::Display for BackendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for BackendError {}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct EngineeringObjectSummary {
    pub object_id: String,
    pub kind: String,
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct PortalProcessSummary {
    pub process_id: u32,
    pub mode: Option<String>,
    pub project_path: Option<String>,
    pub executable_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ConnectParams {
    pub connection_mode: Option<PortalConnectionMode>,
    pub ui_mode: Option<TiaUiMode>,
    pub portal_version: Option<String>,
    pub process_id: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ConnectResult {
    pub backend: String,
    pub portal_version: Option<String>,
    pub origin: SessionOrigin,
    pub process_id: Option<u32>,
    pub ui_mode: Option<TiaUiMode>,
    pub project_open: bool,
    pub processes: Vec<PortalProcessSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct OpenProjectParams {
    pub project_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ProjectOverviewParams {}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ProjectSummary {
    pub object: EngineeringObjectSummary,
    pub project_path: String,
    pub portal_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct DeviceItemSummary {
    pub object: EngineeringObjectSummary,
    pub classification: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct DeviceSummary {
    pub object: EngineeringObjectSummary,
    pub type_identifier: Option<String>,
    pub device_items: Vec<DeviceItemSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct PlcSoftwareSummary {
    pub object: EngineeringObjectSummary,
    pub device_id: String,
    pub device_name: String,
    pub block_group_object_id: String,
    pub tag_table_group_object_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ProjectOverviewResult {
    pub project: ProjectSummary,
    pub devices: Vec<DeviceSummary>,
    pub plc_software: Vec<PlcSoftwareSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ListBlocksParams {
    pub plc_software_id: String,
    pub traversal_mode: Option<TraversalMode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct BlockSummary {
    pub object: EngineeringObjectSummary,
    pub block_type: String,
    pub group_path: String,
    pub number: Option<i32>,
    pub header_author: Option<String>,
    pub header_family: Option<String>,
    pub header_name: Option<String>,
    pub header_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ListBlocksResult {
    pub plc_software_id: String,
    pub blocks: Vec<BlockSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ListTagTablesParams {
    pub plc_software_id: String,
    pub traversal_mode: Option<TraversalMode>,
    pub detail_level: Option<TagTableDetailLevel>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct PlcTagSummary {
    pub object: EngineeringObjectSummary,
    pub data_type_name: Option<String>,
    pub logical_address: Option<String>,
    pub external_accessible: Option<bool>,
    pub external_visible: Option<bool>,
    pub external_writable: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct TagTableSummary {
    pub object: EngineeringObjectSummary,
    pub group_path: String,
    pub tags: Option<Vec<PlcTagSummary>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ListTagTablesResult {
    pub plc_software_id: String,
    pub tag_tables: Vec<TagTableSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ExportObjectParams {
    pub object_id: String,
    pub destination_path: Option<String>,
    pub read_mode: Option<ExportReadMode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct VerifiedField {
    pub field: String,
    pub expected: JsonValue,
    pub actual: JsonValue,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct VerificationResult {
    pub verified: bool,
    pub strategy: String,
    pub checked_fields: Vec<VerifiedField>,
    pub exported_sha256: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ExportObjectResult {
    pub object: EngineeringObjectSummary,
    pub export_path: String,
    pub content_sha256: String,
    pub content_text: Option<String>,
    pub verification: VerificationResult,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct FieldChange {
    pub field: String,
    pub before: JsonValue,
    pub after: JsonValue,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct TouchedObject {
    pub object: EngineeringObjectSummary,
    pub changes: Vec<FieldChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ImportObjectParams {
    pub target_group_object_id: String,
    pub source_file_path: String,
    pub conflict_mode: Option<ImportConflictMode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EditOperation {
    RenameObject {
        new_name: String,
    },
    SetBlockHeader {
        header_author: Option<String>,
        header_family: Option<String>,
        header_name: Option<String>,
        header_version: Option<String>,
    },
    SetPlcTagProperties {
        name: Option<String>,
        data_type_name: Option<String>,
        logical_address: Option<String>,
        external_accessible: Option<bool>,
        external_visible: Option<bool>,
        external_writable: Option<bool>,
        is_safety: Option<bool>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct ApplyEditParams {
    pub object_id: String,
    pub operation: EditOperation,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct RenameObjectParams {
    pub object_id: String,
    pub new_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct SetBlockHeaderParams {
    pub object_id: String,
    pub header_author: Option<String>,
    pub header_family: Option<String>,
    pub header_name: Option<String>,
    pub header_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct SetPlcTagPropertiesParams {
    pub object_id: String,
    pub name: Option<String>,
    pub data_type_name: Option<String>,
    pub logical_address: Option<String>,
    pub external_accessible: Option<bool>,
    pub external_visible: Option<bool>,
    pub external_writable: Option<bool>,
    pub is_safety: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct MutationResult {
    pub touched_objects: Vec<TouchedObject>,
    pub verification: VerificationResult,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct CompilerMessageSummary {
    pub path: Option<String>,
    pub state: Option<String>,
    pub description: Option<String>,
    pub warning_count: u32,
    pub error_count: u32,
    pub messages: Vec<CompilerMessageSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct CompilerResultSummary {
    pub state: Option<String>,
    pub warning_count: u32,
    pub error_count: u32,
    pub messages: Vec<CompilerMessageSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct CompileParams {
    pub scope: CompileScope,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct CompileResultEnvelope {
    pub scope: EngineeringObjectSummary,
    pub result: CompilerResultSummary,
}
