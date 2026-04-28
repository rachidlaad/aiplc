use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use crate::types::EngineeringObjectSummary;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BlockAuthoringLanguage {
    Lad,
    Fbd,
    Scl,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NewBlockKind {
    Ob,
    Fb,
    Fc,
    GlobalDb,
    InstanceDb,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct UdtMemberSummary {
    pub name: String,
    pub data_type_name: String,
    pub comment: Option<String>,
    pub initial_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct DataTypeSummary {
    pub object: EngineeringObjectSummary,
    pub data_type_kind: String,
    pub comment: Option<String>,
    pub members: Vec<UdtMemberSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ListDataTypesParams {
    pub plc_software_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ListDataTypesResult {
    pub plc_software_id: String,
    pub data_types: Vec<DataTypeSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct CreateUdtParams {
    pub plc_software_id: String,
    pub name: String,
    pub comment: Option<String>,
    pub members: Vec<UdtMemberSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct EditUdtParams {
    pub object_id: String,
    pub new_name: Option<String>,
    pub comment: Option<String>,
    pub members: Option<Vec<UdtMemberSummary>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct CreateBlockParams {
    pub plc_software_id: String,
    pub block_kind: NewBlockKind,
    pub name: String,
    pub language: Option<BlockAuthoringLanguage>,
    pub block_body: Option<String>,
    pub header_author: Option<String>,
    pub header_family: Option<String>,
    pub header_name: Option<String>,
    pub header_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct EditBlockBodyParams {
    pub object_id: String,
    pub language: Option<BlockAuthoringLanguage>,
    pub block_body: String,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct BlockCallBinding {
    pub parameter: String,
    pub expression: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct CreateBlockCallParams {
    pub caller_block_id: String,
    pub callee_block_id: String,
    pub instance_db_name: Option<String>,
    pub comment: Option<String>,
    pub parameter_bindings: Vec<BlockCallBinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct DbMemberDefinition {
    pub name: String,
    pub data_type_name: String,
    pub comment: Option<String>,
    pub initial_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct EditDbMembersParams {
    pub object_id: String,
    pub replace_existing: bool,
    pub members: Vec<DbMemberDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct CreatePlcTagParams {
    pub tag_table_object_id: String,
    pub name: String,
    pub data_type_name: String,
    pub logical_address: Option<String>,
    pub external_accessible: Option<bool>,
    pub external_visible: Option<bool>,
    pub external_writable: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct CreateTagTableParams {
    pub plc_software_id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct TechnologyObjectSummary {
    pub object: EngineeringObjectSummary,
    pub technology_type: String,
    pub bound_axis: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ListTechnologyObjectsParams {
    pub plc_software_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ListTechnologyObjectsResult {
    pub plc_software_id: String,
    pub technology_objects: Vec<TechnologyObjectSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct WatchTableExpression {
    pub expression: String,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct WatchTableSummary {
    pub object: EngineeringObjectSummary,
    pub expressions: Vec<WatchTableExpression>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ListWatchTablesParams {
    pub plc_software_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ListWatchTablesResult {
    pub plc_software_id: String,
    pub watch_tables: Vec<WatchTableSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct CreateWatchTableParams {
    pub plc_software_id: String,
    pub name: String,
    pub expressions: Vec<WatchTableExpression>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct NetworkSummary {
    pub object: EngineeringObjectSummary,
    pub connected_object_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ListNetworksParams {}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ListNetworksResult {
    pub networks: Vec<NetworkSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct HmiObjectSummary {
    pub object: EngineeringObjectSummary,
    pub hmi_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ListHmiObjectsParams {}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ListHmiObjectsResult {
    pub hmi_objects: Vec<HmiObjectSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct SafetyObjectSummary {
    pub object: EngineeringObjectSummary,
    pub safety_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ListSafetyObjectsParams {
    pub plc_software_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ListSafetyObjectsResult {
    pub plc_software_id: Option<String>,
    pub safety_objects: Vec<SafetyObjectSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct CrossReferenceParams {
    pub object_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct CrossReferenceHit {
    pub object: EngineeringObjectSummary,
    pub relation: String,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct CrossReferenceResult {
    pub target: EngineeringObjectSummary,
    pub references: Vec<CrossReferenceHit>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ConsistencyCheckScope {
    CurrentProject,
    PlcSoftware { plc_software_id: String },
    Object { object_id: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ConsistencyCheckParams {
    pub scope: ConsistencyCheckScope,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ConsistencyIssue {
    pub severity: String,
    pub code: String,
    pub message: String,
    pub object: Option<EngineeringObjectSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ConsistencyCheckResult {
    pub scope: Option<EngineeringObjectSummary>,
    pub issue_count: u32,
    pub issues: Vec<ConsistencyIssue>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CompareOnlineOfflineScope {
    CurrentProject,
    PlcSoftware { plc_software_id: String },
    Object { object_id: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct CompareOnlineOfflineParams {
    pub scope: CompareOnlineOfflineScope,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct OnlineDifference {
    pub path: String,
    pub difference_type: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct CompareOnlineOfflineResult {
    pub scope: Option<EngineeringObjectSummary>,
    pub status: String,
    pub differences: Vec<OnlineDifference>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct SimulationObservation {
    pub cycle: u32,
    pub signal: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct RunSimulationParams {
    pub plc_software_id: String,
    pub duration_cycles: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct RunSimulationResult {
    pub plc_software: EngineeringObjectSummary,
    pub status: String,
    pub observations: Vec<SimulationObservation>,
}
