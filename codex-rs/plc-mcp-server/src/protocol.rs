use serde::Deserialize;
use serde::Serialize;
use serde_json::Value as JsonValue;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AdapterAction {
    Connect,
    OpenProject,
    ProjectOverview,
    ListBlocks,
    ListTagTables,
    ListDataTypes,
    ExportObject,
    ImportObject,
    ApplyEdit,
    CreateUdt,
    EditUdt,
    CreateBlock,
    EditBlockBody,
    CreateBlockCall,
    EditDbMembers,
    CreatePlcTag,
    CreateTagTable,
    ListTechnologyObjects,
    ListWatchTables,
    CreateWatchTable,
    ListNetworks,
    ListHmiObjects,
    ListSafetyObjects,
    WriteHardwareConfig,
    WriteNetworkConfig,
    CreateHmiAlarm,
    CreateTechnologyObject,
    CreateSafetyObject,
    CrossReference,
    ConsistencyCheck,
    CompareOnlineOffline,
    RunSimulation,
    GoOnline,
    DownloadToDevice,
    Compile,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AdapterRequest {
    pub id: String,
    pub action: AdapterAction,
    pub params: JsonValue,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AdapterResponse {
    pub id: String,
    pub ok: bool,
    pub result: Option<JsonValue>,
    pub error: Option<JsonValue>,
}
