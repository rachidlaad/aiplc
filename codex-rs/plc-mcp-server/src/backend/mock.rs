mod integration;

use std::collections::HashSet;
use std::path::Path;

use anyhow::Context;
use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value as JsonValue;
use serde_json::json;
use sha2::Digest;
use sha2::Sha256;

use crate::advanced_types::BlockAuthoringLanguage;
use crate::advanced_types::CompareOnlineOfflineParams;
use crate::advanced_types::CompareOnlineOfflineResult;
use crate::advanced_types::CompareOnlineOfflineScope;
use crate::advanced_types::ConsistencyCheckParams;
use crate::advanced_types::ConsistencyCheckResult;
use crate::advanced_types::ConsistencyCheckScope;
use crate::advanced_types::ConsistencyIssue;
use crate::advanced_types::CreateBlockCallParams;
use crate::advanced_types::CreateBlockParams;
use crate::advanced_types::CreatePlcTagParams;
use crate::advanced_types::CreateTagTableParams;
use crate::advanced_types::CreateUdtParams;
use crate::advanced_types::CreateWatchTableParams;
use crate::advanced_types::CrossReferenceHit;
use crate::advanced_types::CrossReferenceParams;
use crate::advanced_types::CrossReferenceResult;
use crate::advanced_types::DataTypeSummary;
use crate::advanced_types::DbMemberDefinition;
use crate::advanced_types::EditBlockBodyParams;
use crate::advanced_types::EditDbMembersParams;
use crate::advanced_types::EditUdtParams;
use crate::advanced_types::HmiObjectSummary;
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
use crate::advanced_types::NetworkSummary;
use crate::advanced_types::NewBlockKind;
use crate::advanced_types::OnlineDifference;
use crate::advanced_types::RunSimulationParams;
use crate::advanced_types::RunSimulationResult;
use crate::advanced_types::SafetyObjectSummary;
use crate::advanced_types::SimulationObservation;
use crate::advanced_types::TechnologyObjectSummary;
use crate::advanced_types::UdtMemberSummary;
use crate::advanced_types::WatchTableExpression;
use crate::advanced_types::WatchTableSummary;
use crate::backend::BackendOptions;
use crate::backend::PlcBackend;
use crate::integration_types::CreateHmiAlarmParams;
use crate::integration_types::CreateSafetyObjectParams;
use crate::integration_types::CreateTechnologyObjectParams;
use crate::integration_types::DownloadToDeviceParams;
use crate::integration_types::GoOnlineParams;
use crate::integration_types::WriteHardwareConfigParams;
use crate::integration_types::WriteNetworkConfigParams;
use crate::types::ApplyEditParams;
use crate::types::BackendError;
use crate::types::BlockSummary;
use crate::types::CompileParams;
use crate::types::CompileResultEnvelope;
use crate::types::CompileScope;
use crate::types::CompilerMessageSummary;
use crate::types::CompilerResultSummary;
use crate::types::ConnectParams;
use crate::types::ConnectResult;
use crate::types::DeviceItemSummary;
use crate::types::DeviceSummary;
use crate::types::EditOperation;
use crate::types::EngineeringObjectSummary;
use crate::types::ExportObjectParams;
use crate::types::ExportObjectResult;
use crate::types::ExportReadMode;
use crate::types::FieldChange;
use crate::types::ImportObjectParams;
use crate::types::ListBlocksParams;
use crate::types::ListBlocksResult;
use crate::types::ListTagTablesParams;
use crate::types::ListTagTablesResult;
use crate::types::MutationResult;
use crate::types::OpenProjectParams;
use crate::types::PlcSoftwareSummary;
use crate::types::PlcTagSummary;
use crate::types::PortalProcessSummary;
use crate::types::ProjectOverviewParams;
use crate::types::ProjectOverviewResult;
use crate::types::ProjectSummary;
use crate::types::SessionOrigin;
use crate::types::TagTableDetailLevel;
use crate::types::TagTableSummary;
use crate::types::TiaUiMode;
use crate::types::TouchedObject;
use crate::types::VerificationResult;
use crate::types::VerifiedField;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MockState {
    portal_version: String,
    project: MockProject,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MockProject {
    object: EngineeringObjectSummary,
    project_path: String,
    devices: Vec<MockDevice>,
    plc_software: Vec<MockPlcSoftware>,
    #[serde(default)]
    networks: Vec<NetworkSummary>,
    #[serde(default)]
    hmi_objects: Vec<MockHmiObject>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MockDevice {
    object: EngineeringObjectSummary,
    type_identifier: Option<String>,
    #[serde(default)]
    profinet_device_name: Option<String>,
    #[serde(default)]
    online_state: Option<String>,
    #[serde(default)]
    downloaded_object_ids: Vec<String>,
    #[serde(default)]
    last_download_mode: Option<String>,
    device_items: Vec<MockDeviceItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MockDeviceItem {
    object: EngineeringObjectSummary,
    classification: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MockPlcSoftware {
    summary: PlcSoftwareSummary,
    #[serde(default)]
    blocks: Vec<MockBlock>,
    #[serde(default)]
    tag_tables: Vec<MockTagTable>,
    #[serde(default)]
    data_types: Vec<MockDataType>,
    #[serde(default)]
    technology_objects: Vec<TechnologyObjectSummary>,
    #[serde(default)]
    watch_tables: Vec<WatchTableSummary>,
    #[serde(default)]
    safety_objects: Vec<SafetyObjectSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MockDataType {
    summary: DataTypeSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MockBlock {
    summary: BlockSummary,
    #[serde(default)]
    language: Option<BlockAuthoringLanguage>,
    #[serde(default)]
    comment: Option<String>,
    #[serde(default)]
    block_body: String,
    #[serde(default)]
    db_members: Vec<DbMemberDefinition>,
    #[serde(default)]
    calls: Vec<MockBlockCall>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MockBlockCall {
    callee_block_id: String,
    instance_db_name: Option<String>,
    comment: Option<String>,
    #[serde(default)]
    parameter_bindings: Vec<crate::advanced_types::BlockCallBinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MockTagTable {
    summary: TagTableSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MockHmiObject {
    summary: HmiObjectSummary,
    #[serde(default)]
    trigger_tag: Option<String>,
    #[serde(default)]
    message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MockBlockDocument {
    object_kind: String,
    name: String,
    block_type: Option<String>,
    header_author: Option<String>,
    header_family: Option<String>,
    header_name: Option<String>,
    header_version: Option<String>,
    language: Option<BlockAuthoringLanguage>,
    comment: Option<String>,
    #[serde(default)]
    db_members: Vec<DbMemberDefinition>,
    block_body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MockTagTableDocument {
    object_kind: String,
    name: String,
    tags: Vec<PlcTagSummary>,
}

pub struct MockBackend {
    state: MockState,
    connected: bool,
    current_ui_mode: Option<TiaUiMode>,
    next_process_id: u32,
}

impl MockBackend {
    pub async fn from_options(options: BackendOptions) -> Result<Self> {
        let state = if let Some(simulator_state_path) = options.simulator_state_path {
            let contents = tokio::fs::read_to_string(&simulator_state_path)
                .await
                .with_context(|| {
                    format!(
                        "failed to read simulator state {}",
                        simulator_state_path.display()
                    )
                })?;
            serde_json::from_str::<MockState>(&contents).with_context(|| {
                format!(
                    "failed to parse simulator state {}",
                    simulator_state_path.display()
                )
            })?
        } else {
            Self::default_state()
        };

        Ok(Self {
            state,
            connected: false,
            current_ui_mode: None,
            next_process_id: 4100,
        })
    }

    fn default_state() -> MockState {
        let project_object = EngineeringObjectSummary {
            object_id: "project/sample-packaging-line".to_string(),
            kind: "project".to_string(),
            name: "SamplePackagingLine".to_string(),
            path: "SamplePackagingLine".to_string(),
        };
        let device = MockDevice {
            object: EngineeringObjectSummary {
                object_id: "device/packaging-station-1".to_string(),
                kind: "device".to_string(),
                name: "PackagingStation_1".to_string(),
                path: "SamplePackagingLine/PackagingStation_1".to_string(),
            },
            type_identifier: Some("OrderNumber:6ES7 516-3AN02-0AB0/V2.9".to_string()),
            profinet_device_name: Some("packaging-station-1".to_string()),
            online_state: Some("offline".to_string()),
            downloaded_object_ids: Vec::new(),
            last_download_mode: None,
            device_items: vec![MockDeviceItem {
                object: EngineeringObjectSummary {
                    object_id: "device_item/plc_1".to_string(),
                    kind: "device_item".to_string(),
                    name: "PLC_1".to_string(),
                    path: "SamplePackagingLine/PackagingStation_1/PLC_1".to_string(),
                },
                classification: Some("plc".to_string()),
            }],
        };
        let plc_summary = PlcSoftwareSummary {
            object: EngineeringObjectSummary {
                object_id: "plc/software_main".to_string(),
                kind: "plc_software".to_string(),
                name: "PLC_1".to_string(),
                path: "SamplePackagingLine/PackagingStation_1/PLC_1".to_string(),
            },
            device_id: device.object.object_id.clone(),
            device_name: device.object.name.clone(),
            block_group_object_id: "block_group/software_main".to_string(),
            tag_table_group_object_id: "tag_group/software_main".to_string(),
        };
        let plc_path = plc_summary.object.path.clone();
        let blocks = vec![
            MockBlock {
                summary: BlockSummary {
                    object: EngineeringObjectSummary {
                        object_id: "block/ob1".to_string(),
                        kind: "code_block".to_string(),
                        name: "MainOB1".to_string(),
                        path: format!("{plc_path}/Program blocks/MainOB1"),
                    },
                    block_type: "OB".to_string(),
                    group_path: "Program blocks".to_string(),
                    number: Some(1),
                    header_author: Some("PLC Team".to_string()),
                    header_family: Some("Runtime".to_string()),
                    header_name: Some("OB1".to_string()),
                    header_version: Some("1.0.0".to_string()),
                },
                language: Some(BlockAuthoringLanguage::Scl),
                comment: Some("Main cyclic task".to_string()),
                block_body: "// Main cycle\nIF Start THEN\n    MotorRun := TRUE;\nEND_IF;"
                    .to_string(),
                db_members: Vec::new(),
                calls: Vec::new(),
            },
            MockBlock {
                summary: BlockSummary {
                    object: EngineeringObjectSummary {
                        object_id: "block/motor_fb".to_string(),
                        kind: "code_block".to_string(),
                        name: "MotorFB".to_string(),
                        path: format!("{plc_path}/Program blocks/MotorFB"),
                    },
                    block_type: "FB".to_string(),
                    group_path: "Program blocks".to_string(),
                    number: Some(20),
                    header_author: Some("PLC Team".to_string()),
                    header_family: Some("Drives".to_string()),
                    header_name: Some("Motor".to_string()),
                    header_version: Some("2.1.0".to_string()),
                },
                language: Some(BlockAuthoringLanguage::Scl),
                comment: Some("Motor sequencing FB".to_string()),
                block_body: "FUNCTION_BLOCK MotorFB\nVAR_INPUT\n    Start : Bool;\nEND_VAR\nEND_FUNCTION_BLOCK"
                    .to_string(),
                db_members: Vec::new(),
                calls: Vec::new(),
            },
            MockBlock {
                summary: BlockSummary {
                    object: EngineeringObjectSummary {
                        object_id: "block/conveyor_state_db".to_string(),
                        kind: "data_block".to_string(),
                        name: "ConveyorStateDB".to_string(),
                        path: format!("{plc_path}/Program blocks/ConveyorStateDB"),
                    },
                    block_type: "GlobalDB".to_string(),
                    group_path: "Program blocks".to_string(),
                    number: Some(100),
                    header_author: Some("PLC Team".to_string()),
                    header_family: Some("Runtime".to_string()),
                    header_name: Some("ConveyorState".to_string()),
                    header_version: Some("1.0.0".to_string()),
                },
                language: None,
                comment: Some("Persistent conveyor state".to_string()),
                block_body: String::new(),
                db_members: vec![
                    DbMemberDefinition {
                        name: "MotorState".to_string(),
                        data_type_name: "MotorStateUDT".to_string(),
                        comment: Some("Structured motor state".to_string()),
                        initial_value: None,
                    },
                    DbMemberDefinition {
                        name: "SpeedSetpoint".to_string(),
                        data_type_name: "Real".to_string(),
                        comment: Some("Requested speed".to_string()),
                        initial_value: Some("0.0".to_string()),
                    },
                ],
                calls: Vec::new(),
            },
        ];
        let tag_tables = vec![MockTagTable {
            summary: TagTableSummary {
                object: EngineeringObjectSummary {
                    object_id: "tag_table/default".to_string(),
                    kind: "plc_tag_table".to_string(),
                    name: "Default tag table".to_string(),
                    path: format!("{plc_path}/PLC tags/Default tag table"),
                },
                group_path: "PLC tags".to_string(),
                tags: Some(vec![
                    PlcTagSummary {
                        object: EngineeringObjectSummary {
                            object_id: "tag/start".to_string(),
                            kind: "plc_tag".to_string(),
                            name: "Start".to_string(),
                            path: format!("{plc_path}/PLC tags/Default tag table/Start"),
                        },
                        data_type_name: Some("Bool".to_string()),
                        logical_address: Some("%I0.0".to_string()),
                        external_accessible: Some(true),
                        external_visible: Some(true),
                        external_writable: Some(false),
                    },
                    PlcTagSummary {
                        object: EngineeringObjectSummary {
                            object_id: "tag/motor_run".to_string(),
                            kind: "plc_tag".to_string(),
                            name: "MotorRun".to_string(),
                            path: format!("{plc_path}/PLC tags/Default tag table/MotorRun"),
                        },
                        data_type_name: Some("Bool".to_string()),
                        logical_address: Some("%Q0.0".to_string()),
                        external_accessible: Some(true),
                        external_visible: Some(true),
                        external_writable: Some(false),
                    },
                ]),
            },
        }];
        let data_types = vec![MockDataType {
            summary: DataTypeSummary {
                object: EngineeringObjectSummary {
                    object_id: "udt/motor_state_udt".to_string(),
                    kind: "plc_data_type".to_string(),
                    name: "MotorStateUDT".to_string(),
                    path: format!("{plc_path}/PLC data types/MotorStateUDT"),
                },
                data_type_kind: "udt".to_string(),
                comment: Some("Standard motor state structure".to_string()),
                members: vec![
                    UdtMemberSummary {
                        name: "Running".to_string(),
                        data_type_name: "Bool".to_string(),
                        comment: Some("Motor running state".to_string()),
                        initial_value: Some("FALSE".to_string()),
                    },
                    UdtMemberSummary {
                        name: "Faulted".to_string(),
                        data_type_name: "Bool".to_string(),
                        comment: Some("Motor fault state".to_string()),
                        initial_value: Some("FALSE".to_string()),
                    },
                ],
            },
        }];
        let technology_objects = vec![TechnologyObjectSummary {
            object: EngineeringObjectSummary {
                object_id: "technology/axis_conveyor".to_string(),
                kind: "technology_object".to_string(),
                name: "Axis_Conveyor".to_string(),
                path: format!("{plc_path}/Technology objects/Axis_Conveyor"),
            },
            technology_type: "motion_axis".to_string(),
            bound_axis: Some("ConveyorAxis".to_string()),
        }];
        let watch_tables = vec![WatchTableSummary {
            object: EngineeringObjectSummary {
                object_id: "watch/commissioning".to_string(),
                kind: "watch_table".to_string(),
                name: "CommissioningWatch".to_string(),
                path: format!("{plc_path}/Watch tables/CommissioningWatch"),
            },
            expressions: vec![
                WatchTableExpression {
                    expression: "Start".to_string(),
                    comment: Some("Start command".to_string()),
                },
                WatchTableExpression {
                    expression: "MotorRun".to_string(),
                    comment: Some("Motor run output".to_string()),
                },
            ],
        }];
        let safety_objects = vec![SafetyObjectSummary {
            object: EngineeringObjectSummary {
                object_id: "safety/emergency_stop".to_string(),
                kind: "safety_object".to_string(),
                name: "EmergencyStopGroup".to_string(),
                path: format!("{plc_path}/Safety administration/EmergencyStopGroup"),
            },
            safety_type: "safety_group".to_string(),
        }];
        let hmi_objects = vec![MockHmiObject {
            summary: HmiObjectSummary {
                object: EngineeringObjectSummary {
                    object_id: "hmi/line_panel_1".to_string(),
                    kind: "hmi_object".to_string(),
                    name: "LinePanel_1".to_string(),
                    path: "SamplePackagingLine/LinePanel_1".to_string(),
                },
                hmi_type: "comfort_panel".to_string(),
            },
            trigger_tag: None,
            message: None,
        }];
        let networks = vec![NetworkSummary {
            object: EngineeringObjectSummary {
                object_id: "network/profinet_line_1".to_string(),
                kind: "network".to_string(),
                name: "PROFINET_Line_1".to_string(),
                path: "SamplePackagingLine/Networks/PROFINET_Line_1".to_string(),
            },
            connected_object_ids: vec![
                device.object.object_id.clone(),
                plc_summary.object.object_id.clone(),
                "hmi/line_panel_1".to_string(),
            ],
        }];

        MockState {
            portal_version: "V21".to_string(),
            project: MockProject {
                object: project_object,
                project_path: r"C:\Samples\SamplePackagingLine\SamplePackagingLine.ap21"
                    .to_string(),
                devices: vec![device],
                plc_software: vec![MockPlcSoftware {
                    summary: plc_summary,
                    blocks,
                    tag_tables,
                    data_types,
                    technology_objects,
                    watch_tables,
                    safety_objects,
                }],
                networks,
                hmi_objects,
            },
        }
    }

    fn require_connected(&self) -> std::result::Result<(), BackendError> {
        if self.connected {
            Ok(())
        } else {
            Err(BackendError::new(
                "not_connected",
                "connect to TIA Portal before using this tool",
            ))
        }
    }

    fn project_overview_result(&self) -> ProjectOverviewResult {
        ProjectOverviewResult {
            project: ProjectSummary {
                object: self.state.project.object.clone(),
                project_path: self.state.project.project_path.clone(),
                portal_version: Some(self.state.portal_version.clone()),
            },
            devices: self
                .state
                .project
                .devices
                .iter()
                .map(|device| DeviceSummary {
                    object: device.object.clone(),
                    type_identifier: device.type_identifier.clone(),
                    device_items: device
                        .device_items
                        .iter()
                        .map(|item| DeviceItemSummary {
                            object: item.object.clone(),
                            classification: item.classification.clone(),
                        })
                        .collect(),
                })
                .collect(),
            plc_software: self
                .state
                .project
                .plc_software
                .iter()
                .map(|plc| plc.summary.clone())
                .collect(),
        }
    }

    fn primitive_data_types() -> &'static [&'static str] {
        &[
            "Bool", "Byte", "Word", "DWord", "Int", "UInt", "DInt", "UDInt", "Real", "LReal",
            "Time", "String",
        ]
    }

    fn plc_index(&self, plc_software_id: &str) -> std::result::Result<usize, BackendError> {
        self.state
            .project
            .plc_software
            .iter()
            .position(|plc| plc.summary.object.object_id == plc_software_id)
            .ok_or_else(|| {
                BackendError::with_details(
                    "object_not_found",
                    "unknown PLC software object id",
                    json!({ "plc_software_id": plc_software_id }),
                )
            })
    }

    fn block_location(&self, object_id: &str) -> std::result::Result<(usize, usize), BackendError> {
        self.state
            .project
            .plc_software
            .iter()
            .enumerate()
            .find_map(|(plc_index, plc)| {
                plc.blocks
                    .iter()
                    .position(|block| block.summary.object.object_id == object_id)
                    .map(|block_index| (plc_index, block_index))
            })
            .ok_or_else(|| {
                BackendError::with_details(
                    "object_not_found",
                    "unknown block object id",
                    json!({ "object_id": object_id }),
                )
            })
    }

    fn tag_table_location(
        &self,
        object_id: &str,
    ) -> std::result::Result<(usize, usize), BackendError> {
        self.state
            .project
            .plc_software
            .iter()
            .enumerate()
            .find_map(|(plc_index, plc)| {
                plc.tag_tables
                    .iter()
                    .position(|table| table.summary.object.object_id == object_id)
                    .map(|table_index| (plc_index, table_index))
            })
            .ok_or_else(|| {
                BackendError::with_details(
                    "object_not_found",
                    "unknown PLC tag table object id",
                    json!({ "object_id": object_id }),
                )
            })
    }

    fn tag_location(
        &self,
        object_id: &str,
    ) -> std::result::Result<(usize, usize, usize), BackendError> {
        self.state
            .project
            .plc_software
            .iter()
            .enumerate()
            .find_map(|(plc_index, plc)| {
                plc.tag_tables
                    .iter()
                    .enumerate()
                    .find_map(|(table_index, table)| {
                        table.summary.tags.as_ref().and_then(|tags| {
                            tags.iter()
                                .position(|tag| tag.object.object_id == object_id)
                                .map(|tag_index| (plc_index, table_index, tag_index))
                        })
                    })
            })
            .ok_or_else(|| {
                BackendError::with_details(
                    "object_not_found",
                    "unknown PLC tag object id",
                    json!({ "object_id": object_id }),
                )
            })
    }

    fn data_type_location(
        &self,
        object_id: &str,
    ) -> std::result::Result<(usize, usize), BackendError> {
        self.state
            .project
            .plc_software
            .iter()
            .enumerate()
            .find_map(|(plc_index, plc)| {
                plc.data_types
                    .iter()
                    .position(|data_type| data_type.summary.object.object_id == object_id)
                    .map(|data_type_index| (plc_index, data_type_index))
            })
            .ok_or_else(|| {
                BackendError::with_details(
                    "object_not_found",
                    "unknown PLC data type object id",
                    json!({ "object_id": object_id }),
                )
            })
    }

    fn find_group_plc_index(&self, object_id: &str) -> std::result::Result<usize, BackendError> {
        self.state
            .project
            .plc_software
            .iter()
            .position(|plc| {
                plc.summary.block_group_object_id == object_id
                    || plc.summary.tag_table_group_object_id == object_id
            })
            .ok_or_else(|| {
                BackendError::with_details(
                    "object_not_found",
                    "unknown import target group",
                    json!({ "target_group_object_id": object_id }),
                )
            })
    }

    fn ensure_unique_name(
        mut existing_names: impl Iterator<Item = String>,
        name: &str,
        resource_kind: &str,
    ) -> std::result::Result<(), BackendError> {
        if existing_names.any(|existing| existing == name) {
            Err(BackendError::with_details(
                "object_already_exists",
                "an engineering object with the requested name already exists in the target scope",
                json!({ "name": name, "resource_kind": resource_kind }),
            ))
        } else {
            Ok(())
        }
    }

    fn slugify(name: &str) -> String {
        let mut slug = String::with_capacity(name.len());
        for ch in name.chars() {
            if ch.is_ascii_alphanumeric() {
                slug.push(ch.to_ascii_lowercase());
            } else if ch == '_' || ch == '-' {
                slug.push(ch);
            } else if !slug.ends_with('_') {
                slug.push('_');
            }
        }
        slug.trim_matches('_').to_string()
    }

    fn export_block_document(block: &MockBlock) -> MockBlockDocument {
        MockBlockDocument {
            object_kind: "block".to_string(),
            name: block.summary.object.name.clone(),
            block_type: Some(block.summary.block_type.clone()),
            header_author: block.summary.header_author.clone(),
            header_family: block.summary.header_family.clone(),
            header_name: block.summary.header_name.clone(),
            header_version: block.summary.header_version.clone(),
            language: block.language.clone(),
            comment: block.comment.clone(),
            db_members: block.db_members.clone(),
            block_body: block.block_body.clone(),
        }
    }

    fn export_tag_table_document(table: &MockTagTable) -> MockTagTableDocument {
        MockTagTableDocument {
            object_kind: "tag_table".to_string(),
            name: table.summary.object.name.clone(),
            tags: table.summary.tags.clone().unwrap_or_default(),
        }
    }

    fn write_export(
        destination_path: &Path,
        contents: &str,
    ) -> std::result::Result<String, BackendError> {
        std::fs::create_dir_all(destination_path.parent().unwrap_or_else(|| Path::new(".")))
            .map_err(|err| {
                BackendError::with_details(
                    "io_error",
                    "failed to create export directory",
                    json!({ "error": err.to_string(), "path": destination_path }),
                )
            })?;
        std::fs::write(destination_path, contents).map_err(|err| {
            BackendError::with_details(
                "io_error",
                "failed to write export file",
                json!({ "error": err.to_string(), "path": destination_path }),
            )
        })?;
        Ok(hash_text(contents))
    }

    fn object_summary(&self, object_id: &str) -> Option<EngineeringObjectSummary> {
        if self.state.project.object.object_id == object_id {
            return Some(self.state.project.object.clone());
        }
        for device in &self.state.project.devices {
            if device.object.object_id == object_id {
                return Some(device.object.clone());
            }
            for item in &device.device_items {
                if item.object.object_id == object_id {
                    return Some(item.object.clone());
                }
            }
        }
        for plc in &self.state.project.plc_software {
            if plc.summary.object.object_id == object_id {
                return Some(plc.summary.object.clone());
            }
            for block in &plc.blocks {
                if block.summary.object.object_id == object_id {
                    return Some(block.summary.object.clone());
                }
            }
            for table in &plc.tag_tables {
                if table.summary.object.object_id == object_id {
                    return Some(table.summary.object.clone());
                }
                if let Some(tags) = &table.summary.tags {
                    for tag in tags {
                        if tag.object.object_id == object_id {
                            return Some(tag.object.clone());
                        }
                    }
                }
            }
            for data_type in &plc.data_types {
                if data_type.summary.object.object_id == object_id {
                    return Some(data_type.summary.object.clone());
                }
            }
            for technology_object in &plc.technology_objects {
                if technology_object.object.object_id == object_id {
                    return Some(technology_object.object.clone());
                }
            }
            for watch_table in &plc.watch_tables {
                if watch_table.object.object_id == object_id {
                    return Some(watch_table.object.clone());
                }
            }
            for safety_object in &plc.safety_objects {
                if safety_object.object.object_id == object_id {
                    return Some(safety_object.object.clone());
                }
            }
        }
        for network in &self.state.project.networks {
            if network.object.object_id == object_id {
                return Some(network.object.clone());
            }
        }
        for hmi in &self.state.project.hmi_objects {
            if hmi.summary.object.object_id == object_id {
                return Some(hmi.summary.object.clone());
            }
        }
        None
    }

    fn resolve_scope_summary(
        &self,
        scope: &ConsistencyCheckScope,
    ) -> std::result::Result<Option<EngineeringObjectSummary>, BackendError> {
        match scope {
            ConsistencyCheckScope::CurrentProject => Ok(Some(self.state.project.object.clone())),
            ConsistencyCheckScope::PlcSoftware { plc_software_id } => Ok(Some(
                self.state.project.plc_software[self.plc_index(plc_software_id)?]
                    .summary
                    .object
                    .clone(),
            )),
            ConsistencyCheckScope::Object { object_id } => {
                Ok(Some(self.object_summary(object_id).ok_or_else(|| {
                    BackendError::with_details(
                        "object_not_found",
                        "unknown engineering object id",
                        json!({ "object_id": object_id }),
                    )
                })?))
            }
        }
    }

    fn resolve_compare_scope_summary(
        &self,
        scope: &CompareOnlineOfflineScope,
    ) -> std::result::Result<Option<EngineeringObjectSummary>, BackendError> {
        match scope {
            CompareOnlineOfflineScope::CurrentProject => {
                Ok(Some(self.state.project.object.clone()))
            }
            CompareOnlineOfflineScope::PlcSoftware { plc_software_id } => Ok(Some(
                self.state.project.plc_software[self.plc_index(plc_software_id)?]
                    .summary
                    .object
                    .clone(),
            )),
            CompareOnlineOfflineScope::Object { object_id } => {
                Ok(Some(self.object_summary(object_id).ok_or_else(|| {
                    BackendError::with_details(
                        "object_not_found",
                        "unknown engineering object id",
                        json!({ "object_id": object_id }),
                    )
                })?))
            }
        }
    }

    fn data_type_names(plc: &MockPlcSoftware) -> HashSet<String> {
        let mut names = Self::primitive_data_types()
            .iter()
            .map(ToString::to_string)
            .collect::<HashSet<_>>();
        names.extend(
            plc.data_types
                .iter()
                .map(|data_type| data_type.summary.object.name.clone()),
        );
        names.extend(
            plc.blocks
                .iter()
                .filter(|block| block.summary.block_type == "FB")
                .map(|block| block.summary.object.name.clone()),
        );
        names
    }

    fn block_is_db(block: &MockBlock) -> bool {
        matches!(block.summary.block_type.as_str(), "GlobalDB" | "InstanceDB")
    }

    fn validate_header_family(block: &MockBlock) -> Option<ConsistencyIssue> {
        let header_family = block.summary.header_family.as_deref()?;
        if header_family
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
        {
            None
        } else {
            Some(ConsistencyIssue {
                severity: "error".to_string(),
                code: "INVALID_BLOCK_HEADER_FAMILY".to_string(),
                message: "Block header_family contains invalid characters.".to_string(),
                object: Some(block.summary.object.clone()),
            })
        }
    }

    fn consistency_issues_for_plc(plc: &MockPlcSoftware) -> Vec<ConsistencyIssue> {
        let known_types = Self::data_type_names(plc);
        let mut issues = Vec::new();
        for block in &plc.blocks {
            if block.block_body.contains("COMPILE_ERROR") {
                issues.push(ConsistencyIssue {
                    severity: "error".to_string(),
                    code: "BLOCK_BODY_COMPILE_SENTINEL".to_string(),
                    message: "Block body contains COMPILE_ERROR sentinel.".to_string(),
                    object: Some(block.summary.object.clone()),
                });
            }
            if let Some(issue) = Self::validate_header_family(block) {
                issues.push(issue);
            }
            for member in &block.db_members {
                if !known_types.contains(&member.data_type_name) {
                    issues.push(ConsistencyIssue {
                        severity: "error".to_string(),
                        code: "UNKNOWN_DB_MEMBER_TYPE".to_string(),
                        message: format!(
                            "DB member {} references unknown data type {}.",
                            member.name, member.data_type_name
                        ),
                        object: Some(block.summary.object.clone()),
                    });
                }
            }
            for call in &block.calls {
                if !plc
                    .blocks
                    .iter()
                    .any(|candidate| candidate.summary.object.object_id == call.callee_block_id)
                {
                    issues.push(ConsistencyIssue {
                        severity: "error".to_string(),
                        code: "UNKNOWN_CALLEE_BLOCK".to_string(),
                        message: "Recorded block call references a missing callee block."
                            .to_string(),
                        object: Some(block.summary.object.clone()),
                    });
                }
            }
        }
        for table in &plc.tag_tables {
            if let Some(tags) = &table.summary.tags {
                for tag in tags {
                    if let Some(data_type_name) = &tag.data_type_name
                        && !known_types.contains(data_type_name)
                    {
                        issues.push(ConsistencyIssue {
                            severity: "error".to_string(),
                            code: "UNKNOWN_TAG_DATA_TYPE".to_string(),
                            message: format!(
                                "PLC tag {} references unknown data type {}.",
                                tag.object.name, data_type_name
                            ),
                            object: Some(tag.object.clone()),
                        });
                    }
                }
            }
        }
        issues
    }

    fn generate_block_body(
        kind: &NewBlockKind,
        name: &str,
        language: Option<&BlockAuthoringLanguage>,
    ) -> String {
        match kind {
            NewBlockKind::Ob => format!("// {name}\nIF TRUE THEN\nEND_IF;"),
            NewBlockKind::Fb => format!(
                "FUNCTION_BLOCK {name}\nVAR_INPUT\nEND_VAR\n// language: {}\nEND_FUNCTION_BLOCK",
                match language {
                    Some(BlockAuthoringLanguage::Lad) => "lad",
                    Some(BlockAuthoringLanguage::Fbd) => "fbd",
                    _ => "scl",
                }
            ),
            NewBlockKind::Fc => format!("FUNCTION {name} : Void\nEND_FUNCTION"),
            NewBlockKind::GlobalDb | NewBlockKind::InstanceDb => String::new(),
        }
    }

    fn next_block_number(plc: &MockPlcSoftware, kind: &NewBlockKind) -> i32 {
        let start = match kind {
            NewBlockKind::Ob => 100,
            NewBlockKind::Fb => 10,
            NewBlockKind::Fc => 30,
            NewBlockKind::GlobalDb => 200,
            NewBlockKind::InstanceDb => 300,
        };
        plc.blocks
            .iter()
            .filter_map(|block| block.summary.number)
            .max()
            .unwrap_or(start - 1)
            + 1
    }

    fn block_type_for_kind(kind: &NewBlockKind) -> &'static str {
        match kind {
            NewBlockKind::Ob => "OB",
            NewBlockKind::Fb => "FB",
            NewBlockKind::Fc => "FC",
            NewBlockKind::GlobalDb => "GlobalDB",
            NewBlockKind::InstanceDb => "InstanceDB",
        }
    }
}

fn hash_text(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn verification_from_fields(
    strategy: &str,
    checked_fields: Vec<VerifiedField>,
    exported_sha256: Option<String>,
) -> VerificationResult {
    VerificationResult {
        verified: checked_fields
            .iter()
            .all(|field| field.expected == field.actual),
        strategy: strategy.to_string(),
        checked_fields,
        exported_sha256,
    }
}

#[async_trait]
impl PlcBackend for MockBackend {
    fn supported_tool_names(&self) -> &'static [&'static str] {
        crate::tooling::ALL_TOOL_NAMES
    }

    async fn connect(
        &mut self,
        params: ConnectParams,
    ) -> std::result::Result<ConnectResult, BackendError> {
        self.connected = true;
        self.current_ui_mode = Some(params.ui_mode.unwrap_or_default());
        let process = PortalProcessSummary {
            process_id: self.next_process_id,
            mode: Some(format!("{:?}", self.current_ui_mode.unwrap_or_default())),
            project_path: Some(self.state.project.project_path.clone()),
            executable_path: Some(
                r"C:\Program Files\Siemens\Automation\Portal V21\Bin\Siemens.Automation.Portal.exe"
                    .to_string(),
            ),
        };

        Ok(ConnectResult {
            backend: "simulator".to_string(),
            portal_version: Some(self.state.portal_version.clone()),
            origin: if params.connection_mode.unwrap_or_default()
                == crate::types::PortalConnectionMode::Launch
            {
                SessionOrigin::Launched
            } else {
                SessionOrigin::Attached
            },
            process_id: Some(self.next_process_id),
            ui_mode: self.current_ui_mode,
            project_open: true,
            processes: vec![process],
        })
    }

    async fn open_project(
        &mut self,
        params: OpenProjectParams,
    ) -> std::result::Result<ProjectOverviewResult, BackendError> {
        self.require_connected()?;
        self.state.project.project_path = params.project_path;
        Ok(self.project_overview_result())
    }

    async fn project_overview(
        &mut self,
        _params: ProjectOverviewParams,
    ) -> std::result::Result<ProjectOverviewResult, BackendError> {
        self.require_connected()?;
        Ok(self.project_overview_result())
    }

    async fn list_blocks(
        &mut self,
        params: ListBlocksParams,
    ) -> std::result::Result<ListBlocksResult, BackendError> {
        self.require_connected()?;
        let plc = &self.state.project.plc_software[self.plc_index(&params.plc_software_id)?];
        Ok(ListBlocksResult {
            plc_software_id: params.plc_software_id,
            blocks: plc
                .blocks
                .iter()
                .map(|block| block.summary.clone())
                .collect(),
        })
    }

    async fn list_tag_tables(
        &mut self,
        params: ListTagTablesParams,
    ) -> std::result::Result<ListTagTablesResult, BackendError> {
        self.require_connected()?;
        let plc = &self.state.project.plc_software[self.plc_index(&params.plc_software_id)?];
        let include_tags =
            params.detail_level.unwrap_or_default() == TagTableDetailLevel::IncludeTags;
        Ok(ListTagTablesResult {
            plc_software_id: params.plc_software_id,
            tag_tables: plc
                .tag_tables
                .iter()
                .map(|table| {
                    let mut summary = table.summary.clone();
                    if !include_tags {
                        summary.tags = None;
                    }
                    summary
                })
                .collect(),
        })
    }

    async fn list_data_types(
        &mut self,
        params: ListDataTypesParams,
    ) -> std::result::Result<ListDataTypesResult, BackendError> {
        self.require_connected()?;
        let plc = &self.state.project.plc_software[self.plc_index(&params.plc_software_id)?];
        Ok(ListDataTypesResult {
            plc_software_id: params.plc_software_id,
            data_types: plc
                .data_types
                .iter()
                .map(|data_type| data_type.summary.clone())
                .collect(),
        })
    }

    async fn export_object(
        &mut self,
        params: ExportObjectParams,
    ) -> std::result::Result<ExportObjectResult, BackendError> {
        self.require_connected()?;
        let read_mode = params.read_mode.unwrap_or_default();
        if let Ok((plc_index, block_index)) = self.block_location(&params.object_id) {
            let block = &self.state.project.plc_software[plc_index].blocks[block_index];
            let destination_path = params.destination_path.unwrap_or_else(|| {
                format!(
                    "{}.mock.json",
                    block.summary.object.name.replace(' ', "_").to_lowercase()
                )
            });
            let document = Self::export_block_document(block);
            let content_text = serde_json::to_string_pretty(&document).map_err(|err| {
                BackendError::with_details(
                    "serialization_error",
                    "failed to serialize mock block export",
                    json!({ "error": err.to_string() }),
                )
            })?;
            let content_sha256 = Self::write_export(Path::new(&destination_path), &content_text)?;
            let verification_sha256 = content_sha256.clone();
            return Ok(ExportObjectResult {
                object: block.summary.object.clone(),
                export_path: destination_path,
                content_sha256,
                content_text: if read_mode == ExportReadMode::IncludeText {
                    Some(content_text)
                } else {
                    None
                },
                verification: verification_from_fields(
                    "export_round_trip",
                    Vec::new(),
                    Some(verification_sha256),
                ),
            });
        }

        if let Ok((plc_index, table_index)) = self.tag_table_location(&params.object_id) {
            let table = &self.state.project.plc_software[plc_index].tag_tables[table_index];
            let destination_path = params.destination_path.unwrap_or_else(|| {
                format!(
                    "{}.mock.json",
                    table.summary.object.name.replace(' ', "_").to_lowercase()
                )
            });
            let document = Self::export_tag_table_document(table);
            let content_text = serde_json::to_string_pretty(&document).map_err(|err| {
                BackendError::with_details(
                    "serialization_error",
                    "failed to serialize mock tag-table export",
                    json!({ "error": err.to_string() }),
                )
            })?;
            let content_sha256 = Self::write_export(Path::new(&destination_path), &content_text)?;
            let verification_sha256 = content_sha256.clone();
            return Ok(ExportObjectResult {
                object: table.summary.object.clone(),
                export_path: destination_path,
                content_sha256,
                content_text: if read_mode == ExportReadMode::IncludeText {
                    Some(content_text)
                } else {
                    None
                },
                verification: verification_from_fields(
                    "export_round_trip",
                    Vec::new(),
                    Some(verification_sha256),
                ),
            });
        }

        Err(BackendError::with_details(
            "unsupported_export",
            "simulator backend can only export blocks and PLC tag tables",
            json!({ "object_id": params.object_id }),
        ))
    }

    async fn import_object(
        &mut self,
        params: ImportObjectParams,
    ) -> std::result::Result<MutationResult, BackendError> {
        self.require_connected()?;
        let plc_index = self.find_group_plc_index(&params.target_group_object_id)?;
        let source_text = std::fs::read_to_string(&params.source_file_path).map_err(|err| {
            BackendError::with_details(
                "io_error",
                "failed to read import file",
                json!({ "error": err.to_string(), "path": params.source_file_path }),
            )
        })?;

        if self.state.project.plc_software[plc_index]
            .summary
            .block_group_object_id
            == params.target_group_object_id
        {
            let document =
                serde_json::from_str::<MockBlockDocument>(&source_text).map_err(|err| {
                    BackendError::with_details(
                        "parse_error",
                        "failed to parse mock block document",
                        json!({ "error": err.to_string(), "path": params.source_file_path }),
                    )
                })?;
            let plc = &mut self.state.project.plc_software[plc_index];
            if let Some(block_index) = plc
                .blocks
                .iter()
                .position(|block| block.summary.object.name == document.name)
            {
                let block = &mut plc.blocks[block_index];
                let mut changes = Vec::new();
                let before_author = json!(block.summary.header_author.clone());
                let after_author = json!(document.header_author.clone());
                if before_author != after_author {
                    block.summary.header_author = document.header_author.clone();
                    changes.push(FieldChange {
                        field: "header_author".to_string(),
                        before: before_author,
                        after: after_author,
                    });
                }
                if block.block_body != document.block_body {
                    changes.push(FieldChange {
                        field: "block_body".to_string(),
                        before: JsonValue::String(block.block_body.clone()),
                        after: JsonValue::String(document.block_body.clone()),
                    });
                    block.block_body = document.block_body;
                }
                if block.language != document.language {
                    changes.push(FieldChange {
                        field: "language".to_string(),
                        before: json!(block.language),
                        after: json!(document.language),
                    });
                    block.language = document.language;
                }
                if block.comment != document.comment {
                    changes.push(FieldChange {
                        field: "comment".to_string(),
                        before: json!(block.comment.clone()),
                        after: json!(document.comment.clone()),
                    });
                    block.comment = document.comment;
                }
                if block.db_members != document.db_members {
                    changes.push(FieldChange {
                        field: "db_members".to_string(),
                        before: json!(block.db_members.clone()),
                        after: json!(document.db_members.clone()),
                    });
                    block.db_members = document.db_members;
                }
                let exported_sha256 = hash_text(
                    &serde_json::to_string(&Self::export_block_document(block)).unwrap_or_default(),
                );
                return Ok(MutationResult {
                    touched_objects: vec![TouchedObject {
                        object: block.summary.object.clone(),
                        changes,
                    }],
                    verification: verification_from_fields(
                        "post_import_export",
                        Vec::new(),
                        Some(exported_sha256),
                    ),
                });
            }

            let block_kind = match document.block_type.as_deref() {
                Some("OB") => NewBlockKind::Ob,
                Some("FC") => NewBlockKind::Fc,
                Some("GlobalDB") => NewBlockKind::GlobalDb,
                Some("InstanceDB") => NewBlockKind::InstanceDb,
                _ => NewBlockKind::Fb,
            };
            let object_id = format!("block/{}", Self::slugify(&document.name));
            let created_block = MockBlock {
                summary: BlockSummary {
                    object: EngineeringObjectSummary {
                        object_id,
                        kind: if matches!(
                            block_kind,
                            NewBlockKind::GlobalDb | NewBlockKind::InstanceDb
                        ) {
                            "data_block".to_string()
                        } else {
                            "code_block".to_string()
                        },
                        name: document.name.clone(),
                        path: format!(
                            "{}/Program blocks/{}",
                            plc.summary.object.path, document.name
                        ),
                    },
                    block_type: Self::block_type_for_kind(&block_kind).to_string(),
                    group_path: "Program blocks".to_string(),
                    number: Some(Self::next_block_number(plc, &block_kind)),
                    header_author: document.header_author.clone(),
                    header_family: document.header_family.clone(),
                    header_name: document.header_name.clone(),
                    header_version: document.header_version.clone(),
                },
                language: document.language.clone(),
                comment: document.comment.clone(),
                block_body: document.block_body.clone(),
                db_members: document.db_members,
                calls: Vec::new(),
            };
            let exported_sha256 = hash_text(
                &serde_json::to_string(&Self::export_block_document(&created_block))
                    .unwrap_or_default(),
            );
            let object = created_block.summary.object.clone();
            plc.blocks.push(created_block);
            return Ok(MutationResult {
                touched_objects: vec![TouchedObject {
                    object,
                    changes: vec![FieldChange {
                        field: "created".to_string(),
                        before: JsonValue::Null,
                        after: json!(true),
                    }],
                }],
                verification: verification_from_fields(
                    "post_import_export",
                    Vec::new(),
                    Some(exported_sha256),
                ),
            });
        }

        if self.state.project.plc_software[plc_index]
            .summary
            .tag_table_group_object_id
            == params.target_group_object_id
        {
            let document =
                serde_json::from_str::<MockTagTableDocument>(&source_text).map_err(|err| {
                    BackendError::with_details(
                        "parse_error",
                        "failed to parse mock PLC tag-table document",
                        json!({ "error": err.to_string(), "path": params.source_file_path }),
                    )
                })?;
            let plc = &mut self.state.project.plc_software[plc_index];
            let table_index = plc
                .tag_tables
                .iter()
                .position(|table| table.summary.object.name == document.name)
                .or_else(|| (!plc.tag_tables.is_empty()).then_some(0))
                .ok_or_else(|| {
                    BackendError::new("object_not_found", "default tag table not found")
                })?;
            let table = &mut plc.tag_tables[table_index];
            let before = json!(table.summary.tags.clone());
            table.summary.tags = Some(document.tags);
            let after = json!(table.summary.tags.clone());
            let exported_sha256 = hash_text(
                &serde_json::to_string(&Self::export_tag_table_document(table)).unwrap_or_default(),
            );
            return Ok(MutationResult {
                touched_objects: vec![TouchedObject {
                    object: table.summary.object.clone(),
                    changes: vec![FieldChange {
                        field: "tags".to_string(),
                        before,
                        after,
                    }],
                }],
                verification: verification_from_fields(
                    "post_import_export",
                    Vec::new(),
                    Some(exported_sha256),
                ),
            });
        }

        Err(BackendError::new(
            "unsupported_import_target",
            "unknown mock import target",
        ))
    }

    async fn apply_edit(
        &mut self,
        params: ApplyEditParams,
    ) -> std::result::Result<MutationResult, BackendError> {
        self.require_connected()?;
        match params.operation {
            EditOperation::RenameObject { new_name } => {
                if let Ok((plc_index, block_index)) = self.block_location(&params.object_id) {
                    let block = &mut self.state.project.plc_software[plc_index].blocks[block_index];
                    let before = JsonValue::String(block.summary.object.name.clone());
                    block.summary.object.name = new_name.clone();
                    let after = JsonValue::String(new_name);
                    return Ok(MutationResult {
                        touched_objects: vec![TouchedObject {
                            object: block.summary.object.clone(),
                            changes: vec![FieldChange {
                                field: "name".to_string(),
                                before,
                                after: after.clone(),
                            }],
                        }],
                        verification: verification_from_fields(
                            "read_back",
                            vec![VerifiedField {
                                field: "name".to_string(),
                                expected: after,
                                actual: JsonValue::String(block.summary.object.name.clone()),
                            }],
                            None,
                        ),
                    });
                }

                if let Ok((plc_index, table_index)) = self.tag_table_location(&params.object_id) {
                    let table =
                        &mut self.state.project.plc_software[plc_index].tag_tables[table_index];
                    let before = JsonValue::String(table.summary.object.name.clone());
                    table.summary.object.name = new_name.clone();
                    let after = JsonValue::String(new_name);
                    return Ok(MutationResult {
                        touched_objects: vec![TouchedObject {
                            object: table.summary.object.clone(),
                            changes: vec![FieldChange {
                                field: "name".to_string(),
                                before,
                                after: after.clone(),
                            }],
                        }],
                        verification: verification_from_fields(
                            "read_back",
                            vec![VerifiedField {
                                field: "name".to_string(),
                                expected: after,
                                actual: JsonValue::String(table.summary.object.name.clone()),
                            }],
                            None,
                        ),
                    });
                }

                let (plc_index, table_index, tag_index) = self.tag_location(&params.object_id)?;
                let tag = &mut self.state.project.plc_software[plc_index].tag_tables[table_index]
                    .summary
                    .tags
                    .as_mut()
                    .and_then(|tags| tags.get_mut(tag_index))
                    .ok_or_else(|| {
                        BackendError::new("object_not_found", "PLC tag could not be resolved")
                    })?;
                let before = JsonValue::String(tag.object.name.clone());
                tag.object.name = new_name.clone();
                let after = JsonValue::String(new_name);
                Ok(MutationResult {
                    touched_objects: vec![TouchedObject {
                        object: tag.object.clone(),
                        changes: vec![FieldChange {
                            field: "name".to_string(),
                            before,
                            after: after.clone(),
                        }],
                    }],
                    verification: verification_from_fields(
                        "read_back",
                        vec![VerifiedField {
                            field: "name".to_string(),
                            expected: after,
                            actual: JsonValue::String(tag.object.name.clone()),
                        }],
                        None,
                    ),
                })
            }
            EditOperation::SetBlockHeader {
                header_author,
                header_family,
                header_name,
                header_version,
            } => {
                let (plc_index, block_index) = self.block_location(&params.object_id)?;
                let block = &mut self.state.project.plc_software[plc_index].blocks[block_index];
                let mut changes = Vec::new();
                let mut verified = Vec::new();
                let mut maybe_set_string =
                    |field: &str, slot: &mut Option<String>, next: Option<String>| {
                        if let Some(next) = next {
                            let before = json!(slot.clone());
                            *slot = Some(next.clone());
                            let after = JsonValue::String(next);
                            changes.push(FieldChange {
                                field: field.to_string(),
                                before,
                                after: after.clone(),
                            });
                            verified.push(VerifiedField {
                                field: field.to_string(),
                                expected: after,
                                actual: json!(slot.clone()),
                            });
                        }
                    };
                maybe_set_string(
                    "header_author",
                    &mut block.summary.header_author,
                    header_author,
                );
                maybe_set_string(
                    "header_family",
                    &mut block.summary.header_family,
                    header_family,
                );
                maybe_set_string("header_name", &mut block.summary.header_name, header_name);
                maybe_set_string(
                    "header_version",
                    &mut block.summary.header_version,
                    header_version,
                );

                Ok(MutationResult {
                    touched_objects: vec![TouchedObject {
                        object: block.summary.object.clone(),
                        changes,
                    }],
                    verification: verification_from_fields("read_back", verified, None),
                })
            }
            EditOperation::SetPlcTagProperties {
                name,
                data_type_name,
                logical_address,
                external_accessible,
                external_visible,
                external_writable,
                is_safety: _,
            } => {
                let (plc_index, table_index, tag_index) = self.tag_location(&params.object_id)?;
                let tag = &mut self.state.project.plc_software[plc_index].tag_tables[table_index]
                    .summary
                    .tags
                    .as_mut()
                    .and_then(|tags| tags.get_mut(tag_index))
                    .ok_or_else(|| {
                        BackendError::new("object_not_found", "PLC tag could not be resolved")
                    })?;
                let mut changes = Vec::new();
                let mut verified = Vec::new();
                if let Some(name) = name {
                    let previous = JsonValue::String(tag.object.name.clone());
                    let next = JsonValue::String(name.clone());
                    tag.object.name = name;
                    changes.push(FieldChange {
                        field: "name".to_string(),
                        before: previous,
                        after: next.clone(),
                    });
                    verified.push(VerifiedField {
                        field: "name".to_string(),
                        expected: next,
                        actual: JsonValue::String(tag.object.name.clone()),
                    });
                }
                if let Some(data_type_name) = data_type_name {
                    let previous = json!(tag.data_type_name.clone());
                    tag.data_type_name = Some(data_type_name.clone());
                    let next = JsonValue::String(data_type_name);
                    changes.push(FieldChange {
                        field: "data_type_name".to_string(),
                        before: previous,
                        after: next.clone(),
                    });
                    verified.push(VerifiedField {
                        field: "data_type_name".to_string(),
                        expected: next,
                        actual: json!(tag.data_type_name.clone()),
                    });
                }
                if let Some(logical_address) = logical_address {
                    let previous = json!(tag.logical_address.clone());
                    tag.logical_address = Some(logical_address.clone());
                    let next = JsonValue::String(logical_address);
                    changes.push(FieldChange {
                        field: "logical_address".to_string(),
                        before: previous,
                        after: next.clone(),
                    });
                    verified.push(VerifiedField {
                        field: "logical_address".to_string(),
                        expected: next,
                        actual: json!(tag.logical_address.clone()),
                    });
                }
                if let Some(external_accessible) = external_accessible {
                    let previous = json!(tag.external_accessible);
                    tag.external_accessible = Some(external_accessible);
                    let next = JsonValue::Bool(external_accessible);
                    changes.push(FieldChange {
                        field: "external_accessible".to_string(),
                        before: previous,
                        after: next.clone(),
                    });
                    verified.push(VerifiedField {
                        field: "external_accessible".to_string(),
                        expected: next,
                        actual: json!(tag.external_accessible),
                    });
                }
                if let Some(external_visible) = external_visible {
                    let previous = json!(tag.external_visible);
                    tag.external_visible = Some(external_visible);
                    let next = JsonValue::Bool(external_visible);
                    changes.push(FieldChange {
                        field: "external_visible".to_string(),
                        before: previous,
                        after: next.clone(),
                    });
                    verified.push(VerifiedField {
                        field: "external_visible".to_string(),
                        expected: next,
                        actual: json!(tag.external_visible),
                    });
                }
                if let Some(external_writable) = external_writable {
                    let previous = json!(tag.external_writable);
                    tag.external_writable = Some(external_writable);
                    let next = JsonValue::Bool(external_writable);
                    changes.push(FieldChange {
                        field: "external_writable".to_string(),
                        before: previous,
                        after: next.clone(),
                    });
                    verified.push(VerifiedField {
                        field: "external_writable".to_string(),
                        expected: next,
                        actual: json!(tag.external_writable),
                    });
                }

                Ok(MutationResult {
                    touched_objects: vec![TouchedObject {
                        object: tag.object.clone(),
                        changes,
                    }],
                    verification: verification_from_fields("read_back", verified, None),
                })
            }
        }
    }

    async fn create_udt(
        &mut self,
        params: CreateUdtParams,
    ) -> std::result::Result<MutationResult, BackendError> {
        self.require_connected()?;
        let plc_index = self.plc_index(&params.plc_software_id)?;
        let plc = &mut self.state.project.plc_software[plc_index];
        Self::ensure_unique_name(
            plc.data_types
                .iter()
                .map(|data_type| data_type.summary.object.name.clone()),
            &params.name,
            "udt",
        )?;
        let object = EngineeringObjectSummary {
            object_id: format!("udt/{}", Self::slugify(&params.name)),
            kind: "plc_data_type".to_string(),
            name: params.name.clone(),
            path: format!("{}/PLC data types/{}", plc.summary.object.path, params.name),
        };
        let summary = DataTypeSummary {
            object: object.clone(),
            data_type_kind: "udt".to_string(),
            comment: params.comment.clone(),
            members: params.members.clone(),
        };
        plc.data_types.push(MockDataType {
            summary: summary.clone(),
        });
        Ok(MutationResult {
            touched_objects: vec![TouchedObject {
                object: object.clone(),
                changes: vec![
                    FieldChange {
                        field: "created".to_string(),
                        before: JsonValue::Null,
                        after: json!(true),
                    },
                    FieldChange {
                        field: "members".to_string(),
                        before: json!([]),
                        after: json!(summary.members),
                    },
                ],
            }],
            verification: verification_from_fields(
                "read_back",
                vec![
                    VerifiedField {
                        field: "name".to_string(),
                        expected: json!(object.name),
                        actual: json!(params.name),
                    },
                    VerifiedField {
                        field: "member_count".to_string(),
                        expected: json!(params.members.len()),
                        actual: json!(params.members.len()),
                    },
                ],
                None,
            ),
        })
    }

    async fn edit_udt(
        &mut self,
        params: EditUdtParams,
    ) -> std::result::Result<MutationResult, BackendError> {
        self.require_connected()?;
        let (plc_index, data_type_index) = self.data_type_location(&params.object_id)?;
        let data_type = &mut self.state.project.plc_software[plc_index].data_types[data_type_index];
        let mut changes = Vec::new();
        let mut verified = Vec::new();
        if let Some(new_name) = params.new_name {
            let before = json!(data_type.summary.object.name.clone());
            data_type.summary.object.name = new_name.clone();
            let after = json!(new_name);
            changes.push(FieldChange {
                field: "name".to_string(),
                before,
                after: after.clone(),
            });
            verified.push(VerifiedField {
                field: "name".to_string(),
                expected: after,
                actual: json!(data_type.summary.object.name.clone()),
            });
        }
        if let Some(comment) = params.comment {
            let before = json!(data_type.summary.comment.clone());
            data_type.summary.comment = Some(comment.clone());
            let after = json!(comment);
            changes.push(FieldChange {
                field: "comment".to_string(),
                before,
                after: after.clone(),
            });
            verified.push(VerifiedField {
                field: "comment".to_string(),
                expected: after,
                actual: json!(data_type.summary.comment.clone()),
            });
        }
        if let Some(members) = params.members {
            let before = json!(data_type.summary.members.clone());
            data_type.summary.members = members.clone();
            let after = json!(members);
            changes.push(FieldChange {
                field: "members".to_string(),
                before,
                after: after.clone(),
            });
            verified.push(VerifiedField {
                field: "members".to_string(),
                expected: after,
                actual: json!(data_type.summary.members.clone()),
            });
        }
        Ok(MutationResult {
            touched_objects: vec![TouchedObject {
                object: data_type.summary.object.clone(),
                changes,
            }],
            verification: verification_from_fields("read_back", verified, None),
        })
    }

    async fn create_block(
        &mut self,
        params: CreateBlockParams,
    ) -> std::result::Result<MutationResult, BackendError> {
        self.require_connected()?;
        let plc_index = self.plc_index(&params.plc_software_id)?;
        let plc = &mut self.state.project.plc_software[plc_index];
        Self::ensure_unique_name(
            plc.blocks
                .iter()
                .map(|block| block.summary.object.name.clone()),
            &params.name,
            "block",
        )?;
        let object = EngineeringObjectSummary {
            object_id: format!("block/{}", Self::slugify(&params.name)),
            kind: if matches!(
                params.block_kind,
                NewBlockKind::GlobalDb | NewBlockKind::InstanceDb
            ) {
                "data_block".to_string()
            } else {
                "code_block".to_string()
            },
            name: params.name.clone(),
            path: format!("{}/Program blocks/{}", plc.summary.object.path, params.name),
        };
        let block = MockBlock {
            summary: BlockSummary {
                object: object.clone(),
                block_type: Self::block_type_for_kind(&params.block_kind).to_string(),
                group_path: "Program blocks".to_string(),
                number: Some(Self::next_block_number(plc, &params.block_kind)),
                header_author: params.header_author.clone(),
                header_family: params.header_family.clone(),
                header_name: params.header_name.clone(),
                header_version: params.header_version.clone(),
            },
            language: params.language.clone(),
            comment: None,
            block_body: params.block_body.clone().unwrap_or_else(|| {
                Self::generate_block_body(
                    &params.block_kind,
                    &params.name,
                    params.language.as_ref(),
                )
            }),
            db_members: Vec::new(),
            calls: Vec::new(),
        };
        plc.blocks.push(block.clone());
        Ok(MutationResult {
            touched_objects: vec![TouchedObject {
                object,
                changes: vec![
                    FieldChange {
                        field: "created".to_string(),
                        before: JsonValue::Null,
                        after: json!(true),
                    },
                    FieldChange {
                        field: "block_type".to_string(),
                        before: JsonValue::Null,
                        after: json!(block.summary.block_type),
                    },
                ],
            }],
            verification: verification_from_fields(
                "read_back",
                vec![VerifiedField {
                    field: "name".to_string(),
                    expected: json!(params.name),
                    actual: json!(block.summary.object.name),
                }],
                None,
            ),
        })
    }

    async fn edit_block_body(
        &mut self,
        params: EditBlockBodyParams,
    ) -> std::result::Result<MutationResult, BackendError> {
        self.require_connected()?;
        let (plc_index, block_index) = self.block_location(&params.object_id)?;
        let block = &mut self.state.project.plc_software[plc_index].blocks[block_index];
        if Self::block_is_db(block) {
            return Err(BackendError::with_details(
                "unsupported_block_kind",
                "edit_block_body only supports executable code blocks in the simulator backend",
                json!({ "object_id": params.object_id, "block_type": block.summary.block_type }),
            ));
        }
        let before_body = JsonValue::String(block.block_body.clone());
        block.block_body = params.block_body.clone();
        let mut changes = vec![FieldChange {
            field: "block_body".to_string(),
            before: before_body,
            after: JsonValue::String(params.block_body.clone()),
        }];
        let mut verified = vec![VerifiedField {
            field: "block_body".to_string(),
            expected: JsonValue::String(params.block_body),
            actual: JsonValue::String(block.block_body.clone()),
        }];
        if let Some(language) = params.language {
            let before = json!(block.language.clone());
            block.language = Some(language.clone());
            let after = json!(language);
            changes.push(FieldChange {
                field: "language".to_string(),
                before,
                after: after.clone(),
            });
            verified.push(VerifiedField {
                field: "language".to_string(),
                expected: after,
                actual: json!(block.language.clone()),
            });
        }
        if let Some(comment) = params.comment {
            let before = json!(block.comment.clone());
            block.comment = Some(comment.clone());
            let after = json!(comment);
            changes.push(FieldChange {
                field: "comment".to_string(),
                before,
                after: after.clone(),
            });
            verified.push(VerifiedField {
                field: "comment".to_string(),
                expected: after,
                actual: json!(block.comment.clone()),
            });
        }
        Ok(MutationResult {
            touched_objects: vec![TouchedObject {
                object: block.summary.object.clone(),
                changes,
            }],
            verification: verification_from_fields("read_back", verified, None),
        })
    }

    async fn create_block_call(
        &mut self,
        params: CreateBlockCallParams,
    ) -> std::result::Result<MutationResult, BackendError> {
        self.require_connected()?;
        let (caller_plc_index, caller_block_index) =
            self.block_location(&params.caller_block_id)?;
        let (callee_plc_index, callee_block_index) =
            self.block_location(&params.callee_block_id)?;
        if caller_plc_index != callee_plc_index {
            return Err(BackendError::new(
                "cross_plc_call_unsupported",
                "mock block-call creation only supports caller and callee in the same PLC",
            ));
        }
        let plc = &mut self.state.project.plc_software[caller_plc_index];
        let callee = plc.blocks[callee_block_index].clone();
        let mut touched_objects = Vec::new();
        let binding_lines = params
            .parameter_bindings
            .iter()
            .map(|binding| format!("    {} := {}", binding.parameter, binding.expression))
            .collect::<Vec<_>>();
        let mut snippet = format!("\n// Auto-generated call to {}", callee.summary.object.name);
        if let Some(comment) = &params.comment {
            snippet.push_str(&format!("\n// {comment}"));
        }
        snippet.push_str(&format!("\n{}(", callee.summary.object.name));
        if !binding_lines.is_empty() {
            snippet.push('\n');
            snippet.push_str(&binding_lines.join(",\n"));
            snippet.push('\n');
        }
        snippet.push_str(");");
        let needs_instance_db = matches!(callee.summary.block_type.as_str(), "FB")
            && params.instance_db_name.is_some()
            && !plc.blocks.iter().any(|block| {
                params
                    .instance_db_name
                    .as_ref()
                    .map(|instance_db_name| block.summary.object.name == *instance_db_name)
                    .unwrap_or(false)
            });
        let instance_db = if let Some(instance_db_name) = params.instance_db_name.clone() {
            if !needs_instance_db {
                None
            } else {
                Some(MockBlock {
                    summary: BlockSummary {
                        object: EngineeringObjectSummary {
                            object_id: format!("block/{}", Self::slugify(&instance_db_name)),
                            kind: "data_block".to_string(),
                            name: instance_db_name.clone(),
                            path: format!(
                                "{}/Program blocks/{}",
                                plc.summary.object.path, instance_db_name
                            ),
                        },
                        block_type: "InstanceDB".to_string(),
                        group_path: "Program blocks".to_string(),
                        number: Some(Self::next_block_number(plc, &NewBlockKind::InstanceDb)),
                        header_author: None,
                        header_family: Some("Instances".to_string()),
                        header_name: Some(instance_db_name),
                        header_version: Some("1.0.0".to_string()),
                    },
                    language: None,
                    comment: Some(format!("Instance DB for {}", callee.summary.object.name)),
                    block_body: String::new(),
                    db_members: vec![DbMemberDefinition {
                        name: "InstanceOf".to_string(),
                        data_type_name: callee.summary.object.name.clone(),
                        comment: Some("Instance binding".to_string()),
                        initial_value: None,
                    }],
                    calls: Vec::new(),
                })
            }
        } else {
            None
        };
        let caller_object;
        let caller_contains_callee;
        let body_before;
        {
            let caller = &mut plc.blocks[caller_block_index];
            body_before = caller.block_body.clone();
            caller.block_body.push_str(&snippet);
            caller.calls.push(MockBlockCall {
                callee_block_id: params.callee_block_id.clone(),
                instance_db_name: params.instance_db_name.clone(),
                comment: params.comment.clone(),
                parameter_bindings: params.parameter_bindings,
            });
            caller_contains_callee = caller.block_body.contains(&callee.summary.object.name);
            caller_object = caller.summary.object.clone();
        }
        touched_objects.push(TouchedObject {
            object: caller_object,
            changes: vec![FieldChange {
                field: "block_body".to_string(),
                before: JsonValue::String(body_before),
                after: JsonValue::String(plc.blocks[caller_block_index].block_body.clone()),
            }],
        });

        if let Some(instance_db) = instance_db {
            touched_objects.push(TouchedObject {
                object: instance_db.summary.object.clone(),
                changes: vec![FieldChange {
                    field: "created".to_string(),
                    before: JsonValue::Null,
                    after: json!(true),
                }],
            });
            plc.blocks.push(instance_db);
        }

        Ok(MutationResult {
            verification: verification_from_fields(
                "read_back",
                vec![VerifiedField {
                    field: "caller_body_contains_callee".to_string(),
                    expected: json!(true),
                    actual: json!(caller_contains_callee),
                }],
                None,
            ),
            touched_objects,
        })
    }

    async fn edit_db_members(
        &mut self,
        params: EditDbMembersParams,
    ) -> std::result::Result<MutationResult, BackendError> {
        self.require_connected()?;
        let (plc_index, block_index) = self.block_location(&params.object_id)?;
        let block = &mut self.state.project.plc_software[plc_index].blocks[block_index];
        if !Self::block_is_db(block) {
            return Err(BackendError::with_details(
                "unsupported_block_kind",
                "edit_db_members only supports global and instance DB blocks",
                json!({ "object_id": params.object_id, "block_type": block.summary.block_type }),
            ));
        }
        let before = json!(block.db_members.clone());
        if params.replace_existing {
            block.db_members = params.members;
        } else {
            for member in params.members {
                if let Some(existing) = block
                    .db_members
                    .iter_mut()
                    .find(|existing| existing.name == member.name)
                {
                    *existing = member;
                } else {
                    block.db_members.push(member);
                }
            }
        }
        let after = json!(block.db_members.clone());
        Ok(MutationResult {
            touched_objects: vec![TouchedObject {
                object: block.summary.object.clone(),
                changes: vec![FieldChange {
                    field: "db_members".to_string(),
                    before,
                    after: after.clone(),
                }],
            }],
            verification: verification_from_fields(
                "read_back",
                vec![VerifiedField {
                    field: "db_members".to_string(),
                    expected: after,
                    actual: json!(block.db_members.clone()),
                }],
                None,
            ),
        })
    }

    async fn create_plc_tag(
        &mut self,
        params: CreatePlcTagParams,
    ) -> std::result::Result<MutationResult, BackendError> {
        self.require_connected()?;
        let (plc_index, table_index) = self.tag_table_location(&params.tag_table_object_id)?;
        let table = &mut self.state.project.plc_software[plc_index].tag_tables[table_index];
        let tags = table.summary.tags.get_or_insert_with(Vec::new);
        Self::ensure_unique_name(
            tags.iter().map(|tag| tag.object.name.clone()),
            &params.name,
            "plc_tag",
        )?;
        let object = EngineeringObjectSummary {
            object_id: format!("tag/{}", Self::slugify(&params.name)),
            kind: "plc_tag".to_string(),
            name: params.name.clone(),
            path: format!("{}/{}", table.summary.object.path, params.name),
        };
        let tag = PlcTagSummary {
            object: object.clone(),
            data_type_name: Some(params.data_type_name.clone()),
            logical_address: params.logical_address.clone(),
            external_accessible: params.external_accessible,
            external_visible: params.external_visible,
            external_writable: params.external_writable,
        };
        tags.push(tag.clone());
        Ok(MutationResult {
            touched_objects: vec![TouchedObject {
                object,
                changes: vec![FieldChange {
                    field: "created".to_string(),
                    before: JsonValue::Null,
                    after: json!(true),
                }],
            }],
            verification: verification_from_fields(
                "read_back",
                vec![VerifiedField {
                    field: "data_type_name".to_string(),
                    expected: json!(params.data_type_name),
                    actual: json!(tag.data_type_name),
                }],
                None,
            ),
        })
    }

    async fn create_tag_table(
        &mut self,
        params: CreateTagTableParams,
    ) -> std::result::Result<MutationResult, BackendError> {
        self.require_connected()?;
        let plc_index = self.plc_index(&params.plc_software_id)?;
        let plc = &mut self.state.project.plc_software[plc_index];
        Self::ensure_unique_name(
            plc.tag_tables
                .iter()
                .map(|table| table.summary.object.name.clone()),
            &params.name,
            "plc_tag_table",
        )?;
        let object = EngineeringObjectSummary {
            object_id: format!("tag_table/{}", Self::slugify(&params.name)),
            kind: "plc_tag_table".to_string(),
            name: params.name.clone(),
            path: format!("{}/PLC tags/{}", plc.summary.object.path, params.name),
        };
        plc.tag_tables.push(MockTagTable {
            summary: TagTableSummary {
                object: object.clone(),
                group_path: "PLC tags".to_string(),
                tags: Some(Vec::new()),
            },
        });
        Ok(MutationResult {
            touched_objects: vec![TouchedObject {
                object: object.clone(),
                changes: vec![FieldChange {
                    field: "created".to_string(),
                    before: JsonValue::Null,
                    after: json!(true),
                }],
            }],
            verification: verification_from_fields(
                "read_back",
                vec![VerifiedField {
                    field: "name".to_string(),
                    expected: json!(params.name),
                    actual: json!(object.name),
                }],
                None,
            ),
        })
    }

    async fn list_technology_objects(
        &mut self,
        params: ListTechnologyObjectsParams,
    ) -> std::result::Result<ListTechnologyObjectsResult, BackendError> {
        self.require_connected()?;
        let plc = &self.state.project.plc_software[self.plc_index(&params.plc_software_id)?];
        Ok(ListTechnologyObjectsResult {
            plc_software_id: params.plc_software_id,
            technology_objects: plc.technology_objects.clone(),
        })
    }

    async fn list_watch_tables(
        &mut self,
        params: ListWatchTablesParams,
    ) -> std::result::Result<ListWatchTablesResult, BackendError> {
        self.require_connected()?;
        let plc = &self.state.project.plc_software[self.plc_index(&params.plc_software_id)?];
        Ok(ListWatchTablesResult {
            plc_software_id: params.plc_software_id,
            watch_tables: plc.watch_tables.clone(),
        })
    }

    async fn create_watch_table(
        &mut self,
        params: CreateWatchTableParams,
    ) -> std::result::Result<MutationResult, BackendError> {
        self.require_connected()?;
        let plc_index = self.plc_index(&params.plc_software_id)?;
        let plc = &mut self.state.project.plc_software[plc_index];
        Self::ensure_unique_name(
            plc.watch_tables
                .iter()
                .map(|watch_table| watch_table.object.name.clone()),
            &params.name,
            "watch_table",
        )?;
        let object = EngineeringObjectSummary {
            object_id: format!("watch/{}", Self::slugify(&params.name)),
            kind: "watch_table".to_string(),
            name: params.name.clone(),
            path: format!("{}/Watch tables/{}", plc.summary.object.path, params.name),
        };
        plc.watch_tables.push(WatchTableSummary {
            object: object.clone(),
            expressions: params.expressions.clone(),
        });
        Ok(MutationResult {
            touched_objects: vec![TouchedObject {
                object,
                changes: vec![FieldChange {
                    field: "created".to_string(),
                    before: JsonValue::Null,
                    after: json!(true),
                }],
            }],
            verification: verification_from_fields(
                "read_back",
                vec![VerifiedField {
                    field: "expression_count".to_string(),
                    expected: json!(params.expressions.len()),
                    actual: json!(params.expressions.len()),
                }],
                None,
            ),
        })
    }

    async fn list_networks(
        &mut self,
        _params: ListNetworksParams,
    ) -> std::result::Result<ListNetworksResult, BackendError> {
        self.require_connected()?;
        Ok(ListNetworksResult {
            networks: self.state.project.networks.clone(),
        })
    }

    async fn list_hmi_objects(
        &mut self,
        _params: ListHmiObjectsParams,
    ) -> std::result::Result<ListHmiObjectsResult, BackendError> {
        self.require_connected()?;
        Ok(ListHmiObjectsResult {
            hmi_objects: self
                .state
                .project
                .hmi_objects
                .iter()
                .map(|hmi| hmi.summary.clone())
                .collect(),
        })
    }

    async fn list_safety_objects(
        &mut self,
        params: ListSafetyObjectsParams,
    ) -> std::result::Result<ListSafetyObjectsResult, BackendError> {
        self.require_connected()?;
        if let Some(plc_software_id) = &params.plc_software_id {
            let plc = &self.state.project.plc_software[self.plc_index(plc_software_id)?];
            return Ok(ListSafetyObjectsResult {
                plc_software_id: Some(plc_software_id.clone()),
                safety_objects: plc.safety_objects.clone(),
            });
        }
        Ok(ListSafetyObjectsResult {
            plc_software_id: None,
            safety_objects: self
                .state
                .project
                .plc_software
                .iter()
                .flat_map(|plc| plc.safety_objects.clone())
                .collect(),
        })
    }

    async fn write_hardware_config(
        &mut self,
        params: WriteHardwareConfigParams,
    ) -> std::result::Result<MutationResult, BackendError> {
        self.write_hardware_config_impl(params)
    }

    async fn write_network_config(
        &mut self,
        params: WriteNetworkConfigParams,
    ) -> std::result::Result<MutationResult, BackendError> {
        self.write_network_config_impl(params)
    }

    async fn create_hmi_alarm(
        &mut self,
        params: CreateHmiAlarmParams,
    ) -> std::result::Result<MutationResult, BackendError> {
        self.create_hmi_alarm_impl(params)
    }

    async fn create_technology_object(
        &mut self,
        params: CreateTechnologyObjectParams,
    ) -> std::result::Result<MutationResult, BackendError> {
        self.create_technology_object_impl(params)
    }

    async fn create_safety_object(
        &mut self,
        params: CreateSafetyObjectParams,
    ) -> std::result::Result<MutationResult, BackendError> {
        self.create_safety_object_impl(params)
    }

    async fn cross_reference(
        &mut self,
        params: CrossReferenceParams,
    ) -> std::result::Result<CrossReferenceResult, BackendError> {
        self.require_connected()?;
        let target = self.object_summary(&params.object_id).ok_or_else(|| {
            BackendError::with_details(
                "object_not_found",
                "unknown engineering object id",
                json!({ "object_id": params.object_id }),
            )
        })?;
        let mut references = Vec::new();
        for plc in &self.state.project.plc_software {
            for block in &plc.blocks {
                if block.summary.object.object_id == params.object_id {
                    for call in &block.calls {
                        if let Some(callee) = plc.blocks.iter().find(|candidate| {
                            candidate.summary.object.object_id == call.callee_block_id
                        }) {
                            references.push(CrossReferenceHit {
                                object: callee.summary.object.clone(),
                                relation: "calls".to_string(),
                                detail: Some(
                                    call.parameter_bindings
                                        .iter()
                                        .map(|binding| {
                                            format!(
                                                "{} := {}",
                                                binding.parameter, binding.expression
                                            )
                                        })
                                        .collect::<Vec<_>>()
                                        .join(", "),
                                ),
                            });
                        }
                    }
                } else {
                    for call in &block.calls {
                        if call.callee_block_id == params.object_id {
                            references.push(CrossReferenceHit {
                                object: block.summary.object.clone(),
                                relation: "called_by".to_string(),
                                detail: call.comment.clone(),
                            });
                        }
                    }
                }
                if target.kind == "plc_tag" && block.block_body.contains(&target.name) {
                    references.push(CrossReferenceHit {
                        object: block.summary.object.clone(),
                        relation: "used_by_block".to_string(),
                        detail: Some(target.name.clone()),
                    });
                }
                if target.kind == "plc_data_type"
                    && block
                        .db_members
                        .iter()
                        .any(|member| member.data_type_name == target.name)
                {
                    references.push(CrossReferenceHit {
                        object: block.summary.object.clone(),
                        relation: "used_by_db_member".to_string(),
                        detail: Some(target.name.clone()),
                    });
                }
            }
            for watch_table in &plc.watch_tables {
                if watch_table
                    .expressions
                    .iter()
                    .any(|expression| expression.expression == target.name)
                {
                    references.push(CrossReferenceHit {
                        object: watch_table.object.clone(),
                        relation: "watched_by".to_string(),
                        detail: Some(target.name.clone()),
                    });
                }
            }
        }
        Ok(CrossReferenceResult { target, references })
    }

    async fn consistency_check(
        &mut self,
        params: ConsistencyCheckParams,
    ) -> std::result::Result<ConsistencyCheckResult, BackendError> {
        self.require_connected()?;
        let scope = self.resolve_scope_summary(&params.scope)?;
        let issues = match &params.scope {
            ConsistencyCheckScope::CurrentProject => self
                .state
                .project
                .plc_software
                .iter()
                .flat_map(Self::consistency_issues_for_plc)
                .collect::<Vec<_>>(),
            ConsistencyCheckScope::PlcSoftware { plc_software_id } => {
                vec![self.plc_index(plc_software_id)?]
                    .into_iter()
                    .flat_map(|index| {
                        Self::consistency_issues_for_plc(&self.state.project.plc_software[index])
                    })
                    .collect()
            }
            ConsistencyCheckScope::Object { object_id } => self
                .state
                .project
                .plc_software
                .iter()
                .flat_map(Self::consistency_issues_for_plc)
                .filter(|issue| {
                    issue
                        .object
                        .as_ref()
                        .map(|object| object.object_id == *object_id)
                        .unwrap_or(false)
                })
                .collect(),
        };
        Ok(ConsistencyCheckResult {
            scope,
            issue_count: issues.len() as u32,
            issues,
        })
    }

    async fn compare_online_offline(
        &mut self,
        params: CompareOnlineOfflineParams,
    ) -> std::result::Result<CompareOnlineOfflineResult, BackendError> {
        self.require_connected()?;
        let scope = self.resolve_compare_scope_summary(&params.scope)?;
        let differences = match &params.scope {
            CompareOnlineOfflineScope::CurrentProject => self
                .state
                .project
                .plc_software
                .iter()
                .flat_map(|plc| plc.blocks.iter())
                .filter(|block| block.block_body.contains("ONLINE_DRIFT"))
                .map(|block| OnlineDifference {
                    path: block.summary.object.path.clone(),
                    difference_type: "logic_body".to_string(),
                    description: "Mock online/offline drift sentinel detected.".to_string(),
                })
                .collect::<Vec<_>>(),
            CompareOnlineOfflineScope::PlcSoftware { plc_software_id } => {
                self.state.project.plc_software[self.plc_index(plc_software_id)?]
                    .blocks
                    .iter()
                    .filter(|block| block.block_body.contains("ONLINE_DRIFT"))
                    .map(|block| OnlineDifference {
                        path: block.summary.object.path.clone(),
                        difference_type: "logic_body".to_string(),
                        description: "Mock online/offline drift sentinel detected.".to_string(),
                    })
                    .collect::<Vec<_>>()
            }
            CompareOnlineOfflineScope::Object { object_id } => {
                if let Ok((plc_index, block_index)) = self.block_location(object_id) {
                    let block = &self.state.project.plc_software[plc_index].blocks[block_index];
                    if block.block_body.contains("ONLINE_DRIFT") {
                        vec![OnlineDifference {
                            path: block.summary.object.path.clone(),
                            difference_type: "logic_body".to_string(),
                            description: "Mock online/offline drift sentinel detected.".to_string(),
                        }]
                    } else {
                        Vec::new()
                    }
                } else {
                    Vec::new()
                }
            }
        };
        Ok(CompareOnlineOfflineResult {
            scope,
            status: if differences.is_empty() {
                "in_sync".to_string()
            } else {
                "differences_found".to_string()
            },
            differences,
        })
    }

    async fn run_simulation(
        &mut self,
        params: RunSimulationParams,
    ) -> std::result::Result<RunSimulationResult, BackendError> {
        self.require_connected()?;
        let plc = &self.state.project.plc_software[self.plc_index(&params.plc_software_id)?];
        let cycles = params.duration_cycles.unwrap_or(3).max(1);
        let mut observations = Vec::new();
        for cycle in 1..=cycles {
            observations.push(SimulationObservation {
                cycle,
                signal: "Start".to_string(),
                value: if cycle == 1 { "FALSE" } else { "TRUE" }.to_string(),
            });
            observations.push(SimulationObservation {
                cycle,
                signal: "MotorRun".to_string(),
                value: if cycle == 1 { "FALSE" } else { "TRUE" }.to_string(),
            });
        }
        Ok(RunSimulationResult {
            plc_software: plc.summary.object.clone(),
            status: "completed".to_string(),
            observations,
        })
    }

    async fn go_online(
        &mut self,
        params: GoOnlineParams,
    ) -> std::result::Result<MutationResult, BackendError> {
        self.go_online_impl(params)
    }

    async fn download_to_device(
        &mut self,
        params: DownloadToDeviceParams,
    ) -> std::result::Result<MutationResult, BackendError> {
        self.download_to_device_impl(params)
    }

    async fn compile(
        &mut self,
        params: CompileParams,
    ) -> std::result::Result<CompileResultEnvelope, BackendError> {
        self.require_connected()?;
        match params.scope {
            CompileScope::CurrentProject => {
                let mut messages = Vec::new();
                let mut error_count = 0;
                for plc in &self.state.project.plc_software {
                    for block in &plc.blocks {
                        if block.block_body.contains("COMPILE_ERROR") {
                            error_count += 1;
                            messages.push(CompilerMessageSummary {
                                path: Some(block.summary.object.path.clone()),
                                state: Some("error".to_string()),
                                description: Some(
                                    "Mock compile failed because block body contains COMPILE_ERROR"
                                        .to_string(),
                                ),
                                warning_count: 0,
                                error_count: 1,
                                messages: Vec::new(),
                            });
                        } else if Self::validate_header_family(block).is_some() {
                            error_count += 1;
                            messages.push(CompilerMessageSummary {
                                path: Some(block.summary.object.path.clone()),
                                state: Some("error".to_string()),
                                description: Some(
                                    "Mock compile failed because header_family contains invalid characters"
                                        .to_string(),
                                ),
                                warning_count: 0,
                                error_count: 1,
                                messages: Vec::new(),
                            });
                        }
                    }
                }

                Ok(CompileResultEnvelope {
                    scope: self.state.project.object.clone(),
                    result: CompilerResultSummary {
                        state: Some(if error_count == 0 {
                            "success".to_string()
                        } else {
                            "failed".to_string()
                        }),
                        warning_count: 0,
                        error_count,
                        messages,
                    },
                })
            }
            CompileScope::Object { object_id } => {
                let (plc_index, block_index) = self.block_location(&object_id)?;
                let block = &self.state.project.plc_software[plc_index].blocks[block_index];
                let failed = block.block_body.contains("COMPILE_ERROR")
                    || Self::validate_header_family(block).is_some();
                let description = if block.block_body.contains("COMPILE_ERROR") {
                    "Mock compile failed because block body contains COMPILE_ERROR"
                } else {
                    "Mock compile failed because header_family contains invalid characters"
                };
                Ok(CompileResultEnvelope {
                    scope: block.summary.object.clone(),
                    result: CompilerResultSummary {
                        state: Some(if failed {
                            "failed".to_string()
                        } else {
                            "success".to_string()
                        }),
                        warning_count: 0,
                        error_count: u32::from(failed),
                        messages: if failed {
                            vec![CompilerMessageSummary {
                                path: Some(block.summary.object.path.clone()),
                                state: Some("error".to_string()),
                                description: Some(description.to_string()),
                                warning_count: 0,
                                error_count: 1,
                                messages: Vec::new(),
                            }]
                        } else {
                            Vec::new()
                        },
                    },
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn mock_backend_apply_edit_verifies_read_back() {
        let options = BackendOptions {
            transport: crate::backend::BackendTransport::Simulator,
            adapter_command: None,
            adapter_args: Vec::new(),
            simulator_state_path: None,
        };
        let mut backend = MockBackend::from_options(options)
            .await
            .expect("simulator backend should initialize");
        backend
            .connect(ConnectParams {
                connection_mode: None,
                ui_mode: None,
                portal_version: None,
                process_id: None,
            })
            .await
            .expect("connect should succeed");

        let result = backend
            .apply_edit(ApplyEditParams {
                object_id: "block/motor_fb".to_string(),
                operation: EditOperation::SetBlockHeader {
                    header_author: Some("Automation QA".to_string()),
                    header_family: None,
                    header_name: None,
                    header_version: None,
                },
            })
            .await
            .expect("edit should succeed");

        assert_eq!(result.touched_objects.len(), 1);
        assert!(result.verification.verified);
        assert_eq!(
            result.touched_objects[0].changes[0].field,
            "header_author".to_string()
        );
    }
}
