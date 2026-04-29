use std::collections::HashSet;

use serde_json::Value as JsonValue;
use serde_json::json;

use super::*;
use crate::integration_types::CreateHmiAlarmParams;
use crate::integration_types::CreateSafetyObjectParams;
use crate::integration_types::CreateTechnologyObjectParams;
use crate::integration_types::DownloadMode;
use crate::integration_types::DownloadToDeviceParams;
use crate::integration_types::GoOnlineParams;
use crate::integration_types::HardwareConfigOperation;
use crate::integration_types::HmiAlarmSeverity;
use crate::integration_types::NetworkConfigOperation;
use crate::integration_types::OnlineSessionMode;
use crate::integration_types::PostDownloadOnlineAction;
use crate::integration_types::WriteHardwareConfigParams;
use crate::integration_types::WriteNetworkConfigParams;

impl MockBackend {
    fn device_index(&self, object_id: &str) -> std::result::Result<usize, BackendError> {
        self.state
            .project
            .devices
            .iter()
            .position(|device| device.object.object_id == object_id)
            .ok_or_else(|| {
                BackendError::with_details(
                    "object_not_found",
                    "unknown device object id",
                    json!({ "device_object_id": object_id }),
                )
            })
    }

    fn network_index(&self, object_id: &str) -> std::result::Result<usize, BackendError> {
        self.state
            .project
            .networks
            .iter()
            .position(|network| network.object.object_id == object_id)
            .ok_or_else(|| {
                BackendError::with_details(
                    "object_not_found",
                    "unknown network object id",
                    json!({ "network_object_id": object_id }),
                )
            })
    }

    fn hmi_object_index(&self, object_id: &str) -> std::result::Result<usize, BackendError> {
        self.state
            .project
            .hmi_objects
            .iter()
            .position(|hmi| hmi.summary.object.object_id == object_id)
            .ok_or_else(|| {
                BackendError::with_details(
                    "object_not_found",
                    "unknown HMI object id",
                    json!({ "hmi_object_id": object_id }),
                )
            })
    }

    fn linked_plc_indices_for_device(&self, device_object_id: &str) -> Vec<usize> {
        self.state
            .project
            .plc_software
            .iter()
            .enumerate()
            .filter_map(|(index, plc)| {
                if plc.summary.device_id == device_object_id {
                    Some(index)
                } else {
                    None
                }
            })
            .collect()
    }

    fn replace_path_prefix(path: &str, old_prefix: &str, new_prefix: &str) -> String {
        if let Some(suffix) = path.strip_prefix(old_prefix) {
            format!("{new_prefix}{suffix}")
        } else {
            path.to_string()
        }
    }

    fn replace_terminal_path_segment(path: &str, new_name: &str) -> String {
        if let Some((parent, _)) = path.rsplit_once('/') {
            format!("{parent}/{new_name}")
        } else {
            new_name.to_string()
        }
    }

