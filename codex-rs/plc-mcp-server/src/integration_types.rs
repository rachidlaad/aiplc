use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HardwareConfigOperation {
    RenameDevice { new_name: String },
    SetProfinetDeviceName { profinet_device_name: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct WriteHardwareConfigParams {
    pub device_object_id: String,
    pub operation: HardwareConfigOperation,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NetworkConfigOperation {
    RenameNetwork { new_name: String },
    SetConnectedObjects { connected_object_ids: Vec<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct WriteNetworkConfigParams {
    pub network_object_id: String,
    pub operation: NetworkConfigOperation,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HmiAlarmSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct CreateHmiAlarmParams {
    pub hmi_object_id: String,
    pub name: String,
    pub trigger_tag: String,
    pub severity: HmiAlarmSeverity,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct CreateTechnologyObjectParams {
    pub plc_software_id: String,
    pub name: String,
    pub technology_type: String,
    pub bound_axis: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct CreateSafetyObjectParams {
    pub plc_software_id: String,
    pub name: String,
    pub safety_type: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum OnlineSessionMode {
    #[default]
    Monitor,
    Commissioning,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct GoOnlineParams {
    pub device_object_id: String,
    pub mode: Option<OnlineSessionMode>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum DownloadMode {
    #[default]
    HardwareAndSoftware,
    SoftwareOnly,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum PostDownloadOnlineAction {
    #[default]
    LeaveOffline,
    GoOnline,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct DownloadToDeviceParams {
    pub device_object_id: String,
    pub object_ids: Option<Vec<String>>,
    pub download_mode: Option<DownloadMode>,
    pub post_download_online_action: Option<PostDownloadOnlineAction>,
}
