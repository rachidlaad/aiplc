use std::sync::Arc;

use rmcp::ErrorData as McpError;
use rmcp::handler::server::ServerHandler;
use rmcp::model::CallToolRequestParams;
use rmcp::model::CallToolResult;
use rmcp::model::Content;
use rmcp::model::ListToolsResult;
use rmcp::model::PaginatedRequestParams;
use rmcp::model::ServerCapabilities;
use rmcp::model::ServerInfo;
use rmcp::model::Tool;
use tokio::sync::Mutex;

use crate::advanced_types::CompareOnlineOfflineParams;
use crate::advanced_types::ConsistencyCheckParams;
use crate::advanced_types::CreateBlockCallParams;
use crate::advanced_types::CreateBlockParams;
use crate::advanced_types::CreatePlcTagParams;
use crate::advanced_types::CreateTagTableParams;
use crate::advanced_types::CreateUdtParams;
use crate::advanced_types::CreateWatchTableParams;
use crate::advanced_types::CrossReferenceParams;
use crate::advanced_types::EditBlockBodyParams;
use crate::advanced_types::EditDbMembersParams;
use crate::advanced_types::EditUdtParams;
use crate::advanced_types::ListDataTypesParams;
use crate::advanced_types::ListHmiObjectsParams;
use crate::advanced_types::ListNetworksParams;
use crate::advanced_types::ListSafetyObjectsParams;
use crate::advanced_types::ListTechnologyObjectsParams;
use crate::advanced_types::ListWatchTablesParams;
use crate::advanced_types::RunSimulationParams;
use crate::backend::PlcBackend;
use crate::integration_types::CreateHmiAlarmParams;
use crate::integration_types::CreateSafetyObjectParams;
use crate::integration_types::CreateTechnologyObjectParams;
use crate::integration_types::DownloadToDeviceParams;
use crate::integration_types::GoOnlineParams;
use crate::integration_types::WriteHardwareConfigParams;
use crate::integration_types::WriteNetworkConfigParams;
use crate::tooling::ALL_TOOL_NAMES;
use crate::tooling::TOOL_APPLY_EDIT;
use crate::tooling::TOOL_COMPARE_ONLINE_OFFLINE;
use crate::tooling::TOOL_COMPILE;
use crate::tooling::TOOL_CONNECT;
use crate::tooling::TOOL_CONSISTENCY_CHECK;
use crate::tooling::TOOL_CREATE_BLOCK;
use crate::tooling::TOOL_CREATE_BLOCK_CALL;
use crate::tooling::TOOL_CREATE_HMI_ALARM;
use crate::tooling::TOOL_CREATE_PLC_TAG;
use crate::tooling::TOOL_CREATE_SAFETY_OBJECT;
use crate::tooling::TOOL_CREATE_TAG_TABLE;
use crate::tooling::TOOL_CREATE_TECHNOLOGY_OBJECT;
use crate::tooling::TOOL_CREATE_UDT;
use crate::tooling::TOOL_CREATE_WATCH_TABLE;
use crate::tooling::TOOL_CROSS_REFERENCE;
use crate::tooling::TOOL_DOWNLOAD_TO_DEVICE;
use crate::tooling::TOOL_EDIT_BLOCK_BODY;
use crate::tooling::TOOL_EDIT_DB_MEMBERS;
use crate::tooling::TOOL_EDIT_UDT;
use crate::tooling::TOOL_EXPORT_OBJECT;
use crate::tooling::TOOL_GO_ONLINE;
use crate::tooling::TOOL_IMPORT_OBJECT;
use crate::tooling::TOOL_LIST_BLOCKS;
use crate::tooling::TOOL_LIST_DATA_TYPES;
use crate::tooling::TOOL_LIST_HMI_OBJECTS;
use crate::tooling::TOOL_LIST_NETWORKS;
use crate::tooling::TOOL_LIST_SAFETY_OBJECTS;
use crate::tooling::TOOL_LIST_TAG_TABLES;
use crate::tooling::TOOL_LIST_TECHNOLOGY_OBJECTS;
use crate::tooling::TOOL_LIST_WATCH_TABLES;
use crate::tooling::TOOL_OPEN_PROJECT;
use crate::tooling::TOOL_PROJECT_OVERVIEW;
use crate::tooling::TOOL_RENAME_OBJECT;
use crate::tooling::TOOL_RUN_SIMULATION;
use crate::tooling::TOOL_SET_BLOCK_HEADER;
use crate::tooling::TOOL_SET_PLC_TAG_PROPERTIES;
use crate::tooling::TOOL_WRITE_HARDWARE_CONFIG;
use crate::tooling::TOOL_WRITE_NETWORK_CONFIG;
use crate::tooling::tool_definitions_for;
use crate::types::ApplyEditParams;
use crate::types::BackendError;
use crate::types::CompileParams;
use crate::types::ConnectParams;
use crate::types::EditOperation;
use crate::types::ExportObjectParams;
use crate::types::ImportObjectParams;
use crate::types::ListBlocksParams;
use crate::types::ListTagTablesParams;
use crate::types::OpenProjectParams;
use crate::types::ProjectOverviewParams;
use crate::types::RenameObjectParams;
use crate::types::SetBlockHeaderParams;
use crate::types::SetPlcTagPropertiesParams;