    fn hmi_alarm_severity_label(severity: HmiAlarmSeverity) -> &'static str {
        match severity {
            HmiAlarmSeverity::Info => "info",
            HmiAlarmSeverity::Warning => "warning",
            HmiAlarmSeverity::Error => "error",
            HmiAlarmSeverity::Critical => "critical",
        }
    }

    fn resolve_tag_reference(
        &self,
        reference: &str,
    ) -> std::result::Result<EngineeringObjectSummary, BackendError> {
        let matches = self
            .state
            .project
            .plc_software
            .iter()
            .flat_map(|plc| plc.tag_tables.iter())
            .filter_map(|table| table.summary.tags.as_ref())
            .flat_map(|tags| tags.iter())
            .filter(|tag| tag.object.object_id == reference || tag.object.name == reference)
            .map(|tag| tag.object.clone())
            .collect::<Vec<_>>();
        match matches.as_slice() {
            [] => Err(BackendError::with_details(
                "object_not_found",
                "unknown PLC tag reference for HMI alarm trigger",
                json!({ "trigger_tag": reference }),
            )),
            [single] => Ok(single.clone()),
            multiple => Err(BackendError::with_details(
                "ambiguous_reference",
                "multiple PLC tags matched the requested HMI alarm trigger reference",
                json!({
                    "trigger_tag": reference,
                    "matches": multiple.iter().map(|tag| tag.object_id.clone()).collect::<Vec<_>>(),
                }),
            )),
        }
    }

    fn downloadable_object_ids_for_device(&self, device_object_id: &str) -> HashSet<String> {
        let mut object_ids = HashSet::from([device_object_id.to_string()]);
        for plc_index in self.linked_plc_indices_for_device(device_object_id) {
            let plc = &self.state.project.plc_software[plc_index];
            object_ids.insert(plc.summary.object.object_id.clone());
            for block in &plc.blocks {
                object_ids.insert(block.summary.object.object_id.clone());
            }
            for table in &plc.tag_tables {
                object_ids.insert(table.summary.object.object_id.clone());
                if let Some(tags) = &table.summary.tags {
                    for tag in tags {
                        object_ids.insert(tag.object.object_id.clone());
                    }
                }
            }
            for data_type in &plc.data_types {
                object_ids.insert(data_type.summary.object.object_id.clone());
            }
            for technology_object in &plc.technology_objects {
                object_ids.insert(technology_object.object.object_id.clone());
            }
            for watch_table in &plc.watch_tables {
                object_ids.insert(watch_table.object.object_id.clone());
            }
            for safety_object in &plc.safety_objects {
                object_ids.insert(safety_object.object.object_id.clone());
            }
        }
        object_ids
    }

    pub(super) fn write_hardware_config_impl(
        &mut self,
        params: WriteHardwareConfigParams,
    ) -> std::result::Result<MutationResult, BackendError> {
        self.require_connected()?;
        let device_index = self.device_index(&params.device_object_id)?;
        match params.operation {
            HardwareConfigOperation::RenameDevice { new_name } => {
                Self::ensure_unique_name(
                    self.state
                        .project
                        .devices
                        .iter()
                        .enumerate()
                        .filter_map(|(index, device)| {
                            if index == device_index {
                                None
                            } else {
                                Some(device.object.name.clone())
                            }
                        }),
                    &new_name,
                    "device",
                )?;
                let device_id = self.state.project.devices[device_index]
                    .object
                    .object_id
                    .clone();
                let old_path = self.state.project.devices[device_index].object.path.clone();
                let before_name = self.state.project.devices[device_index].object.name.clone();
                let new_path = Self::replace_terminal_path_segment(&old_path, &new_name);
                let linked_plc_indices = self.linked_plc_indices_for_device(&device_id);
                {
                    let device = &mut self.state.project.devices[device_index];
                    device.object.name = new_name.clone();
                    device.object.path = new_path.clone();
                    for item in &mut device.device_items {
                        item.object.path =
                            Self::replace_path_prefix(&item.object.path, &old_path, &new_path);
                    }
                }
                for plc_index in linked_plc_indices {
                    let plc = &mut self.state.project.plc_software[plc_index];
                    let old_plc_path = plc.summary.object.path.clone();
                    plc.summary.device_name = new_name.clone();
                    plc.summary.object.path =
                        Self::replace_path_prefix(&plc.summary.object.path, &old_path, &new_path);
                    let new_plc_path = plc.summary.object.path.clone();
                    for block in &mut plc.blocks {
                        block.summary.object.path = Self::replace_path_prefix(
                            &block.summary.object.path,
                            &old_plc_path,
                            &new_plc_path,
                        );
                    }
                    for table in &mut plc.tag_tables {
                        table.summary.object.path = Self::replace_path_prefix(
                            &table.summary.object.path,
                            &old_plc_path,
                            &new_plc_path,
                        );
                        if let Some(tags) = &mut table.summary.tags {
                            for tag in tags {
                                tag.object.path = Self::replace_path_prefix(
                                    &tag.object.path,
                                    &old_plc_path,
                                    &new_plc_path,
                                );
                            }
                        }
                    }
                    for data_type in &mut plc.data_types {
                        data_type.summary.object.path = Self::replace_path_prefix(
                            &data_type.summary.object.path,
                            &old_plc_path,
                            &new_plc_path,
                        );
                    }
                    for technology_object in &mut plc.technology_objects {
                        technology_object.object.path = Self::replace_path_prefix(
                            &technology_object.object.path,
                            &old_plc_path,
                            &new_plc_path,
                        );
                    }
                    for watch_table in &mut plc.watch_tables {
                        watch_table.object.path = Self::replace_path_prefix(
                            &watch_table.object.path,
                            &old_plc_path,
                            &new_plc_path,
                        );
                    }
                    for safety_object in &mut plc.safety_objects {
                        safety_object.object.path = Self::replace_path_prefix(
                            &safety_object.object.path,
                            &old_plc_path,
                            &new_plc_path,
                        );
                    }
                }
                let device = &self.state.project.devices[device_index];
                Ok(MutationResult {
                    touched_objects: vec![TouchedObject {
                        object: device.object.clone(),
                        changes: vec![
                            FieldChange {
                                field: "name".to_string(),
                                before: json!(before_name),
                                after: json!(device.object.name),
                            },
                            FieldChange {
                                field: "path".to_string(),
                                before: json!(old_path),
                                after: json!(device.object.path),
                            },
                        ],
                    }],
                    verification: verification_from_fields(
                        "read_back",
                        vec![
                            VerifiedField {
                                field: "name".to_string(),
                                expected: json!(new_name),
                                actual: json!(device.object.name),
                            },
                            VerifiedField {
                                field: "path".to_string(),
                                expected: json!(new_path),
                                actual: json!(device.object.path),
                            },
                        ],
                        None,
                    ),
                })
            }
            HardwareConfigOperation::SetProfinetDeviceName {
                profinet_device_name,
            } => {
                let device = &mut self.state.project.devices[device_index];
                let before = json!(device.profinet_device_name.clone());
                device.profinet_device_name = Some(profinet_device_name.clone());
                let after = json!(device.profinet_device_name.clone());
                Ok(MutationResult {
                    touched_objects: vec![TouchedObject {
                        object: device.object.clone(),
                        changes: vec![FieldChange {
                            field: "profinet_device_name".to_string(),
                            before,
                            after: after.clone(),
                        }],
                    }],
                    verification: verification_from_fields(
                        "read_back",
                        vec![VerifiedField {
                            field: "profinet_device_name".to_string(),
                            expected: json!(profinet_device_name),
                            actual: after,
                        }],
                        None,
                    ),
                })
            }
        }
    }

    pub(super) fn write_network_config_impl(
        &mut self,
        params: WriteNetworkConfigParams,
    ) -> std::result::Result<MutationResult, BackendError> {
        self.require_connected()?;
        let network_index = self.network_index(&params.network_object_id)?;
        match params.operation {
            NetworkConfigOperation::RenameNetwork { new_name } => {
                Self::ensure_unique_name(
                    self.state.project.networks.iter().enumerate().filter_map(
                        |(index, network)| {
                            if index == network_index {
                                None
                            } else {
                                Some(network.object.name.clone())
                            }
                        },
                    ),
                    &new_name,
                    "network",
                )?;
                let network = &mut self.state.project.networks[network_index];
                let before_name = network.object.name.clone();
                let before_path = network.object.path.clone();
                network.object.name = new_name.clone();
                network.object.path =
                    Self::replace_terminal_path_segment(&network.object.path, &new_name);
                Ok(MutationResult {
                    touched_objects: vec![TouchedObject {
                        object: network.object.clone(),
                        changes: vec![
                            FieldChange {
                                field: "name".to_string(),
                                before: json!(before_name),
                                after: json!(network.object.name),
                            },
                            FieldChange {
                                field: "path".to_string(),
                                before: json!(before_path),
                                after: json!(network.object.path),
                            },
                        ],
                    }],
                    verification: verification_from_fields(
                        "read_back",
                        vec![
                            VerifiedField {
                                field: "name".to_string(),
                                expected: json!(new_name),
                                actual: json!(network.object.name),
                            },
                            VerifiedField {
                                field: "path".to_string(),
                                expected: json!(network.object.path),
                                actual: json!(network.object.path),
                            },
                        ],
                        None,
                    ),
                })
            }
            NetworkConfigOperation::SetConnectedObjects {
                connected_object_ids,
            } => {
                for object_id in &connected_object_ids {
                    if self.object_summary(object_id).is_none() {
                        return Err(BackendError::with_details(
                            "object_not_found",
                            "network participants must reference known engineering objects",
                            json!({ "network_object_id": params.network_object_id, "object_id": object_id }),
                        ));
                    }
                }
                let network = &mut self.state.project.networks[network_index];
                let before = json!(network.connected_object_ids.clone());
                network.connected_object_ids = connected_object_ids.clone();
                Ok(MutationResult {
                    touched_objects: vec![TouchedObject {
                        object: network.object.clone(),
                        changes: vec![FieldChange {
                            field: "connected_object_ids".to_string(),
                            before,
                            after: json!(network.connected_object_ids.clone()),
                        }],
                    }],
                    verification: verification_from_fields(
                        "read_back",
                        vec![
                            VerifiedField {
                                field: "connected_object_ids".to_string(),
                                expected: json!(connected_object_ids),
                                actual: json!(network.connected_object_ids.clone()),
                            },
                            VerifiedField {
                                field: "participant_count".to_string(),
                                expected: json!(network.connected_object_ids.len()),
                                actual: json!(network.connected_object_ids.len()),
                            },
                        ],
                        None,
                    ),
                })
            }
        }
    }

    pub(super) fn create_hmi_alarm_impl(
        &mut self,
        params: CreateHmiAlarmParams,
    ) -> std::result::Result<MutationResult, BackendError> {
        self.require_connected()?;
        let hmi_index = self.hmi_object_index(&params.hmi_object_id)?;
        let parent = self.state.project.hmi_objects[hmi_index].clone();
        if parent.summary.hmi_type.starts_with("alarm_") {
            return Err(BackendError::with_details(
                "unsupported_parent",
                "HMI alarms can only be created under a primary HMI object in the simulator backend",
                json!({ "hmi_object_id": params.hmi_object_id }),
            ));
        }
        let trigger_tag = self.resolve_tag_reference(&params.trigger_tag)?;
        Self::ensure_unique_name(
            self.state
                .project
                .hmi_objects
                .iter()
                .map(|hmi| hmi.summary.object.name.clone()),
            &params.name,
            "hmi_alarm",
        )?;
        let hmi_type = format!("alarm_{}", Self::hmi_alarm_severity_label(params.severity));
        let created = MockHmiObject {
            summary: HmiObjectSummary {
                object: EngineeringObjectSummary {
                    object_id: format!("hmi_alarm/{}", Self::slugify(&params.name)),
                    kind: "hmi_object".to_string(),
                    name: params.name.clone(),
                    path: format!("{}/Alarms/{}", parent.summary.object.path, params.name),
                },
                hmi_type: hmi_type.clone(),
            },
            trigger_tag: Some(trigger_tag.object_id.clone()),
            message: Some(params.message.clone()),
        };
        self.state.project.hmi_objects.push(created.clone());
        Ok(MutationResult {
            touched_objects: vec![TouchedObject {
                object: created.summary.object.clone(),
                changes: vec![
                    FieldChange {
                        field: "created".to_string(),
                        before: JsonValue::Null,
                        after: json!(true),
                    },
                    FieldChange {
                        field: "trigger_tag_object_id".to_string(),
                        before: JsonValue::Null,
                        after: json!(created.trigger_tag.clone()),
                    },
                    FieldChange {
                        field: "message".to_string(),
                        before: JsonValue::Null,
                        after: json!(created.message.clone()),
                    },
                ],
            }],
            verification: verification_from_fields(
                "read_back",
                vec![
                    VerifiedField {
                        field: "name".to_string(),
                        expected: json!(params.name),
                        actual: json!(created.summary.object.name),
                    },
                    VerifiedField {
                        field: "trigger_tag_object_id".to_string(),
                        expected: json!(trigger_tag.object_id),
                        actual: json!(created.trigger_tag.clone()),
                    },
                    VerifiedField {
                        field: "message".to_string(),
                        expected: json!(params.message),
                        actual: json!(created.message.clone()),
                    },
                    VerifiedField {
                        field: "hmi_type".to_string(),
                        expected: json!(hmi_type),
                        actual: json!(created.summary.hmi_type),
                    },
                ],
                None,
            ),
        })
    }

    pub(super) fn create_technology_object_impl(
        &mut self,
        params: CreateTechnologyObjectParams,
    ) -> std::result::Result<MutationResult, BackendError> {
        self.require_connected()?;
        let plc_index = self.plc_index(&params.plc_software_id)?;
        let plc = &mut self.state.project.plc_software[plc_index];
        Self::ensure_unique_name(
            plc.technology_objects
                .iter()
                .map(|technology_object| technology_object.object.name.clone()),
            &params.name,
            "technology_object",
        )?;
        let created = TechnologyObjectSummary {
            object: EngineeringObjectSummary {
                object_id: format!("technology/{}", Self::slugify(&params.name)),
                kind: "technology_object".to_string(),
                name: params.name.clone(),
                path: format!(
                    "{}/Technology objects/{}",
                    plc.summary.object.path, params.name
                ),
            },
            technology_type: params.technology_type.clone(),
            bound_axis: params.bound_axis.clone(),
        };
        plc.technology_objects.push(created.clone());
        Ok(MutationResult {
            touched_objects: vec![TouchedObject {
                object: created.object.clone(),
                changes: vec![FieldChange {
                    field: "created".to_string(),
                    before: JsonValue::Null,
                    after: json!(true),
                }],
            }],
            verification: verification_from_fields(
                "read_back",
                vec![
                    VerifiedField {
                        field: "name".to_string(),
                        expected: json!(params.name),
                        actual: json!(created.object.name),
                    },
                    VerifiedField {
                        field: "technology_type".to_string(),
                        expected: json!(params.technology_type),
                        actual: json!(created.technology_type),
                    },
                    VerifiedField {
                        field: "bound_axis".to_string(),
                        expected: json!(params.bound_axis),
                        actual: json!(created.bound_axis),
                    },
                ],
                None,
            ),
        })
    }

    pub(super) fn create_safety_object_impl(
        &mut self,
        params: CreateSafetyObjectParams,
    ) -> std::result::Result<MutationResult, BackendError> {
        self.require_connected()?;
        let plc_index = self.plc_index(&params.plc_software_id)?;
        let plc = &mut self.state.project.plc_software[plc_index];
        Self::ensure_unique_name(
            plc.safety_objects
                .iter()
                .map(|safety_object| safety_object.object.name.clone()),
            &params.name,
            "safety_object",
        )?;
        let created = SafetyObjectSummary {
            object: EngineeringObjectSummary {
                object_id: format!("safety/{}", Self::slugify(&params.name)),
                kind: "safety_object".to_string(),
                name: params.name.clone(),
                path: format!(
                    "{}/Safety administration/{}",
                    plc.summary.object.path, params.name
                ),
            },
            safety_type: params.safety_type.clone(),
        };
        plc.safety_objects.push(created.clone());
        Ok(MutationResult {
            touched_objects: vec![TouchedObject {
                object: created.object.clone(),
                changes: vec![FieldChange {
                    field: "created".to_string(),
                    before: JsonValue::Null,
                    after: json!(true),
                }],
            }],
            verification: verification_from_fields(
                "read_back",
                vec![
                    VerifiedField {
                        field: "name".to_string(),
                        expected: json!(params.name),
                        actual: json!(created.object.name),
                    },
                    VerifiedField {
                        field: "safety_type".to_string(),
                        expected: json!(params.safety_type),
                        actual: json!(created.safety_type),
                    },
                ],
                None,
            ),
        })
    }

    pub(super) fn go_online_impl(
        &mut self,
        params: GoOnlineParams,
    ) -> std::result::Result<MutationResult, BackendError> {
        self.require_connected()?;
        let device_index = self.device_index(&params.device_object_id)?;
        let device = &mut self.state.project.devices[device_index];
        let before = json!(device.online_state.clone());
        let after_state = match params.mode.unwrap_or_default() {
            OnlineSessionMode::Monitor => "online_monitor",
            OnlineSessionMode::Commissioning => "online_commissioning",
        };
        device.online_state = Some(after_state.to_string());
        Ok(MutationResult {
            touched_objects: vec![TouchedObject {
                object: device.object.clone(),
                changes: vec![FieldChange {
                    field: "online_state".to_string(),
                    before,
                    after: json!(device.online_state.clone()),
                }],
            }],
            verification: verification_from_fields(
                "read_back",
                vec![VerifiedField {
                    field: "online_state".to_string(),
                    expected: json!(after_state),
                    actual: json!(device.online_state.clone()),
                }],
                None,
            ),
        })
    }

    pub(super) fn download_to_device_impl(
        &mut self,
        params: DownloadToDeviceParams,
    ) -> std::result::Result<MutationResult, BackendError> {
        self.require_connected()?;
        let device_index = self.device_index(&params.device_object_id)?;
        let plc_indices = self.linked_plc_indices_for_device(&params.device_object_id);
        if plc_indices.is_empty() {
            return Err(BackendError::with_details(
                "download_scope_unavailable",
                "the selected device has no linked PLC software in the mock project",
                json!({ "device_object_id": params.device_object_id }),
            ));
        }
        if let Some(block) = plc_indices
            .iter()
            .flat_map(|plc_index| self.state.project.plc_software[*plc_index].blocks.iter())
            .find(|block| block.block_body.contains("COMPILE_ERROR"))
        {
            return Err(BackendError::with_details(
                "download_blocked",
                "download blocked because the linked PLC software contains compile errors",
                json!({
                    "device_object_id": params.device_object_id,
                    "block_object_id": block.summary.object.object_id,
                }),
            ));
        }

        let allowed_object_ids = self.downloadable_object_ids_for_device(&params.device_object_id);
        let requested_object_ids = if let Some(object_ids) = params.object_ids.clone() {
            if let Some(object_id) = object_ids
                .iter()
                .find(|object_id| !allowed_object_ids.contains(*object_id))
            {
                return Err(BackendError::with_details(
                    "download_scope_mismatch",
                    "requested download scope contains objects that are not linked to the selected device",
                    json!({
                        "device_object_id": params.device_object_id,
                        "object_id": object_id,
                    }),
                ));
            }
            object_ids
        } else {
            plc_indices
                .iter()
                .map(|plc_index| {
                    self.state.project.plc_software[*plc_index]
                        .summary
                        .object
                        .object_id
                        .clone()
                })
                .collect()
        };

        let device = &mut self.state.project.devices[device_index];
        let before_object_ids = json!(device.downloaded_object_ids.clone());
        let before_mode = json!(device.last_download_mode.clone());
        let before_online_state = json!(device.online_state.clone());
        let download_mode = match params.download_mode.unwrap_or_default() {
            DownloadMode::HardwareAndSoftware => "hardware_and_software",
            DownloadMode::SoftwareOnly => "software_only",
        };
        let online_state = match params.post_download_online_action.unwrap_or_default() {
            PostDownloadOnlineAction::LeaveOffline => "offline",
            PostDownloadOnlineAction::GoOnline => "online_monitor",
        };
        device.downloaded_object_ids = requested_object_ids.clone();
        device.last_download_mode = Some(download_mode.to_string());
        device.online_state = Some(online_state.to_string());
        Ok(MutationResult {
            touched_objects: vec![TouchedObject {
                object: device.object.clone(),
                changes: vec![
                    FieldChange {
                        field: "downloaded_object_ids".to_string(),
                        before: before_object_ids,
                        after: json!(device.downloaded_object_ids.clone()),
                    },
                    FieldChange {
                        field: "last_download_mode".to_string(),
                        before: before_mode,
                        after: json!(device.last_download_mode.clone()),
                    },
                    FieldChange {
                        field: "online_state".to_string(),
                        before: before_online_state,
                        after: json!(device.online_state.clone()),
                    },
                ],
            }],
            verification: verification_from_fields(
                "read_back",
                vec![
                    VerifiedField {
                        field: "downloaded_object_ids".to_string(),
                        expected: json!(requested_object_ids),
                        actual: json!(device.downloaded_object_ids.clone()),
                    },
                    VerifiedField {
                        field: "last_download_mode".to_string(),
                        expected: json!(download_mode),
                        actual: json!(device.last_download_mode.clone()),
                    },
                    VerifiedField {
                        field: "online_state".to_string(),
                        expected: json!(online_state),
                        actual: json!(device.online_state.clone()),
                    },
                ],
                None,
            ),
        })
    }
}
