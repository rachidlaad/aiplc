mod mock;
mod subprocess;

use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::Result;
use anyhow::bail;
use async_trait::async_trait;

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
use crate::types::BackendError;
use crate::types::CompileParams;
use crate::types::CompileResultEnvelope;
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

pub use mock::MockBackend;
pub use subprocess::SubprocessBackend;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendTransport {
    Simulator,
    Subprocess,
}

impl fmt::Display for BackendTransport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Simulator => write!(f, "simulator"),
            Self::Subprocess => write!(f, "subprocess"),
        }
    }
}

impl FromStr for BackendTransport {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "simulator" if cfg!(debug_assertions) => Ok(Self::Simulator),
            "simulator" => Err(
                "simulator backend is only available in debug/test builds; use subprocess for live TIA Portal"
                    .to_string(),
            ),
            "mock" => Err(
                "legacy test backend mode has been removed; use subprocess for live TIA Portal"
                    .to_string(),
            ),
            "subprocess" => Ok(Self::Subprocess),
            other => Err(format!("unsupported backend transport: {other}")),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BackendOptions {
    pub transport: BackendTransport,
    pub adapter_command: Option<String>,
    pub adapter_args: Vec<String>,
    pub simulator_state_path: Option<PathBuf>,
}

#[async_trait]
/// Backend boundary for PLC engineering operations exposed through MCP.
///
/// Implementations should keep reads deterministic and side-effect free where
/// possible, and return structured verification failures instead of reporting a
/// successful mutation that could not be read back from the engineering
/// system.
pub trait PlcBackend: Send + Sync {
    fn supported_tool_names(&self) -> &'static [&'static str];

    async fn connect(
        &mut self,
        params: ConnectParams,
    ) -> std::result::Result<ConnectResult, BackendError>;
    async fn open_project(
        &mut self,
        params: OpenProjectParams,
    ) -> std::result::Result<ProjectOverviewResult, BackendError>;
    async fn project_overview(
        &mut self,
        params: ProjectOverviewParams,
    ) -> std::result::Result<ProjectOverviewResult, BackendError>;
    async fn list_blocks(
        &mut self,
        params: ListBlocksParams,
    ) -> std::result::Result<ListBlocksResult, BackendError>;
    async fn list_tag_tables(
        &mut self,
        params: ListTagTablesParams,
    ) -> std::result::Result<ListTagTablesResult, BackendError>;
    async fn list_data_types(
        &mut self,
        params: ListDataTypesParams,
    ) -> std::result::Result<ListDataTypesResult, BackendError>;
    async fn export_object(
        &mut self,
        params: ExportObjectParams,
    ) -> std::result::Result<ExportObjectResult, BackendError>;
    async fn import_object(
        &mut self,
        params: ImportObjectParams,
    ) -> std::result::Result<MutationResult, BackendError>;
    async fn apply_edit(
        &mut self,
        params: ApplyEditParams,
    ) -> std::result::Result<MutationResult, BackendError>;
    async fn create_udt(
        &mut self,
        params: CreateUdtParams,
    ) -> std::result::Result<MutationResult, BackendError>;
    async fn edit_udt(
        &mut self,
        params: EditUdtParams,
    ) -> std::result::Result<MutationResult, BackendError>;
    async fn create_block(
        &mut self,
        params: CreateBlockParams,
    ) -> std::result::Result<MutationResult, BackendError>;
    async fn edit_block_body(
        &mut self,
        params: EditBlockBodyParams,
    ) -> std::result::Result<MutationResult, BackendError>;
    async fn create_block_call(
        &mut self,
        params: CreateBlockCallParams,
    ) -> std::result::Result<MutationResult, BackendError>;
    async fn edit_db_members(
        &mut self,
        params: EditDbMembersParams,
    ) -> std::result::Result<MutationResult, BackendError>;
    async fn create_plc_tag(
        &mut self,
        params: CreatePlcTagParams,
    ) -> std::result::Result<MutationResult, BackendError>;
    async fn create_tag_table(
        &mut self,
        params: CreateTagTableParams,
    ) -> std::result::Result<MutationResult, BackendError>;
    async fn list_technology_objects(
        &mut self,
        params: ListTechnologyObjectsParams,
    ) -> std::result::Result<ListTechnologyObjectsResult, BackendError>;
    async fn list_watch_tables(
        &mut self,
        params: ListWatchTablesParams,
    ) -> std::result::Result<ListWatchTablesResult, BackendError>;
    async fn create_watch_table(
        &mut self,
        params: CreateWatchTableParams,
    ) -> std::result::Result<MutationResult, BackendError>;
    async fn list_networks(
        &mut self,
        params: ListNetworksParams,
    ) -> std::result::Result<ListNetworksResult, BackendError>;
    async fn list_hmi_objects(
        &mut self,
        params: ListHmiObjectsParams,
    ) -> std::result::Result<ListHmiObjectsResult, BackendError>;
    async fn list_safety_objects(
        &mut self,
        params: ListSafetyObjectsParams,
    ) -> std::result::Result<ListSafetyObjectsResult, BackendError>;
    async fn write_hardware_config(
        &mut self,
        params: WriteHardwareConfigParams,
    ) -> std::result::Result<MutationResult, BackendError>;
    async fn write_network_config(
        &mut self,
        params: WriteNetworkConfigParams,
    ) -> std::result::Result<MutationResult, BackendError>;
    async fn create_hmi_alarm(
        &mut self,
        params: CreateHmiAlarmParams,
    ) -> std::result::Result<MutationResult, BackendError>;
    async fn create_technology_object(
        &mut self,
        params: CreateTechnologyObjectParams,
    ) -> std::result::Result<MutationResult, BackendError>;
    async fn create_safety_object(
        &mut self,
        params: CreateSafetyObjectParams,
    ) -> std::result::Result<MutationResult, BackendError>;
    async fn cross_reference(
        &mut self,
        params: CrossReferenceParams,
    ) -> std::result::Result<CrossReferenceResult, BackendError>;
    async fn consistency_check(
        &mut self,
        params: ConsistencyCheckParams,
    ) -> std::result::Result<ConsistencyCheckResult, BackendError>;
    async fn compare_online_offline(
        &mut self,
        params: CompareOnlineOfflineParams,
    ) -> std::result::Result<CompareOnlineOfflineResult, BackendError>;
    async fn run_simulation(
        &mut self,
        params: RunSimulationParams,
    ) -> std::result::Result<RunSimulationResult, BackendError>;
    async fn go_online(
        &mut self,
        params: GoOnlineParams,
    ) -> std::result::Result<MutationResult, BackendError>;
    async fn download_to_device(
        &mut self,
        params: DownloadToDeviceParams,
    ) -> std::result::Result<MutationResult, BackendError>;
    async fn compile(
        &mut self,
        params: CompileParams,
    ) -> std::result::Result<CompileResultEnvelope, BackendError>;
}

pub async fn build_backend(options: BackendOptions) -> Result<Box<dyn PlcBackend>> {
    match options.transport {
        BackendTransport::Simulator if cfg!(debug_assertions) => {
            Ok(Box::new(MockBackend::from_options(options).await?))
        }
        BackendTransport::Simulator => {
            bail!(
                "simulator backend is only available in debug/test builds; use subprocess for live TIA Portal"
            )
        }
        BackendTransport::Subprocess => Ok(Box::new(SubprocessBackend::spawn(options).await?)),
    }
}