pub struct PlcToolServer {
    backend: Arc<Mutex<Box<dyn PlcBackend>>>,
    supported_tool_names: &'static [&'static str],
    tools: Arc<Vec<Tool>>,
}

impl PlcToolServer {
    pub fn new(backend: Box<dyn PlcBackend>) -> Self {
        let supported_tool_names = backend.supported_tool_names();
        Self {
            backend: Arc::new(Mutex::new(backend)),
            supported_tool_names,
            tools: Arc::new(tool_definitions_for(supported_tool_names)),
        }
    }

    fn supports_tool(&self, tool_name: &str) -> bool {
        self.supported_tool_names.contains(&tool_name)
    }
}

impl ServerHandler for PlcToolServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_tool_list_changed()
                .build(),
            instructions: Some(
                "Prefer read tools first. Use object ids from inspection results for all mutations. Prefer tia_portal_rename_object, tia_portal_set_block_header, and tia_portal_set_plc_tag_properties over tia_portal_apply_edit when they fit the requested change. For tia_portal_apply_edit, operation must always be a structured object and never free text."
                    .to_string(),
            ),
            ..ServerInfo::default()
        }
    }

    fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: rmcp::service::RequestContext<rmcp::service::RoleServer>,
    ) -> impl std::future::Future<Output = Result<ListToolsResult, McpError>> + Send + '_ {
        let tools = self.tools.clone();
        async move {
            Ok(ListToolsResult {
                tools: (*tools).clone(),
                next_cursor: None,
                meta: None,
            })
        }
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: rmcp::service::RequestContext<rmcp::service::RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        if !ALL_TOOL_NAMES.contains(&request.name.as_ref()) {
            return Err(McpError::invalid_params(
                format!("unknown tool: {}", request.name),
                None,
            ));
        }

        if !self.supports_tool(request.name.as_ref()) {
            return error_result(BackendError::with_details(
                "unsupported_backend_tool",
                format!(
                    "{} is not supported by the active PLC backend",
                    request.name
                ),
                serde_json::json!({
                    "tool": request.name,
                    "supported_tools": self.supported_tool_names,
                }),
            ));
        }

        match request.name.as_ref() {
            TOOL_CONNECT => {
                let params: ConnectParams = parse_params(request.arguments)?;
                let mut backend = self.backend.lock().await;
                backend
                    .connect(params)
                    .await
                    .map_or_else(error_result, success_result)
            }
            TOOL_OPEN_PROJECT => {
                let params: OpenProjectParams = parse_params(request.arguments)?;
                let mut backend = self.backend.lock().await;
                backend
                    .open_project(params)
                    .await
                    .map_or_else(error_result, success_result)
            }
            TOOL_PROJECT_OVERVIEW => {
                let params: ProjectOverviewParams = parse_params(request.arguments)?;
                let mut backend = self.backend.lock().await;
                backend
                    .project_overview(params)
                    .await
                    .map_or_else(error_result, success_result)
            }
            TOOL_LIST_BLOCKS => {
                let params: ListBlocksParams = parse_params(request.arguments)?;
                let mut backend = self.backend.lock().await;
                backend
                    .list_blocks(params)
                    .await
                    .map_or_else(error_result, success_result)
            }
            TOOL_LIST_TAG_TABLES => {
                let params: ListTagTablesParams = parse_params(request.arguments)?;
                let mut backend = self.backend.lock().await;
                backend
                    .list_tag_tables(params)
                    .await
                    .map_or_else(error_result, success_result)
            }
            TOOL_LIST_DATA_TYPES => {
                let params: ListDataTypesParams = parse_params(request.arguments)?;
                let mut backend = self.backend.lock().await;
                backend
                    .list_data_types(params)
                    .await
                    .map_or_else(error_result, success_result)
            }
            TOOL_EXPORT_OBJECT => {
                let params: ExportObjectParams = parse_params(request.arguments)?;
                let mut backend = self.backend.lock().await;
                backend
                    .export_object(params)
                    .await
                    .map_or_else(error_result, success_result)
            }
            TOOL_IMPORT_OBJECT => {
                let params: ImportObjectParams = parse_params(request.arguments)?;
                let mut backend = self.backend.lock().await;
                backend
                    .import_object(params)
                    .await
                    .map_or_else(error_result, success_result)
            }
            TOOL_RENAME_OBJECT => {
                let params: RenameObjectParams = parse_params(request.arguments)?;
                let mut backend = self.backend.lock().await;
                backend
                    .apply_edit(ApplyEditParams {
                        object_id: params.object_id,
                        operation: EditOperation::RenameObject {
                            new_name: params.new_name,
                        },
                    })
                    .await
                    .map_or_else(error_result, success_result)
            }
            TOOL_SET_BLOCK_HEADER => {
                let params: SetBlockHeaderParams = parse_params(request.arguments)?;
                let object_id = params.object_id.clone();
                let operation = block_header_operation(params)?;
                let mut backend = self.backend.lock().await;
                backend
                    .apply_edit(ApplyEditParams {
                        object_id,
                        operation,
                    })
                    .await
                    .map_or_else(error_result, success_result)
            }
            TOOL_SET_PLC_TAG_PROPERTIES => {
                let params: SetPlcTagPropertiesParams = parse_params(request.arguments)?;
                let object_id = params.object_id.clone();
                let operation = plc_tag_properties_operation(params)?;
                let mut backend = self.backend.lock().await;
                backend
                    .apply_edit(ApplyEditParams {
                        object_id,
                        operation,
                    })
                    .await
                    .map_or_else(error_result, success_result)
            }
            TOOL_APPLY_EDIT => {
                let params: ApplyEditParams = parse_params(request.arguments)?;
                let mut backend = self.backend.lock().await;
                backend
                    .apply_edit(params)
                    .await
                    .map_or_else(error_result, success_result)
            }
            TOOL_CREATE_UDT => {
                let params: CreateUdtParams = parse_params(request.arguments)?;
                let mut backend = self.backend.lock().await;
                backend
                    .create_udt(params)
                    .await
                    .map_or_else(error_result, success_result)
            }
            TOOL_EDIT_UDT => {
                let params: EditUdtParams = parse_params(request.arguments)?;
                let mut backend = self.backend.lock().await;
                backend
                    .edit_udt(params)
                    .await
                    .map_or_else(error_result, success_result)
            }
            TOOL_CREATE_BLOCK => {
                let params: CreateBlockParams = parse_params(request.arguments)?;
                let mut backend = self.backend.lock().await;
                backend
                    .create_block(params)
                    .await
                    .map_or_else(error_result, success_result)
            }
            TOOL_EDIT_BLOCK_BODY => {
                let params: EditBlockBodyParams = parse_params(request.arguments)?;
                let mut backend = self.backend.lock().await;
                backend
                    .edit_block_body(params)
                    .await
                    .map_or_else(error_result, success_result)
            }
            TOOL_CREATE_BLOCK_CALL => {
                let params: CreateBlockCallParams = parse_params(request.arguments)?;
                let mut backend = self.backend.lock().await;
                backend
                    .create_block_call(params)
                    .await
                    .map_or_else(error_result, success_result)
            }
            TOOL_EDIT_DB_MEMBERS => {
                let params: EditDbMembersParams = parse_params(request.arguments)?;
                let mut backend = self.backend.lock().await;
                backend
                    .edit_db_members(params)
                    .await
                    .map_or_else(error_result, success_result)
            }
            TOOL_CREATE_PLC_TAG => {
                let params: CreatePlcTagParams = parse_params(request.arguments)?;
                let mut backend = self.backend.lock().await;
                backend
                    .create_plc_tag(params)
                    .await
                    .map_or_else(error_result, success_result)
            }
            TOOL_CREATE_TAG_TABLE => {
                let params: CreateTagTableParams = parse_params(request.arguments)?;
                let mut backend = self.backend.lock().await;
                backend
                    .create_tag_table(params)
                    .await
                    .map_or_else(error_result, success_result)
            }
            TOOL_LIST_TECHNOLOGY_OBJECTS => {
                let params: ListTechnologyObjectsParams = parse_params(request.arguments)?;
                let mut backend = self.backend.lock().await;
                backend
                    .list_technology_objects(params)
                    .await
                    .map_or_else(error_result, success_result)
            }
            TOOL_LIST_WATCH_TABLES => {
                let params: ListWatchTablesParams = parse_params(request.arguments)?;
                let mut backend = self.backend.lock().await;
                backend
                    .list_watch_tables(params)
                    .await
                    .map_or_else(error_result, success_result)
            }
            TOOL_CREATE_WATCH_TABLE => {
                let params: CreateWatchTableParams = parse_params(request.arguments)?;
                let mut backend = self.backend.lock().await;
                backend
                    .create_watch_table(params)
                    .await
                    .map_or_else(error_result, success_result)
            }
            TOOL_LIST_NETWORKS => {
                let params: ListNetworksParams = parse_params(request.arguments)?;
                let mut backend = self.backend.lock().await;
                backend
                    .list_networks(params)
                    .await
                    .map_or_else(error_result, success_result)
            }
            TOOL_LIST_HMI_OBJECTS => {
                let params: ListHmiObjectsParams = parse_params(request.arguments)?;
                let mut backend = self.backend.lock().await;
                backend
                    .list_hmi_objects(params)
                    .await
                    .map_or_else(error_result, success_result)
            }
            TOOL_LIST_SAFETY_OBJECTS => {
                let params: ListSafetyObjectsParams = parse_params(request.arguments)?;
                let mut backend = self.backend.lock().await;
                backend
                    .list_safety_objects(params)
                    .await
                    .map_or_else(error_result, success_result)
            }
            TOOL_WRITE_HARDWARE_CONFIG => {
                let params: WriteHardwareConfigParams = parse_params(request.arguments)?;
                let mut backend = self.backend.lock().await;
                backend
                    .write_hardware_config(params)
                    .await
                    .map_or_else(error_result, success_result)
            }
            TOOL_WRITE_NETWORK_CONFIG => {
                let params: WriteNetworkConfigParams = parse_params(request.arguments)?;
                let mut backend = self.backend.lock().await;
                backend
                    .write_network_config(params)
                    .await
                    .map_or_else(error_result, success_result)
            }
            TOOL_CREATE_HMI_ALARM => {
                let params: CreateHmiAlarmParams = parse_params(request.arguments)?;
                let mut backend = self.backend.lock().await;
                backend
                    .create_hmi_alarm(params)
                    .await
                    .map_or_else(error_result, success_result)
            }
            TOOL_CREATE_TECHNOLOGY_OBJECT => {
                let params: CreateTechnologyObjectParams = parse_params(request.arguments)?;
                let mut backend = self.backend.lock().await;
                backend
                    .create_technology_object(params)
                    .await
                    .map_or_else(error_result, success_result)
            }
            TOOL_CREATE_SAFETY_OBJECT => {
                let params: CreateSafetyObjectParams = parse_params(request.arguments)?;
                let mut backend = self.backend.lock().await;
                backend
                    .create_safety_object(params)
                    .await
                    .map_or_else(error_result, success_result)
            }
            TOOL_CROSS_REFERENCE => {
                let params: CrossReferenceParams = parse_params(request.arguments)?;
                let mut backend = self.backend.lock().await;
                backend
                    .cross_reference(params)
                    .await
                    .map_or_else(error_result, success_result)
            }
            TOOL_CONSISTENCY_CHECK => {
                let params: ConsistencyCheckParams = parse_params(request.arguments)?;
                let mut backend = self.backend.lock().await;
                backend
                    .consistency_check(params)
                    .await
                    .map_or_else(error_result, success_result)
            }
            TOOL_COMPARE_ONLINE_OFFLINE => {
                let params: CompareOnlineOfflineParams = parse_params(request.arguments)?;
                let mut backend = self.backend.lock().await;
                backend
                    .compare_online_offline(params)
                    .await
                    .map_or_else(error_result, success_result)
            }
            TOOL_RUN_SIMULATION => {
                let params: RunSimulationParams = parse_params(request.arguments)?;
                let mut backend = self.backend.lock().await;
                backend
                    .run_simulation(params)
                    .await
                    .map_or_else(error_result, success_result)
            }
            TOOL_GO_ONLINE => {
                let params: GoOnlineParams = parse_params(request.arguments)?;
                let mut backend = self.backend.lock().await;
                backend
                    .go_online(params)
                    .await
                    .map_or_else(error_result, success_result)
            }
            TOOL_DOWNLOAD_TO_DEVICE => {
                let params: DownloadToDeviceParams = parse_params(request.arguments)?;
                let mut backend = self.backend.lock().await;
                backend
                    .download_to_device(params)
                    .await
                    .map_or_else(error_result, success_result)
            }
            TOOL_COMPILE => {
                let params: CompileParams = parse_params(request.arguments)?;
                let mut backend = self.backend.lock().await;
                backend
                    .compile(params)
                    .await
                    .map_or_else(error_result, success_result)
            }
            other => Err(McpError::invalid_params(
                format!("unknown tool: {other}"),
                None,
            )),
        }
    }
}

