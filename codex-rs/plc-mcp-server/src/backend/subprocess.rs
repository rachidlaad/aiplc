use std::process::Stdio;

use anyhow::Context;
use anyhow::Result;
use async_trait::async_trait;
use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncWriteExt;
use tokio::io::BufReader;
use tokio::process::Child;
use tokio::process::ChildStdin;
use tokio::process::ChildStdout;
use tokio::process::Command;
use uuid::Uuid;

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
use crate::backend::BackendOptions;
use crate::backend::PlcBackend;
use crate::integration_types::CreateHmiAlarmParams;
use crate::integration_types::CreateSafetyObjectParams;
use crate::integration_types::CreateTechnologyObjectParams;
use crate::integration_types::DownloadToDeviceParams;
use crate::integration_types::GoOnlineParams;
use crate::integration_types::WriteHardwareConfigParams;
use crate::integration_types::WriteNetworkConfigParams;
use crate::protocol::AdapterAction;
use crate::protocol::AdapterRequest;
use crate::protocol::AdapterResponse;
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

pub struct SubprocessBackend {
    #[allow(dead_code)]
    child: Child,
    stdin: ChildStdin,
    stdout: tokio::io::Lines<BufReader<ChildStdout>>,
}

impl SubprocessBackend {
    pub async fn spawn(options: BackendOptions) -> Result<Self> {
        let adapter_command = options
            .adapter_command
            .context("subprocess backend requires --adapter-command")?;
        let mut command = Command::new(adapter_command);
        command.args(options.adapter_args);
        command.stdin(Stdio::piped());
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());
        command.kill_on_drop(true);

        let mut child = command.spawn().context("failed to spawn TIA adapter")?;
        let stdin = child.stdin.take().context("adapter stdin unavailable")?;
        let stdout = child.stdout.take().context("adapter stdout unavailable")?;
        if let Some(stderr) = child.stderr.take() {
            let mut stderr_lines = BufReader::new(stderr).lines();
            tokio::spawn(async move {
                while let Ok(Some(line)) = stderr_lines.next_line().await {
                    eprintln!("[tia-adapter] {line}");
                }
            });
        }

        Ok(Self {
            child,
            stdin,
            stdout: BufReader::new(stdout).lines(),
        })
    }

    async fn request<T, R>(
        &mut self,
        action: AdapterAction,
        params: T,
    ) -> std::result::Result<R, BackendError>
    where
        T: serde::Serialize,
        R: serde::de::DeserializeOwned,
    {
        let request = AdapterRequest {
            id: Uuid::new_v4().to_string(),
            action,
            params: serde_json::to_value(params).map_err(|err| {
                BackendError::with_details(
                    "serialization_error",
                    "failed to serialize adapter request",
                    serde_json::json!({ "error": err.to_string() }),
                )
            })?,
        };
        let line = serde_json::to_string(&request).map_err(|err| {
            BackendError::with_details(
                "serialization_error",
                "failed to encode adapter request",
                serde_json::json!({ "error": err.to_string() }),
            )
        })?;
        self.stdin
            .write_all(line.as_bytes())
            .await
            .map_err(|err| BackendError::new("io_error", format!("adapter write failed: {err}")))?;
        self.stdin
            .write_all(b"\n")
            .await
            .map_err(|err| BackendError::new("io_error", format!("adapter write failed: {err}")))?;
        self.stdin
            .flush()
            .await
            .map_err(|err| BackendError::new("io_error", format!("adapter flush failed: {err}")))?;

        let response_line = self
            .stdout
            .next_line()
            .await
            .map_err(|err| BackendError::new("io_error", format!("adapter read failed: {err}")))?
            .ok_or_else(|| BackendError::new("adapter_closed", "adapter process exited"))?;
        let response: AdapterResponse = serde_json::from_str(&response_line).map_err(|err| {
            BackendError::with_details(
                "parse_error",
                "failed to parse adapter response",
                serde_json::json!({ "error": err.to_string(), "line": response_line }),
            )
        })?;
        if !response.ok {
            return Err(response
                .error
                .and_then(|value| serde_json::from_value::<BackendError>(value).ok())
                .unwrap_or_else(|| {
                    BackendError::new("adapter_error", "adapter reported failure")
                }));
        }

        serde_json::from_value(response.result.unwrap_or_default()).map_err(|err| {
            BackendError::with_details(
                "parse_error",
                "failed to decode adapter result",
                serde_json::json!({ "error": err.to_string() }),
            )
        })
    }
}