fn parse_params<T>(
    arguments: Option<serde_json::Map<String, serde_json::Value>>,
) -> Result<T, McpError>
where
    T: serde::de::DeserializeOwned,
{
    serde_json::from_value(serde_json::Value::Object(
        arguments.unwrap_or_default().into_iter().collect(),
    ))
    .map_err(|err| McpError::invalid_params(err.to_string(), None))
}

fn success_result<T>(value: T) -> Result<CallToolResult, McpError>
where
    T: serde::Serialize,
{
    let structured_content = serde_json::to_value(&value)
        .map_err(|err| McpError::internal_error(err.to_string(), None))?;
    let text = serde_json::to_string_pretty(&structured_content)
        .map_err(|err| McpError::internal_error(err.to_string(), None))?;
    Ok(CallToolResult {
        content: vec![Content::text(text)],
        structured_content: Some(structured_content),
        is_error: Some(false),
        meta: None,
    })
}

fn error_result(err: BackendError) -> Result<CallToolResult, McpError> {
    let structured_content = serde_json::to_value(&err)
        .map_err(|serialize_err| McpError::internal_error(serialize_err.to_string(), None))?;
    Ok(CallToolResult {
        content: vec![Content::text(err.to_string())],
        structured_content: Some(structured_content),
        is_error: Some(true),
        meta: None,
    })
}