#[async_trait]
impl PlcBackend for SubprocessBackend {
    fn supported_tool_names(&self) -> &'static [&'static str] {
        crate::tooling::LIVE_SUBPROCESS_TOOL_NAMES
    }

    async fn connect(
        &mut self,
        params: ConnectParams,
    ) -> std::result::Result<ConnectResult, BackendError> {
        self.request(AdapterAction::Connect, params).await
    }

    async fn open_project(
        &mut self,
        params: OpenProjectParams,
    ) -> std::result::Result<ProjectOverviewResult, BackendError> {
        self.request(AdapterAction::OpenProject, params).await
    }

    async fn project_overview(
        &mut self,
        params: ProjectOverviewParams,
    ) -> std::result::Result<ProjectOverviewResult, BackendError> {
        self.request(AdapterAction::ProjectOverview, params).await
    }

    async fn list_blocks(
        &mut self,
        params: ListBlocksParams,
    ) -> std::result::Result<ListBlocksResult, BackendError> {
        self.request(AdapterAction::ListBlocks, params).await
    }

    async fn list_tag_tables(
        &mut self,
        params: ListTagTablesParams,
    ) -> std::result::Result<ListTagTablesResult, BackendError> {
        self.request(AdapterAction::ListTagTables, params).await
    }

    async fn list_data_types(
        &mut self,
        params: ListDataTypesParams,
    ) -> std::result::Result<ListDataTypesResult, BackendError> {
        self.request(AdapterAction::ListDataTypes, params).await
    }

    async fn export_object(
        &mut self,
        params: ExportObjectParams,
    ) -> std::result::Result<ExportObjectResult, BackendError> {
        self.request(AdapterAction::ExportObject, params).await
    }

    async fn import_object(
        &mut self,
        params: ImportObjectParams,
    ) -> std::result::Result<MutationResult, BackendError> {
        self.request(AdapterAction::ImportObject, params).await
    }

    async fn apply_edit(
        &mut self,
        params: ApplyEditParams,
    ) -> std::result::Result<MutationResult, BackendError> {
        self.request(AdapterAction::ApplyEdit, params).await
    }

    async fn create_udt(
        &mut self,
        params: CreateUdtParams,
    ) -> std::result::Result<MutationResult, BackendError> {
        self.request(AdapterAction::CreateUdt, params).await
    }

    async fn edit_udt(
        &mut self,
        params: EditUdtParams,
    ) -> std::result::Result<MutationResult, BackendError> {
        self.request(AdapterAction::EditUdt, params).await
    }

    async fn create_block(
        &mut self,
        params: CreateBlockParams,
    ) -> std::result::Result<MutationResult, BackendError> {
        self.request(AdapterAction::CreateBlock, params).await
    }

    async fn edit_block_body(
        &mut self,
        params: EditBlockBodyParams,
    ) -> std::result::Result<MutationResult, BackendError> {
        self.request(AdapterAction::EditBlockBody, params).await
    }

    async fn create_block_call(
        &mut self,
        params: CreateBlockCallParams,
    ) -> std::result::Result<MutationResult, BackendError> {
        self.request(AdapterAction::CreateBlockCall, params).await
    }

    async fn edit_db_members(
        &mut self,
        params: EditDbMembersParams,
    ) -> std::result::Result<MutationResult, BackendError> {
        self.request(AdapterAction::EditDbMembers, params).await
    }

    async fn create_plc_tag(
        &mut self,
        params: CreatePlcTagParams,
    ) -> std::result::Result<MutationResult, BackendError> {
        self.request(AdapterAction::CreatePlcTag, params).await
    }

    async fn create_tag_table(
        &mut self,
        params: CreateTagTableParams,
    ) -> std::result::Result<MutationResult, BackendError> {
        self.request(AdapterAction::CreateTagTable, params).await
    }

    async fn list_technology_objects(
        &mut self,
        params: ListTechnologyObjectsParams,
    ) -> std::result::Result<ListTechnologyObjectsResult, BackendError> {
        self.request(AdapterAction::ListTechnologyObjects, params)
            .await
    }

    async fn list_watch_tables(
        &mut self,
        params: ListWatchTablesParams,
    ) -> std::result::Result<ListWatchTablesResult, BackendError> {
        self.request(AdapterAction::ListWatchTables, params).await
    }

    async fn create_watch_table(
        &mut self,
        params: CreateWatchTableParams,
    ) -> std::result::Result<MutationResult, BackendError> {
        self.request(AdapterAction::CreateWatchTable, params).await
    }

    async fn list_networks(
        &mut self,
        params: ListNetworksParams,
    ) -> std::result::Result<ListNetworksResult, BackendError> {
        self.request(AdapterAction::ListNetworks, params).await
    }

    async fn list_hmi_objects(
        &mut self,
        params: ListHmiObjectsParams,
    ) -> std::result::Result<ListHmiObjectsResult, BackendError> {
        self.request(AdapterAction::ListHmiObjects, params).await
    }

    async fn list_safety_objects(
        &mut self,
        params: ListSafetyObjectsParams,
    ) -> std::result::Result<ListSafetyObjectsResult, BackendError> {
        self.request(AdapterAction::ListSafetyObjects, params).await
    }

    async fn write_hardware_config(
        &mut self,
        params: WriteHardwareConfigParams,
    ) -> std::result::Result<MutationResult, BackendError> {
        self.request(AdapterAction::WriteHardwareConfig, params)
            .await
    }

    async fn write_network_config(
        &mut self,
        params: WriteNetworkConfigParams,
    ) -> std::result::Result<MutationResult, BackendError> {
        self.request(AdapterAction::WriteNetworkConfig, params)
            .await
    }

    async fn create_hmi_alarm(
        &mut self,
        params: CreateHmiAlarmParams,
    ) -> std::result::Result<MutationResult, BackendError> {
        self.request(AdapterAction::CreateHmiAlarm, params).await
    }

    async fn create_technology_object(
        &mut self,
        params: CreateTechnologyObjectParams,
    ) -> std::result::Result<MutationResult, BackendError> {
        self.request(AdapterAction::CreateTechnologyObject, params)
            .await
    }

    async fn create_safety_object(
        &mut self,
        params: CreateSafetyObjectParams,
    ) -> std::result::Result<MutationResult, BackendError> {
        self.request(AdapterAction::CreateSafetyObject, params)
            .await
    }

    async fn cross_reference(
        &mut self,
        params: CrossReferenceParams,
    ) -> std::result::Result<CrossReferenceResult, BackendError> {
        self.request(AdapterAction::CrossReference, params).await
    }

    async fn consistency_check(
        &mut self,
        params: ConsistencyCheckParams,
    ) -> std::result::Result<ConsistencyCheckResult, BackendError> {
        self.request(AdapterAction::ConsistencyCheck, params).await
    }

    async fn compare_online_offline(
        &mut self,
        params: CompareOnlineOfflineParams,
    ) -> std::result::Result<CompareOnlineOfflineResult, BackendError> {
        self.request(AdapterAction::CompareOnlineOffline, params)
            .await
    }

    async fn run_simulation(
        &mut self,
        params: RunSimulationParams,
    ) -> std::result::Result<RunSimulationResult, BackendError> {
        self.request(AdapterAction::RunSimulation, params).await
    }

    async fn go_online(
        &mut self,
        params: GoOnlineParams,
    ) -> std::result::Result<MutationResult, BackendError> {
        self.request(AdapterAction::GoOnline, params).await
    }

    async fn download_to_device(
        &mut self,
        params: DownloadToDeviceParams,
    ) -> std::result::Result<MutationResult, BackendError> {
        self.request(AdapterAction::DownloadToDevice, params).await
    }

    async fn compile(
        &mut self,
        params: CompileParams,
    ) -> std::result::Result<CompileResultEnvelope, BackendError> {
        self.request(AdapterAction::Compile, params).await
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::protocol::AdapterAction;
    use crate::protocol::AdapterRequest;

    #[test]
    fn adapter_request_serializes_snake_case_action() {
        let request = AdapterRequest {
            id: "req-1".to_string(),
            action: AdapterAction::ProjectOverview,
            params: serde_json::json!({}),
        };

        let value = serde_json::to_value(&request).expect("request should serialize");
        assert_eq!(value["action"], serde_json::json!("project_overview"));
    }
}