fn block_header_operation(params: SetBlockHeaderParams) -> Result<EditOperation, McpError> {
    let SetBlockHeaderParams {
        object_id: _,
        header_author,
        header_family,
        header_name,
        header_version,
    } = params;
    if header_author.is_none()
        && header_family.is_none()
        && header_name.is_none()
        && header_version.is_none()
    {
        return Err(McpError::invalid_params(
            "tia_portal_set_block_header requires at least one header_* field".to_string(),
            None,
        ));
    }
    Ok(EditOperation::SetBlockHeader {
        header_author,
        header_family,
        header_name,
        header_version,
    })
}

fn plc_tag_properties_operation(
    params: SetPlcTagPropertiesParams,
) -> Result<EditOperation, McpError> {
    let SetPlcTagPropertiesParams {
        object_id: _,
        name,
        data_type_name,
        logical_address,
        external_accessible,
        external_visible,
        external_writable,
        is_safety,
    } = params;
    if name.is_none()
        && data_type_name.is_none()
        && logical_address.is_none()
        && external_accessible.is_none()
        && external_visible.is_none()
        && external_writable.is_none()
        && is_safety.is_none()
    {
        return Err(McpError::invalid_params(
            "tia_portal_set_plc_tag_properties requires at least one editable field".to_string(),
            None,
        ));
    }
    Ok(EditOperation::SetPlcTagProperties {
        name,
        data_type_name,
        logical_address,
        external_accessible,
        external_visible,
        external_writable,
        is_safety,
    })
}
