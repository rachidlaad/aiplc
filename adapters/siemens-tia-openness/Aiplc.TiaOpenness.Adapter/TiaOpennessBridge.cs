using Microsoft.Win32;
using System;
using System.Collections;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Security.Cryptography;
using System.Text;
using System.Xml.Linq;

namespace Aiplc.TiaOpenness.Adapter
{
    internal sealed class TiaOpennessBridge
    {
        private const string FallbackIdentifierPrefix = "fallback:";
        private readonly string _publicApiDirectoryOverride;
        private readonly string _portalVersionOverride;
        private readonly Dictionary<string, Assembly> _loadedAssemblies =
            new Dictionary<string, Assembly>(StringComparer.OrdinalIgnoreCase);

        private string _assemblyDirectory;
        private string _portalVersion;
        private object _tiaPortal;
        private object _project;
        private object _objectIdentifierProvider;

        internal TiaOpennessBridge(string[] args)
        {
            _publicApiDirectoryOverride =
                ReadOption(args, "--public-api-dir") ??
                Environment.GetEnvironmentVariable("CODEX_TIA_PUBLICAPI_DIR");
            _portalVersionOverride =
                ReadOption(args, "--portal-version") ??
                Environment.GetEnvironmentVariable("CODEX_TIA_PORTAL_VERSION");
            AppDomain.CurrentDomain.AssemblyResolve += ResolveAssembly;
        }

        internal Dictionary<string, object> HandleRequest(Dictionary<string, object> request)
        {
            var id = GetString(request, "id") ?? Guid.NewGuid().ToString("N");
            var action = GetRequiredString(request, "action");
            var parameters = GetDictionary(request, "params", required: false) ??
                new Dictionary<string, object>(StringComparer.OrdinalIgnoreCase);

            try
            {
                object result;
                switch (action)
                {
                    case "connect":
                        result = Connect(parameters);
                        break;
                    case "open_project":
                        result = OpenProject(parameters);
                        break;
                    case "project_overview":
                        result = ProjectOverview();
                        break;
                    case "list_blocks":
                        result = ListBlocks(parameters);
                        break;
                    case "list_tag_tables":
                        result = ListTagTables(parameters);
                        break;
                    case "list_data_types":
                        result = ListDataTypes(parameters);
                        break;
                    case "create_udt":
                        result = CreateUdt(parameters);
                        break;
                    case "edit_udt":
                        result = EditUdt(parameters);
                        break;
                    case "create_block":
                        result = CreateBlock(parameters);
                        break;
                    case "edit_block_body":
                        result = EditBlockBody(parameters);
                        break;
                    case "create_block_call":
                        result = CreateBlockCall(parameters);
                        break;
                    case "edit_db_members":
                        result = EditDbMembers(parameters);
                        break;
                    case "create_plc_tag":
                        result = CreatePlcTag(parameters);
                        break;
                    case "create_tag_table":
                        result = CreateTagTable(parameters);
                        break;
                    case "list_technology_objects":
                        result = ListTechnologyObjects(parameters);
                        break;
                    case "list_watch_tables":
                        result = ListWatchTables(parameters);
                        break;
                    case "list_networks":
                        result = ListNetworks(parameters);
                        break;
                    case "list_hmi_objects":
                        result = ListHmiObjects(parameters);
                        break;
                    case "list_safety_objects":
                        result = ListSafetyObjects(parameters);
                        break;
                    case "consistency_check":
                        result = ConsistencyCheck(parameters);
                        break;
                    case "cross_reference":
                        result = CrossReference(parameters);
                        break;
                    case "create_watch_table":
                        result = CreateWatchTable(parameters);
                        break;
                    case "write_hardware_config":
                        result = WriteHardwareConfig(parameters);
                        break;
                    case "write_network_config":
                        result = WriteNetworkConfig(parameters);
                        break;
                    case "create_hmi_alarm":
                        result = CreateHmiAlarm(parameters);
                        break;
                    case "create_technology_object":
                        result = CreateTechnologyObject(parameters);
                        break;
                    case "create_safety_object":
                        result = CreateSafetyObject(parameters);
                        break;
                    case "compare_online_offline":
                        result = CompareOnlineOffline(parameters);
                        break;
                    case "run_simulation":
                        result = RunSimulation(parameters);
                        break;
                    case "go_online":
                        result = GoOnline(parameters);
                        break;
                    case "download_to_device":
                        result = DownloadToDevice(parameters);
                        break;
                    case "export_object":
                        result = ExportObject(parameters);
                        break;
                    case "import_object":
                        result = ImportObject(parameters);
                        break;
                    case "apply_edit":
                        result = ApplyEdit(parameters);
                        break;
                    case "compile":
                        result = Compile(parameters);
                        break;
                    default:
                        throw new AdapterException(
                            "unknown_action",
                            "Unsupported adapter action.",
                            new Dictionary<string, object> { { "action", action } });
                }

                return Program.SuccessResponse(id, result);
            }
            catch (TargetInvocationException ex)
            {
                var inner = ex.InnerException ?? ex;
                return Failure(id, inner);
            }
            catch (Exception ex)
            {
                return Failure(id, ex);
            }
        }

        private Dictionary<string, object> Failure(string id, Exception ex)
        {
            var adapterException = ex as AdapterException;
            if (adapterException != null)
            {
                return Program.FailureResponse(
                    id,
                    adapterException.Code,
                    adapterException.Message,
                    adapterException.Details);
            }

            return Program.FailureResponse(
                id,
                "adapter_error",
                ex.Message,
                new Dictionary<string, object> { { "exception", ex.GetType().FullName } });
        }

        private object Connect(Dictionary<string, object> parameters)
        {
            var requestedVersion = GetString(parameters, "portal_version") ?? _portalVersionOverride;
            var requestedMode = GetString(parameters, "connection_mode") ?? "auto";
            var requestedUiMode = GetString(parameters, "ui_mode") ?? "with_ui";
            var requestedProcessId = GetNullableInt(parameters, "process_id");

            var install = ResolveInstall(requestedVersion);
            EnsureAssembliesLoaded(install);

            var tiaPortalType = FindType("Siemens.Engineering.TiaPortal");
            var processes = EnumerateObjects(InvokeStatic(tiaPortalType, "GetProcesses")).ToList();
            object process = null;
            if (requestedProcessId.HasValue)
            {
                process = processes.FirstOrDefault(
                    item => GetNullableInt(item, "Id") == requestedProcessId.Value);
            }
            else if (processes.Count > 0)
            {
                process = processes[0];
            }

            string origin;
            if (string.Equals(requestedMode, "launch", StringComparison.OrdinalIgnoreCase))
            {
                _tiaPortal = LaunchPortal(tiaPortalType, requestedUiMode);
                origin = "launched";
            }
            else if (string.Equals(requestedMode, "attach", StringComparison.OrdinalIgnoreCase))
            {
                if (process == null)
                {
                    throw new AdapterException(
                        "no_running_portal",
                        "No running TIA Portal process matched the attach request.",
                        new Dictionary<string, object> { { "portal_version", requestedVersion } });
                }

                _tiaPortal = InvokeMethod(process, "Attach");
                origin = "attached";
            }
            else
            {
                if (process != null)
                {
                    _tiaPortal = InvokeMethod(process, "Attach");
                    origin = "attached";
                }
                else
                {
                    _tiaPortal = LaunchPortal(tiaPortalType, requestedUiMode);
                    origin = "launched";
                }
            }

            AttachOpenProjectIfPresent();

            return new Dictionary<string, object>
            {
                { "backend", "subprocess" },
                { "portal_version", _portalVersion },
                { "origin", origin },
                { "process_id", GetNullableInt(_tiaPortal, "ProcessId") ?? GetNullableInt(_tiaPortal, "Id") },
                { "ui_mode", requestedUiMode },
                { "project_open", _project != null },
                { "processes", processes.Select(DescribeProcess).ToList() },
            };
        }

        private object OpenProject(Dictionary<string, object> parameters)
        {
            EnsureConnected();
            var projectPath = GetRequiredString(parameters, "project_path");
            var projects = GetPropertyValue(_tiaPortal, "Projects");
            _project = InvokeMethod(projects, "Open", new FileInfo(projectPath));
            RefreshObjectIdentifierProvider();
            return ProjectOverview();
        }

        private object ProjectOverview()
        {
            EnsureProjectOpen();

            var devices = EnumerateObjects(GetPropertyValue(_project, "Devices")).ToList();
            var plcSoftware = new List<Dictionary<string, object>>();
            var deviceSummaries = new List<Dictionary<string, object>>();

            foreach (var device in devices)
            {
                var deviceItems = FlattenDeviceItems(device).ToList();
                deviceSummaries.Add(
                    new Dictionary<string, object>
                    {
                        { "object", DescribeEngineeringObject(device) },
                        { "type_identifier", GetString(device, "TypeIdentifier") },
                        {
                            "device_items",
                            deviceItems.Select(
                                item => new Dictionary<string, object>
                                {
                                    { "object", DescribeEngineeringObject(item) },
                                    { "classification", ClassifyDeviceItem(item) },
                                }).ToList()
                        },
                    });

                foreach (var deviceItem in deviceItems)
                {
                    var software = TryGetPlcSoftware(deviceItem);
                    if (software == null)
                    {
                        continue;
                    }

                    plcSoftware.Add(
                        new Dictionary<string, object>
                        {
                            { "object", DescribeEngineeringObject(software) },
                            { "device_id", GetObjectIdentifier(device) },
                            { "device_name", ReadName(device) },
                            { "block_group_object_id", GetObjectIdentifier(GetPropertyValue(software, "BlockGroup")) },
                            { "tag_table_group_object_id", GetObjectIdentifier(GetPropertyValue(software, "TagTableGroup")) },
                        });
                }
            }

            return new Dictionary<string, object>
            {
                {
                    "project",
                    new Dictionary<string, object>
                    {
                        { "object", DescribeEngineeringObject(_project) },
                        { "project_path", GetString(_project, "Path") ?? GetString(_project, "ProjectPath") },
                        { "portal_version", _portalVersion },
                    }
                },
                { "devices", deviceSummaries },
                { "plc_software", plcSoftware },
            };
        }

        private object ListBlocks(Dictionary<string, object> parameters)
        {
            EnsureProjectOpen();
            var plcSoftware = ResolveObject(GetRequiredString(parameters, "plc_software_id"));
            var traversalMode = GetString(parameters, "traversal_mode") ?? "recursive";
            var blocks = new List<Dictionary<string, object>>();
            WalkBlockGroup(GetPropertyValue(plcSoftware, "BlockGroup"), blocks, traversalMode);

            return new Dictionary<string, object>
            {
                { "plc_software_id", GetRequiredString(parameters, "plc_software_id") },
                { "blocks", blocks },
            };
        }

        private object ListTagTables(Dictionary<string, object> parameters)
        {
            EnsureProjectOpen();
            var plcSoftware = ResolveObject(GetRequiredString(parameters, "plc_software_id"));
            var detailLevel = GetString(parameters, "detail_level") ?? "tables_only";
            var traversalMode = GetString(parameters, "traversal_mode") ?? "recursive";
            var includeTags = string.Equals(detailLevel, "include_tags", StringComparison.OrdinalIgnoreCase);

            var tagTables = new List<Dictionary<string, object>>();
            WalkTagTableGroup(GetPropertyValue(plcSoftware, "TagTableGroup"), tagTables, traversalMode, includeTags);

            return new Dictionary<string, object>
            {
                { "plc_software_id", GetRequiredString(parameters, "plc_software_id") },
                { "tag_tables", tagTables },
            };
        }

        private object ListDataTypes(Dictionary<string, object> parameters)
        {
            EnsureProjectOpen();
            var plcSoftwareId = GetRequiredString(parameters, "plc_software_id");
            var plcSoftware = ResolveObject(plcSoftwareId);
            var root = FindDataTypeRoot(plcSoftware);
            var dataTypes = new List<Dictionary<string, object>>();
            var seen = new HashSet<string>(StringComparer.Ordinal);
            if (root != null)
            {
                WalkDataTypeGroup(root, dataTypes, seen);
            }

            return new Dictionary<string, object>
            {
                { "plc_software_id", plcSoftwareId },
                { "data_types", dataTypes },
            };
        }

        private object CreateUdt(Dictionary<string, object> parameters)
        {
            EnsureProjectOpen();
            var plcSoftware = ResolveObject(GetRequiredString(parameters, "plc_software_id"));
            var name = GetRequiredString(parameters, "name");
            EnsureNamedObjectDoesNotExist(EnumerateDataTypeObjects(FindDataTypeRoot(plcSoftware)), name, "plc_data_type");

            var members = GetList(parameters, "members", required: true);
            var generatedObjects = GenerateObjectsFromSourceText(
                plcSoftware,
                name,
                ".udt",
                BuildUdtSource(name, members));
            var created = FindNamedGeneratedObject(generatedObjects, name) ?? FindDataTypeByName(plcSoftware, name);
            if (created == null)
            {
                throw new AdapterException(
                    "verification_failed",
                    "UDT source generation completed but the created data type could not be found by read-back.",
                    new Dictionary<string, object>
                    {
                        { "name", name },
                        { "plc_software", DescribeEngineeringObject(plcSoftware) },
                    });
            }

            var changes = new List<Dictionary<string, object>>
            {
                new Dictionary<string, object>
                {
                    { "field", "created" },
                    { "before", null },
                    { "after", true },
                },
            };
            var verifiedFields = new List<Dictionary<string, object>>
            {
                new Dictionary<string, object>
                {
                    { "field", "name" },
                    { "expected", name },
                    { "actual", ReadName(created) },
                },
            };

            MaybeApplyScalarEdit(created, "Comment", GetString(parameters, "comment"), changes, verifiedFields);

            var described = DescribeDataType(created);
            var readBackMembers = (List<Dictionary<string, object>>)described["members"];
            changes.Add(
                new Dictionary<string, object>
                {
                    { "field", "member_count" },
                    { "before", 0 },
                    { "after", readBackMembers.Count },
                });
            verifiedFields.Add(
                new Dictionary<string, object>
                {
                    { "field", "member_count" },
                    { "expected", members.Count },
                    { "actual", readBackMembers.Count },
                });
            verifiedFields.Add(
                new Dictionary<string, object>
                {
                    { "field", "member_names" },
                    { "expected", JoinRequestedMemberNames(members) },
                    { "actual", JoinReadBackMemberNames(readBackMembers) },
                });

            return BuildVerifiedMutationResponse(created, changes, verifiedFields);
        }

        private object EditUdt(Dictionary<string, object> parameters)
        {
            EnsureProjectOpen();
            var target = ResolveObject(GetRequiredString(parameters, "object_id"));
            var plcSoftware = ResolveOwningPlcSoftware(target);
            if (plcSoftware == null)
            {
                throw new AdapterException(
                    "unsupported_live_operation",
                    "The selected data type is not associated with a PLC software root.",
                    DescribeEngineeringObject(target));
            }

            var changes = new List<Dictionary<string, object>>();
            var verifiedFields = new List<Dictionary<string, object>>();
            var currentName = ReadName(target);
            var requestedName = GetString(parameters, "new_name");
            if (!string.IsNullOrWhiteSpace(requestedName) &&
                !string.Equals(currentName, requestedName, StringComparison.OrdinalIgnoreCase))
            {
                EnsureNamedObjectDoesNotExist(
                    EnumerateDataTypeObjects(FindDataTypeRoot(plcSoftware)).Where(
                        candidate => !string.Equals(
                            GetObjectIdentifier(candidate),
                            GetObjectIdentifier(target),
                            StringComparison.OrdinalIgnoreCase)),
                    requestedName,
                    "plc_data_type");
                ApplyScalarEdit(target, "Name", requestedName, changes, verifiedFields);
                currentName = requestedName;
            }

            MaybeApplyScalarEdit(target, "Comment", GetString(parameters, "comment"), changes, verifiedFields);

            var requestedMembers = GetList(parameters, "members", required: false);
            if (requestedMembers != null)
            {
                var beforeMembers = DescribeDataTypeMembers(target);
                var regeneratedObjects = GenerateObjectsFromSourceText(
                    plcSoftware,
                    currentName,
                    ".udt",
                    BuildUdtSource(currentName, requestedMembers));
                target = FindNamedGeneratedObject(regeneratedObjects, currentName) ??
                    FindDataTypeByName(plcSoftware, currentName) ??
                    target;
                var afterMembers = DescribeDataTypeMembers(target);
                changes.Add(
                    new Dictionary<string, object>
                    {
                        { "field", "members" },
                        { "before", beforeMembers },
                        { "after", afterMembers },
                    });
                verifiedFields.Add(
                    new Dictionary<string, object>
                    {
                        { "field", "member_count" },
                        { "expected", requestedMembers.Count },
                        { "actual", afterMembers.Count },
                    });
                verifiedFields.Add(
                    new Dictionary<string, object>
                    {
                        { "field", "member_names" },
                        { "expected", JoinRequestedMemberNames(requestedMembers) },
                        { "actual", JoinReadBackMemberNames(afterMembers) },
                    });
            }

            return BuildVerifiedMutationResponse(target, changes, verifiedFields);
        }

        private object CreateBlock(Dictionary<string, object> parameters)
        {
            EnsureProjectOpen();
            var plcSoftware = ResolveObject(GetRequiredString(parameters, "plc_software_id"));
            var name = GetRequiredString(parameters, "name");
            var blockKind = GetRequiredString(parameters, "block_kind");
            var language = GetString(parameters, "language");
            var blockBody = GetString(parameters, "block_body");

            EnsureNamedObjectDoesNotExist(EnumerateBlockGroupObjects(GetPropertyValue(plcSoftware, "BlockGroup")), name, "block");

            object created;
            if (string.Equals(blockKind, "instance_db", StringComparison.OrdinalIgnoreCase))
            {
                throw new AdapterException(
                    "unsupported_live_operation",
                    "Live instance DB creation requires the source FB name, but create_block does not expose that parameter. Use create_block_call when an FB instance DB is required.",
                    new Dictionary<string, object>
                    {
                        { "name", name },
                        { "block_kind", blockKind },
                    });
            }

            if (ShouldUseDirectFbCreate(blockKind, language, blockBody))
            {
                created = InvokeCompatibleMethod(
                    ResolveBlockComposition(plcSoftware),
                    "CreateFB",
                    name,
                    true,
                    0,
                    ResolveProgrammingLanguage(language));
            }
            else
            {
                created = FindNamedGeneratedObject(
                    GenerateObjectsFromSourceText(
                        plcSoftware,
                        name,
                        InferSourceExtensionForBlockKind(blockKind),
                        BuildBlockSource(blockKind, name, blockBody)),
                    name) ?? FindBlockByName(plcSoftware, name);
            }

            if (created == null)
            {
                throw new AdapterException(
                    "verification_failed",
                    "Block creation completed but the created block could not be found by read-back.",
                    new Dictionary<string, object>
                    {
                        { "name", name },
                        { "block_kind", blockKind },
                        { "plc_software", DescribeEngineeringObject(plcSoftware) },
                    });
            }

            var changes = new List<Dictionary<string, object>>
            {
                new Dictionary<string, object>
                {
                    { "field", "created" },
                    { "before", null },
                    { "after", true },
                },
            };
            var described = DescribeBlock(created);
            changes.Add(
                new Dictionary<string, object>
                {
                    { "field", "block_type" },
                    { "before", null },
                    { "after", described["block_type"] },
                });

            var verifiedFields = new List<Dictionary<string, object>>
            {
                new Dictionary<string, object>
                {
                    { "field", "name" },
                    { "expected", name },
                    { "actual", ReadName(created) },
                },
            };

            MaybeApplyScalarEdit(created, "HeaderAuthor", GetString(parameters, "header_author"), changes, verifiedFields);
            MaybeApplyScalarEdit(created, "HeaderFamily", GetString(parameters, "header_family"), changes, verifiedFields);
            MaybeApplyScalarEdit(created, "HeaderName", GetString(parameters, "header_name"), changes, verifiedFields);
            MaybeApplyScalarEdit(created, "HeaderVersion", GetString(parameters, "header_version"), changes, verifiedFields);

            return BuildVerifiedMutationResponse(created, changes, verifiedFields);
        }

        private object EditBlockBody(Dictionary<string, object> parameters)
        {
            EnsureProjectOpen();
            var target = ResolveObject(GetRequiredString(parameters, "object_id"));
            if (IsDataBlock(target))
            {
                throw new AdapterException(
                    "unsupported_block_kind",
                    "edit_block_body only supports executable code blocks in the live TIA adapter.",
                    new Dictionary<string, object>
                    {
                        { "object", DescribeEngineeringObject(target) },
                        { "block_type", target.GetType().Name },
                    });
            }

            var requestedLanguage = GetString(parameters, "language");
            if (!string.IsNullOrWhiteSpace(requestedLanguage) &&
                !string.Equals(requestedLanguage, "scl", StringComparison.OrdinalIgnoreCase))
            {
                throw new AdapterException(
                    "unsupported_live_operation",
                    "Live block-body replacement currently supports SCL source bodies only.",
                    new Dictionary<string, object>
                    {
                        { "language", requestedLanguage },
                        { "object", DescribeEngineeringObject(target) },
                    });
            }

            var plcSoftware = ResolveOwningPlcSoftware(target);
            var targetName = ReadName(target);
            var beforeSource = GenerateSourceText(plcSoftware, new[] { target });
            var beforeBody = ExtractBlockBodyFromSource(beforeSource);
            var requestedBody = GetRequiredString(parameters, "block_body");
            target = FindNamedGeneratedObject(
                GenerateObjectsFromSourceText(
                    plcSoftware,
                    targetName,
                    ".scl",
                    ReplaceBlockBodyInSource(beforeSource, requestedBody)),
                targetName) ?? FindBlockByName(plcSoftware, targetName) ?? target;

            var changes = new List<Dictionary<string, object>>
            {
                new Dictionary<string, object>
                {
                    { "field", "block_body" },
                    { "before", beforeBody },
                    { "after", requestedBody },
                },
            };
            var verifiedFields = new List<Dictionary<string, object>>();

            var afterBody = ExtractBlockBodyFromSource(GenerateSourceText(plcSoftware, new[] { target }));
            verifiedFields.Add(
                new Dictionary<string, object>
                {
                    { "field", "block_body" },
                    { "expected", NormalizeSourceBodyText(requestedBody) },
                    { "actual", NormalizeSourceBodyText(afterBody) },
                });

            return BuildVerifiedMutationResponse(target, changes, verifiedFields);
        }

        private object CreateBlockCall(Dictionary<string, object> parameters)
        {
            EnsureProjectOpen();
            var caller = ResolveObject(GetRequiredString(parameters, "caller_block_id"));
            var callee = ResolveObject(GetRequiredString(parameters, "callee_block_id"));
            var callerName = ReadName(caller);
            var calleeName = ReadName(callee);
            var callerPlc = ResolveOwningPlcSoftware(caller);
            var calleePlc = ResolveOwningPlcSoftware(callee);
            if (callerPlc == null ||
                calleePlc == null ||
                !string.Equals(GetObjectIdentifier(callerPlc), GetObjectIdentifier(calleePlc), StringComparison.OrdinalIgnoreCase))
            {
                throw new AdapterException(
                    "cross_plc_call_unsupported",
                    "Live block-call creation requires caller and callee to belong to the same PLC software root.",
                    new Dictionary<string, object>
                    {
                        { "caller", DescribeEngineeringObject(caller) },
                        { "callee", DescribeEngineeringObject(callee) },
                    });
            }

            var touchedObjects = new List<Dictionary<string, object>>();
            var instanceDbName = GetString(parameters, "instance_db_name");
            if (string.Equals(callee.GetType().Name, "FB", StringComparison.OrdinalIgnoreCase) &&
                string.IsNullOrWhiteSpace(instanceDbName))
            {
                throw new AdapterException(
                    "missing_parameter",
                    "Creating a live call to an FB requires instance_db_name so the adapter can create or verify the instance DB deterministically.",
                    new Dictionary<string, object> { { "parameter", "instance_db_name" } });
            }

            if (!string.IsNullOrWhiteSpace(instanceDbName))
            {
                var instanceDb = FindBlockByName(callerPlc, instanceDbName);
                if (instanceDb == null)
                {
                    instanceDb = EnsureInstanceDbExists(callerPlc, instanceDbName, calleeName);
                    touchedObjects.Add(
                        new Dictionary<string, object>
                        {
                            { "object", DescribeEngineeringObject(instanceDb) },
                            {
                                "changes",
                                new List<Dictionary<string, object>>
                                {
                                    new Dictionary<string, object>
                                    {
                                        { "field", "created" },
                                        { "before", null },
                                        { "after", true },
                                    },
                                    new Dictionary<string, object>
                                    {
                                        { "field", "number" },
                                        { "before", null },
                                        { "after", GetNullableInt(instanceDb, "Number") ?? GetNullableIntAttribute(instanceDb, "Number") },
                                    },
                                }
                            },
                        });
                }

                var instanceDbNumber = GetNullableInt(instanceDb, "Number") ?? GetNullableIntAttribute(instanceDb, "Number");
                if (instanceDbNumber == null || instanceDbNumber <= 0)
                {
                    throw new AdapterException(
                        "verification_failed",
                        "Instance DB creation completed but the resulting DB number is invalid.",
                        new Dictionary<string, object>
                        {
                            { "instance_db", DescribeEngineeringObject(instanceDb) },
                            { "number", instanceDbNumber },
                        });
                }
            }

            var callerSource = GenerateSourceText(callerPlc, new[] { caller });
            var beforeBody = ExtractBlockBodyFromSource(callerSource);
            var afterBody = AppendBlockCallToBody(
                beforeBody,
                BuildBlockCallStatement(calleeName, instanceDbName, GetString(parameters, "comment"), GetList(parameters, "parameter_bindings", required: true)));
            caller = FindNamedGeneratedObject(
                GenerateObjectsFromSourceText(
                    callerPlc,
                    callerName,
                    ".scl",
                    ReplaceBlockBodyInSource(callerSource, afterBody)),
                callerName) ?? FindBlockByName(callerPlc, callerName) ?? caller;

            touchedObjects.Insert(
                0,
                new Dictionary<string, object>
                {
                    { "object", DescribeEngineeringObject(caller) },
                    {
                        "changes",
                        new List<Dictionary<string, object>>
                        {
                            new Dictionary<string, object>
                            {
                                { "field", "block_body" },
                                { "before", beforeBody },
                                { "after", afterBody },
                            },
                        }
                    },
                });

            var readBackBody = ExtractBlockBodyFromSource(GenerateSourceText(callerPlc, new[] { caller }));
            var verificationField = string.IsNullOrWhiteSpace(instanceDbName)
                ? QuotePlcIdentifier(calleeName)
                : QuotePlcIdentifier(instanceDbName);
            var verifiedFields = new List<Dictionary<string, object>>
            {
                new Dictionary<string, object>
                {
                    { "field", "caller_body_contains_call" },
                    { "expected", true },
                    { "actual", readBackBody.IndexOf(verificationField, StringComparison.OrdinalIgnoreCase) >= 0 },
                },
            };
            if (!verifiedFields.All(field => Equals(field["expected"], field["actual"])))
            {
                throw new AdapterException(
                    "verification_failed",
                    "Block-call insertion completed but the regenerated caller body does not contain the expected call target.",
                    new Dictionary<string, object>
                    {
                        { "caller", DescribeEngineeringObject(caller) },
                        { "callee", DescribeEngineeringObject(callee) },
                    });
            }

            return new Dictionary<string, object>
            {
                { "touched_objects", touchedObjects },
                {
                    "verification",
                    new Dictionary<string, object>
                    {
                        { "verified", true },
                        { "strategy", "read_back" },
                        { "checked_fields", verifiedFields },
                        { "exported_sha256", null },
                    }
                },
            };
        }

        private object EditDbMembers(Dictionary<string, object> parameters)
        {
            EnsureProjectOpen();
            var target = ResolveObject(GetRequiredString(parameters, "object_id"));
            if (!IsDataBlock(target))
            {
                throw new AdapterException(
                    "unsupported_block_kind",
                    "edit_db_members only supports global and instance DB blocks in the live TIA adapter.",
                    new Dictionary<string, object>
                    {
                        { "object", DescribeEngineeringObject(target) },
                        { "block_type", target.GetType().Name },
                    });
            }

            var requestedMembers = GetList(parameters, "members", required: true);
            var beforeMembers = DescribeDbMembers(target);
            var effectiveMembers = MergeDbMembers(beforeMembers, requestedMembers, GetNullableBoolean(parameters, "replace_existing") ?? false);
            var plcSoftware = ResolveOwningPlcSoftware(target);
            var targetName = ReadName(target);
            target = FindNamedGeneratedObject(
                GenerateObjectsFromSourceText(
                    plcSoftware,
                    targetName,
                    ".db",
                    ReplaceDbMembersInSource(
                        GenerateSourceText(plcSoftware, new[] { target }),
                        effectiveMembers)),
                targetName) ?? FindBlockByName(plcSoftware, targetName) ?? target;
            var afterMembers = DescribeDbMembers(target);

            var changes = new List<Dictionary<string, object>>
            {
                new Dictionary<string, object>
                {
                    { "field", "db_members" },
                    { "before", beforeMembers },
                    { "after", afterMembers },
                },
            };
            var verifiedFields = new List<Dictionary<string, object>>
            {
                new Dictionary<string, object>
                {
                    { "field", "member_count" },
                    { "expected", effectiveMembers.Count },
                    { "actual", afterMembers.Count },
                },
                new Dictionary<string, object>
                {
                    { "field", "member_names" },
                    { "expected", JoinRequestedMemberNames(effectiveMembers.Cast<object>().ToList()) },
                    { "actual", JoinReadBackMemberNames(afterMembers) },
                },
            };

            return BuildVerifiedMutationResponse(target, changes, verifiedFields);
        }

        private object CreateTagTable(Dictionary<string, object> parameters)
        {
            EnsureProjectOpen();
            var plcSoftware = ResolveObject(GetRequiredString(parameters, "plc_software_id"));
            var name = GetRequiredString(parameters, "name");
            var tagTableGroup = GetPropertyValue(plcSoftware, "TagTableGroup");
            var composition = GetPropertyValue(tagTableGroup, "TagTables");
            var created = CreateObjectInComposition(
                composition,
                name,
                new object[] { name });
            var changes = new List<Dictionary<string, object>>
            {
                new Dictionary<string, object>
                {
                    { "field", "created" },
                    { "before", null },
                    { "after", true },
                },
            };
            var verifiedFields = new List<Dictionary<string, object>>
            {
                new Dictionary<string, object>
                {
                    { "field", "name" },
                    { "expected", name },
                    { "actual", ReadName(created) },
                },
            };
            return BuildVerifiedMutationResponse(created, changes, verifiedFields);
        }

        private object CreatePlcTag(Dictionary<string, object> parameters)
        {
            EnsureProjectOpen();
            var tagTable = ResolveObject(GetRequiredString(parameters, "tag_table_object_id"));
            var name = GetRequiredString(parameters, "name");
            var dataTypeName = GetRequiredString(parameters, "data_type_name");
            var plcSoftware = ResolveOwningPlcSoftware(tagTable);
            if (plcSoftware != null)
            {
                EnsureNamedObjectDoesNotExist(
                    EnumerateTagTableGroupObjects(GetPropertyValue(plcSoftware, "TagTableGroup"))
                        .Where(candidate => TryGetPropertyValue(candidate, "DataTypeName") != null || TypeNameContains(candidate, "PlcTag")),
                    name,
                    "plc_tag");
            }
            var tags = GetPropertyValue(tagTable, "Tags");
            var created = CreateObjectInComposition(
                tags,
                name,
                new object[] { name, dataTypeName, GetString(parameters, "logical_address") },
                new object[] { name, dataTypeName },
                new object[] { name });
            var changes = new List<Dictionary<string, object>>
            {
                new Dictionary<string, object>
                {
                    { "field", "created" },
                    { "before", null },
                    { "after", true },
                },
            };
            var verifiedFields = new List<Dictionary<string, object>>
            {
                new Dictionary<string, object>
                {
                    { "field", "name" },
                    { "expected", name },
                    { "actual", ReadName(created) },
                },
            };

            EnsureFieldValue(created, "DataTypeName", dataTypeName, changes, verifiedFields);
            EnsureFieldValue(created, "LogicalAddress", GetString(parameters, "logical_address"), changes, verifiedFields);
            EnsureFieldValue(created, "ExternalAccessible", GetNullableBoolean(parameters, "external_accessible"), changes, verifiedFields);
            EnsureFieldValue(created, "ExternalVisible", GetNullableBoolean(parameters, "external_visible"), changes, verifiedFields);
            EnsureFieldValue(created, "ExternalWritable", GetNullableBoolean(parameters, "external_writable"), changes, verifiedFields);

            return BuildVerifiedMutationResponse(created, changes, verifiedFields);
        }

        private object CreateWatchTable(Dictionary<string, object> parameters)
        {
            EnsureProjectOpen();
            var plcSoftware = ResolveObject(GetRequiredString(parameters, "plc_software_id"));
            var name = GetRequiredString(parameters, "name");
            var expressions = GetList(parameters, "expressions", required: false);
            var root = FindWatchTableRoot(plcSoftware);
            var composition = GetPropertyValue(root, "WatchTables");
            if (expressions != null && expressions.Count > 0)
            {
                return ImportWatchTableWithExpressions(composition, name, expressions);
            }

            var created = CreateObjectInComposition(composition, name, new object[] { name });
            var changes = new List<Dictionary<string, object>>
            {
                new Dictionary<string, object>
                {
                    { "field", "created" },
                    { "before", null },
                    { "after", true },
                },
            };
            var verifiedFields = new List<Dictionary<string, object>>
            {
                new Dictionary<string, object>
                {
                    { "field", "name" },
                    { "expected", name },
                    { "actual", ReadName(created) },
                },
            };

            AddWatchExpressionVerification(created, expressions, verifiedFields);

            if (expressions != null)
            {
                changes.Add(
                    new Dictionary<string, object>
                    {
                        { "field", "expression_count" },
                        { "before", 0 },
                        { "after", expressions.Count },
                    });
            }

            return BuildVerifiedMutationResponse(created, changes, verifiedFields);
        }

        private object ImportWatchTableWithExpressions(
            object composition,
            string name,
            IList<object> expressions)
        {
            var importPath = Path.Combine(
                Path.GetTempPath(),
                "codex-tia-watch-imports",
                Guid.NewGuid().ToString("N") + ".xml");
            Directory.CreateDirectory(Path.GetDirectoryName(importPath));
            try
            {
                BuildWatchTableImportDocument(name, expressions).Save(importPath);
                var imported = InvokeImport(composition, importPath, "none");
                var created = ResolveImportedWatchTable(imported, composition, name);
                if (created == null)
                {
                    throw new AdapterException(
                        "verification_failed",
                        "Watch-table import completed but the imported table could not be resolved by name.",
                        new Dictionary<string, object> { { "name", name } });
                }

                var verifiedFields = new List<Dictionary<string, object>>
                {
                    new Dictionary<string, object>
                    {
                        { "field", "name" },
                        { "expected", name },
                        { "actual", ReadName(created) },
                    },
                };
                AddWatchExpressionVerification(created, expressions, verifiedFields);

                return BuildVerifiedMutationResponse(
                    created,
                    new List<Dictionary<string, object>>
                    {
                        new Dictionary<string, object>
                        {
                            { "field", "created" },
                            { "before", null },
                            { "after", true },
                        },
                        new Dictionary<string, object>
                        {
                            { "field", "expression_count" },
                            { "before", 0 },
                            { "after", expressions.Count },
                        },
                    },
                    verifiedFields);
            }
            finally
            {
                if (File.Exists(importPath))
                {
                    File.Delete(importPath);
                }
            }
        }

        private XDocument BuildWatchTableImportDocument(string name, IList<object> expressions)
        {
            var nextId = 1;
            var objectList = new XElement("ObjectList");
            foreach (var expressionItem in expressions)
            {
                var expression = expressionItem as Dictionary<string, object>;
                if (expression == null)
                {
                    continue;
                }

                objectList.Add(
                    new XElement(
                        "SW.WatchAndForceTables.PlcWatchTableEntry",
                        new XAttribute("ID", nextId.ToString(CultureInfo.InvariantCulture)),
                        new XAttribute("CompositionName", "Entries"),
                        new XElement(
                            "AttributeList",
                            new XElement("Address", GetRequiredString(expression, "expression")))));
                nextId++;
            }

            return new XDocument(
                new XDeclaration("1.0", "utf-8", null),
                new XElement(
                    "Document",
                    new XElement("Engineering", new XAttribute("version", _portalVersion ?? "V21")),
                    new XElement(
                        "DocumentInfo",
                        new XElement("Created", DateTime.UtcNow.ToString("O", CultureInfo.InvariantCulture)),
                        new XElement("ExportSetting", "WithDefaults")),
                    new XElement(
                        "SW.WatchAndForceTables.PlcWatchTable",
                        new XAttribute("ID", "0"),
                        new XElement("AttributeList", new XElement("Name", name)),
                        objectList)));
        }

        private object ResolveImportedWatchTable(object imported, object composition, string name)
        {
            if (imported != null && string.Equals(ReadNameOrNull(imported), name, StringComparison.OrdinalIgnoreCase))
            {
                return imported;
            }

            foreach (var item in EnumerateObjects(imported))
            {
                if (string.Equals(ReadNameOrNull(item), name, StringComparison.OrdinalIgnoreCase))
                {
                    return item;
                }
            }

            return EnumerateObjects(composition)
                .FirstOrDefault(item => string.Equals(ReadNameOrNull(item), name, StringComparison.OrdinalIgnoreCase));
        }

        private void AddWatchExpressionVerification(
            object watchTable,
            IList<object> expressions,
            List<Dictionary<string, object>> verifiedFields)
        {
            var readBackExpressions = DescribeWatchTable(watchTable);
            var readBackExpressionList = (List<Dictionary<string, object>>)readBackExpressions["expressions"];
            verifiedFields.Add(
                new Dictionary<string, object>
                {
                    { "field", "expression_count" },
                    { "expected", expressions == null ? 0 : expressions.Count },
                    { "actual", readBackExpressionList.Count },
                });

            verifiedFields.Add(
                new Dictionary<string, object>
                {
                    { "field", "expression_texts" },
                    {
                        "expected",
                        expressions == null
                            ? string.Empty
                            : string.Join(
                                "|",
                                expressions
                                    .OfType<Dictionary<string, object>>()
                                    .Select(expression => GetRequiredString(expression, "expression")))
                    },
                    {
                        "actual",
                        string.Join(
                            "|",
                            readBackExpressionList.Select(expression => SafeToString(expression["expression"])))
                    },
                });
        }

        private IEnumerable<object> EnumerateWatchExpressionCompositions(object watchTable)
        {
            var compositions = new List<object>();
            foreach (var propertyName in new[] { "Expressions", "Entries", "Rows", "WatchExpressions" })
            {
                var composition = TryGetPropertyValue(watchTable, propertyName);
                if (composition != null && !compositions.Contains(composition))
                {
                    compositions.Add(composition);
                }
            }

            return compositions;
        }

        private object WriteHardwareConfig(Dictionary<string, object> parameters)
        {
            EnsureProjectOpen();
            var target = ResolveObject(GetRequiredString(parameters, "device_object_id"));
            var operation = GetDictionary(parameters, "operation", required: true);
            var operationType = GetRequiredString(operation, "type");
            var changes = new List<Dictionary<string, object>>();
            var verifiedFields = new List<Dictionary<string, object>>();

            switch (operationType)
            {
                case "rename_device":
                    ApplyScalarEdit(target, "Name", GetRequiredString(operation, "new_name"), changes, verifiedFields);
                    break;
                case "set_profinet_device_name":
                    ApplyScalarEditWithCandidates(
                        target,
                        GetRequiredString(operation, "profinet_device_name"),
                        changes,
                        verifiedFields,
                        "ProfinetDeviceName",
                        "DeviceName",
                        "NameOfStation");
                    break;
                default:
                    throw new AdapterException(
                        "unsupported_live_operation",
                        "Unsupported live hardware configuration operation.",
                        new Dictionary<string, object> { { "type", operationType } });
            }

            return BuildVerifiedMutationResponse(target, changes, verifiedFields);
        }

        private object WriteNetworkConfig(Dictionary<string, object> parameters)
        {
            EnsureProjectOpen();
            var target = ResolveObject(GetRequiredString(parameters, "network_object_id"));
            var operation = GetDictionary(parameters, "operation", required: true);
            var operationType = GetRequiredString(operation, "type");
            var changes = new List<Dictionary<string, object>>();
            var verifiedFields = new List<Dictionary<string, object>>();

            switch (operationType)
            {
                case "rename_network":
                    ApplyScalarEdit(target, "Name", GetRequiredString(operation, "new_name"), changes, verifiedFields);
                    break;
                case "set_connected_objects":
                    throw new AdapterException(
                        "unsupported_live_operation",
                        "Live subnet participant rewiring is not yet implemented in the TIA Openness adapter.",
                        new Dictionary<string, object>
                        {
                            { "type", operationType },
                            { "network", DescribeEngineeringObject(target) },
                        });
                default:
                    throw new AdapterException(
                        "unsupported_live_operation",
                        "Unsupported live network configuration operation.",
                        new Dictionary<string, object> { { "type", operationType } });
            }

            return BuildVerifiedMutationResponse(target, changes, verifiedFields);
        }

        private object CreateHmiAlarm(Dictionary<string, object> parameters)
        {
            EnsureProjectOpen();
            var hmiTarget = ResolveObject(GetRequiredString(parameters, "hmi_object_id"));
            var hmiSoftware = ResolveUnifiedHmiSoftware(hmiTarget);
            var name = GetRequiredString(parameters, "name");
            var severity = GetRequiredString(parameters, "severity");
            var triggerTag = GetRequiredString(parameters, "trigger_tag");
            var message = GetRequiredString(parameters, "message");
            var touchedObjects = new List<Dictionary<string, object>>();
            var alarmClass = EnsureUnifiedAlarmClass(hmiSoftware, severity, touchedObjects);
            var alarms = GetPropertyValue(hmiSoftware, "DiscreteAlarms");
            var created = CreateObjectInComposition(alarms, name, new object[] { name });

            var changes = new List<Dictionary<string, object>>
            {
                new Dictionary<string, object>
                {
                    { "field", "created" },
                    { "before", null },
                    { "after", true },
                },
            };
            var verifiedFields = new List<Dictionary<string, object>>
            {
                new Dictionary<string, object>
                {
                    { "field", "name" },
                    { "expected", name },
                    { "actual", ReadName(created) },
                },
            };

            EnsureFieldValue(created, "AlarmClass", ReadName(alarmClass), changes, verifiedFields);
            EnsureFieldValue(created, "RaisedStateTag", triggerTag, changes, verifiedFields);
            EnsureFieldValue(created, "RaisedStateTagBitNumber", 0, changes, verifiedFields);
            EnsureFieldValue(created, "Priority", MapSeverityToPriority(severity), changes, verifiedFields);
            SetMultilingualTextField(created, "EventText", message, changes, verifiedFields);

            touchedObjects.Add(
                new Dictionary<string, object>
                {
                    { "object", DescribeEngineeringObject(created) },
                    { "changes", changes },
                });

            if (!verifiedFields.All(field => Equals(field["expected"], field["actual"])))
            {
                throw new AdapterException(
                    "verification_failed",
                    "The live HMI alarm creation call completed but read-back verification did not match the requested values.",
                    DescribeEngineeringObject(created));
            }

            return new Dictionary<string, object>
            {
                { "touched_objects", touchedObjects },
                {
                    "verification",
                    new Dictionary<string, object>
                    {
                        { "verified", true },
                        { "strategy", "read_back" },
                        { "checked_fields", verifiedFields },
                        { "exported_sha256", null },
                    }
                },
            };
        }

        private object CreateTechnologyObject(Dictionary<string, object> parameters)
        {
            EnsureProjectOpen();
            var plcSoftware = ResolveObject(GetRequiredString(parameters, "plc_software_id"));
            var root = FindTechnologyObjectRoot(plcSoftware);
            var composition = TryGetFirstPropertyValue(root, "TechnologicalObjects", "TechnologyObjects", "Objects");
            var rawTechnologyType = GetRequiredString(parameters, "technology_type");
            Version technologyVersion;
            string technologyType;
            ParseTechnologyType(rawTechnologyType, out technologyType, out technologyVersion);
            var created = CreateObjectInComposition(
                composition,
                GetRequiredString(parameters, "name"),
                new object[] { GetRequiredString(parameters, "name"), technologyType, technologyVersion });

            var changes = new List<Dictionary<string, object>>
            {
                new Dictionary<string, object>
                {
                    { "field", "created" },
                    { "before", null },
                    { "after", true },
                },
            };
            var verifiedFields = new List<Dictionary<string, object>>
            {
                new Dictionary<string, object>
                {
                    { "field", "name" },
                    { "expected", GetRequiredString(parameters, "name") },
                    { "actual", ReadName(created) },
                },
                new Dictionary<string, object>
                {
                    { "field", "technology_type" },
                    { "expected", technologyType },
                    { "actual", Convert.ToString(DescribeTechnologyObject(created)["technology_type"], CultureInfo.InvariantCulture) },
                },
            };

            var boundAxis = GetString(parameters, "bound_axis");
            if (!string.IsNullOrWhiteSpace(boundAxis))
            {
                ApplyScalarEditWithCandidates(created, boundAxis, changes, verifiedFields, "AxisName", "BoundAxisName");
            }

            return BuildVerifiedMutationResponse(created, changes, verifiedFields);
        }

        private object CreateSafetyObject(Dictionary<string, object> parameters)
        {
            EnsureProjectOpen();
            var plcSoftware = ResolveObject(GetRequiredString(parameters, "plc_software_id"));
            var name = GetRequiredString(parameters, "name");
            var safetyType = GetRequiredString(parameters, "safety_type");
            if (!string.Equals(safetyType, "runtime_group", StringComparison.OrdinalIgnoreCase) &&
                !string.Equals(safetyType, "runtimegroup", StringComparison.OrdinalIgnoreCase))
            {
                throw new AdapterException(
                    "unsupported_live_operation",
                    "Live safety authoring currently supports creation of Safety Runtime Groups only.",
                    new Dictionary<string, object>
                    {
                        { "safety_type", safetyType },
                        { "plc_software", DescribeEngineeringObject(plcSoftware) },
                    });
            }

            var safetyAdministration = ResolveSafetyAdministration(plcSoftware);
            if (safetyAdministration == null)
            {
                throw new AdapterException(
                    "unsupported_live_operation",
                    "The selected PLC software does not expose a SafetyAdministration object.",
                    DescribeEngineeringObject(plcSoftware));
            }

            var runtimeGroups = GetPropertyValue(safetyAdministration, "RuntimeGroups");
            var created = CreateObjectInComposition(runtimeGroups, name, new object[] { name });
            var changes = new List<Dictionary<string, object>>
            {
                new Dictionary<string, object>
                {
                    { "field", "created" },
                    { "before", null },
                    { "after", true },
                },
            };
            var verifiedFields = new List<Dictionary<string, object>>
            {
                new Dictionary<string, object>
                {
                    { "field", "name" },
                    { "expected", name },
                    { "actual", ReadName(created) },
                },
                new Dictionary<string, object>
                {
                    { "field", "safety_type" },
                    { "expected", "RuntimeGroup" },
                    { "actual", created.GetType().Name },
                },
            };

            return BuildVerifiedMutationResponse(created, changes, verifiedFields);
        }

        private object CompareOnlineOffline(Dictionary<string, object> parameters)
        {
            EnsureProjectOpen();
            var scope = GetDictionary(parameters, "scope", required: true);
            var scopeType = GetRequiredString(scope, "type");
            var differences = new List<Dictionary<string, object>>();
            Dictionary<string, object> scopeSummary = null;

            if (string.Equals(scopeType, "current_project", StringComparison.OrdinalIgnoreCase))
            {
                scopeSummary = DescribeEngineeringObject(_project);
                foreach (var plcSoftware in EnumerateProjectPlcSoftware())
                {
                    CollectCompareDifferences(plcSoftware, differences);
                }
            }
            else if (string.Equals(scopeType, "plc_software", StringComparison.OrdinalIgnoreCase))
            {
                var plcSoftware = ResolveObject(GetRequiredString(scope, "plc_software_id"));
                scopeSummary = DescribeEngineeringObject(plcSoftware);
                CollectCompareDifferences(plcSoftware, differences);
            }
            else if (string.Equals(scopeType, "object", StringComparison.OrdinalIgnoreCase))
            {
                var target = ResolveObject(GetRequiredString(scope, "object_id"));
                scopeSummary = DescribeEngineeringObject(target);
                var plcSoftware = ResolveOwningPlcSoftware(target);
                if (plcSoftware == null)
                {
                    throw new AdapterException(
                        "unsupported_live_operation",
                        "Online/offline compare currently requires a PLC software-backed object scope.",
                        new Dictionary<string, object>
                        {
                            { "scope", scopeSummary },
                        });
                }

                var allDifferences = new List<Dictionary<string, object>>();
                CollectCompareDifferences(plcSoftware, allDifferences);
                var targetPath = BuildPath(target);
                var targetName = ReadName(target);
                differences.AddRange(
                    allDifferences.Where(
                        difference =>
                        {
                            var path = Convert.ToString(difference["path"], CultureInfo.InvariantCulture) ?? string.Empty;
                            return (!string.IsNullOrWhiteSpace(targetPath) &&
                                    path.IndexOf(targetPath, StringComparison.OrdinalIgnoreCase) >= 0) ||
                                (!string.IsNullOrWhiteSpace(targetName) &&
                                    path.IndexOf(targetName, StringComparison.OrdinalIgnoreCase) >= 0);
                        }));
            }
            else
            {
                throw new AdapterException(
                    "unsupported_scope",
                    "Unsupported online/offline compare scope.",
                    new Dictionary<string, object> { { "type", scopeType } });
            }

            return new Dictionary<string, object>
            {
                { "scope", scopeSummary },
                { "status", differences.Count == 0 ? "in_sync" : "differences_found" },
                { "differences", differences },
            };
        }

        private object RunSimulation(Dictionary<string, object> parameters)
        {
            EnsureProjectOpen();
            var plcSoftware = ResolveObject(GetRequiredString(parameters, "plc_software_id"));
            var compileService = GetService(plcSoftware, "Siemens.Engineering.Compiler.ICompilable", required: false);
            if (compileService == null)
            {
                throw new AdapterException(
                    "unsupported_live_operation",
                    "The selected PLC software does not expose a direct compile service for a commissioning dry-run.",
                    DescribeEngineeringObject(plcSoftware));
            }

            var simulationProvider = GetServiceByAnyTypeName(
                plcSoftware,
                false,
                "Siemens.Engineering.SW.PlcSimulationSettingsProvider",
                "PlcSimulationSettingsProvider") ??
                GetServiceByAnyTypeName(
                    _project,
                    false,
                    "Siemens.Engineering.SW.PlcSimulationSettingsProvider",
                    "PlcSimulationSettingsProvider");
            var simulationEnabled = simulationProvider != null &&
                (GetNullableBoolean(simulationProvider, "IsSimulationDuringBlockCompilationEnabled") ?? false);
            var compilerResult = DescribeCompilerResult(InvokeMethod(compileService, "Compile"));
            var compileState = SafeToString(compilerResult["state"]) ?? "Unknown";
            var errorCount = Convert.ToInt32(compilerResult["error_count"], CultureInfo.InvariantCulture);
            var warningCount = Convert.ToInt32(compilerResult["warning_count"], CultureInfo.InvariantCulture);

            return new Dictionary<string, object>
            {
                { "plc_software", DescribeEngineeringObject(plcSoftware) },
                { "status", errorCount == 0 ? "dry_run_completed" : "dry_run_failed" },
                {
                    "observations",
                    new List<Dictionary<string, object>>
                    {
                        new Dictionary<string, object>
                        {
                            { "cycle", 1 },
                            { "signal", "simulation_during_block_compilation_enabled" },
                            { "value", simulationEnabled ? "TRUE" : "FALSE" },
                        },
                        new Dictionary<string, object>
                        {
                            { "cycle", 1 },
                            { "signal", "compile_state" },
                            { "value", compileState },
                        },
                        new Dictionary<string, object>
                        {
                            { "cycle", 1 },
                            { "signal", "compile_warning_count" },
                            { "value", warningCount.ToString(CultureInfo.InvariantCulture) },
                        },
                        new Dictionary<string, object>
                        {
                            { "cycle", 1 },
                            { "signal", "compile_error_count" },
                            { "value", errorCount.ToString(CultureInfo.InvariantCulture) },
                        },
                    }
                },
            };
        }

        private object GoOnline(Dictionary<string, object> parameters)
        {
            EnsureProjectOpen();
            var target = ResolveObject(GetRequiredString(parameters, "device_object_id"));
            var providerTarget = ResolveTargetWithService(
                target,
                "Siemens.Engineering.Online.RHOnlineProvider");
            var provider = GetServiceByAnyTypeName(
                providerTarget,
                true,
                "Siemens.Engineering.Online.RHOnlineProvider",
                "RHOnlineProvider");
            var before = SafeToString(TryGetPropertyValue(provider, "PrimaryState"));
            InvokeMethod(provider, "GoOnlineToPrimary");
            var after = SafeToString(TryGetPropertyValue(provider, "PrimaryState"));
            if (!string.Equals(after, "Online", StringComparison.OrdinalIgnoreCase))
            {
                throw new AdapterException(
                    "verification_failed",
                    "Go-online call returned but the provider did not report an online primary state.",
                    new Dictionary<string, object>
                    {
                        { "target", DescribeEngineeringObject(providerTarget) },
                        { "primary_state", after },
                    });
            }

            var requestedMode = GetString(parameters, "mode") ?? "monitor";
            return new Dictionary<string, object>
            {
                {
                    "touched_objects",
                    new List<Dictionary<string, object>>
                    {
                        new Dictionary<string, object>
                        {
                            { "object", DescribeEngineeringObject(target) },
                            {
                                "changes",
                                new List<Dictionary<string, object>>
                                {
                                    new Dictionary<string, object>
                                    {
                                        { "field", "online_state" },
                                        { "before", before },
                                        { "after", after },
                                    },
                                    new Dictionary<string, object>
                                    {
                                        { "field", "mode" },
                                        { "before", null },
                                        { "after", requestedMode },
                                    },
                                }
                            },
                        }
                    }
                },
                {
                    "verification",
                    new Dictionary<string, object>
                    {
                        { "verified", true },
                        { "strategy", "read_back" },
                        {
                            "checked_fields",
                            new List<Dictionary<string, object>>
                            {
                                new Dictionary<string, object>
                                {
                                    { "field", "online_state" },
                                    { "expected", "Online" },
                                    { "actual", after },
                                }
                            }
                        },
                        { "exported_sha256", null },
                    }
                },
            };
        }

        private object DownloadToDevice(Dictionary<string, object> parameters)
        {
            EnsureProjectOpen();
            var target = ResolveObject(GetRequiredString(parameters, "device_object_id"));
            var requestedObjectIds = GetList(parameters, "object_ids", required: false);
            if (requestedObjectIds != null && requestedObjectIds.Count > 0)
            {
                throw new AdapterException(
                    "unsupported_live_operation",
                    "Selective object-scope download is not yet implemented in the live TIA adapter.",
                    new Dictionary<string, object>
                    {
                        { "device", DescribeEngineeringObject(target) },
                    });
            }

            var providerTarget = ResolveTargetWithService(
                target,
                "Siemens.Engineering.Download.RHDownloadProvider");
            var provider = GetServiceByAnyTypeName(
                providerTarget,
                true,
                "Siemens.Engineering.Download.RHDownloadProvider",
                "RHDownloadProvider");
            var configuration = TryGetPropertyValue(provider, "Configuration");
            var configurationMode = SelectFirstConfigurationMode(configuration);
            var downloadResult = InvokeMethod(
                provider,
                "DownloadToPrimary",
                configurationMode,
                null,
                null,
                ResolveDownloadOptions(GetString(parameters, "download_mode")));
            var describedResult = DescribeDownloadResult(downloadResult);
            var state = Convert.ToString(describedResult["state"], CultureInfo.InvariantCulture);
            var errorCount = Convert.ToInt32(describedResult["error_count"], CultureInfo.InvariantCulture);
            if (errorCount > 0 || string.Equals(state, "Error", StringComparison.OrdinalIgnoreCase))
            {
                throw new AdapterException(
                    "download_failed",
                    "The live TIA download call reported errors.",
                    describedResult);
            }

            var postAction = GetString(parameters, "post_download_online_action") ?? "leave_offline";
            var onlineProvider = GetServiceByAnyTypeName(
                providerTarget,
                false,
                "Siemens.Engineering.Online.RHOnlineProvider",
                "RHOnlineProvider");
            var postOnlineState = "Offline";
            if (string.Equals(postAction, "go_online", StringComparison.OrdinalIgnoreCase))
            {
                if (onlineProvider == null)
                {
                    throw new AdapterException(
                        "missing_service",
                        "Download requested go-online post action, but no online provider is available for the selected target.",
                        new Dictionary<string, object>
                        {
                            { "target", DescribeEngineeringObject(providerTarget) },
                        });
                }

                InvokeMethod(onlineProvider, "GoOnlineToPrimary");
                postOnlineState = SafeToString(TryGetPropertyValue(onlineProvider, "PrimaryState")) ?? postOnlineState;
            }
            else if (onlineProvider != null)
            {
                InvokeMethod(onlineProvider, "GoOffline");
                postOnlineState = SafeToString(TryGetPropertyValue(onlineProvider, "PrimaryState")) ?? postOnlineState;
            }

            return new Dictionary<string, object>
            {
                {
                    "touched_objects",
                    new List<Dictionary<string, object>>
                    {
                        new Dictionary<string, object>
                        {
                            { "object", DescribeEngineeringObject(target) },
                            {
                                "changes",
                                new List<Dictionary<string, object>>
                                {
                                    new Dictionary<string, object>
                                    {
                                        { "field", "download_mode" },
                                        { "before", null },
                                        { "after", GetString(parameters, "download_mode") ?? "hardware_and_software" },
                                    },
                                    new Dictionary<string, object>
                                    {
                                        { "field", "download_state" },
                                        { "before", null },
                                        { "after", state },
                                    },
                                    new Dictionary<string, object>
                                    {
                                        { "field", "post_download_online_action" },
                                        { "before", null },
                                        { "after", postAction },
                                    },
                                }
                            },
                        }
                    }
                },
                {
                    "verification",
                    new Dictionary<string, object>
                    {
                        { "verified", true },
                        { "strategy", "provider_result" },
                        {
                            "checked_fields",
                            new List<Dictionary<string, object>>
                            {
                                new Dictionary<string, object>
                                {
                                    { "field", "download_state" },
                                    { "expected", state },
                                    { "actual", state },
                                },
                                new Dictionary<string, object>
                                {
                                    { "field", "error_count" },
                                    { "expected", 0 },
                                    { "actual", errorCount },
                                },
                                new Dictionary<string, object>
                                {
                                    { "field", "post_download_online_state" },
                                    { "expected", string.Equals(postAction, "go_online", StringComparison.OrdinalIgnoreCase) ? "Online" : "Offline" },
                                    { "actual", postOnlineState },
                                },
                            }
                        },
                        { "exported_sha256", null },
                    }
                },
                { "download_result", describedResult },
            };
        }

        private object ListTechnologyObjects(Dictionary<string, object> parameters)
        {
            EnsureProjectOpen();
            var plcSoftwareId = GetRequiredString(parameters, "plc_software_id");
            var plcSoftware = ResolveObject(plcSoftwareId);
            var root = FindTechnologyObjectRoot(plcSoftware);
            var technologyObjects = new List<Dictionary<string, object>>();
            var seen = new HashSet<string>(StringComparer.Ordinal);
            if (root != null)
            {
                WalkTechnologyObjectGroup(root, technologyObjects, seen);
            }

            return new Dictionary<string, object>
            {
                { "plc_software_id", plcSoftwareId },
                { "technology_objects", technologyObjects },
            };
        }

        private object ListWatchTables(Dictionary<string, object> parameters)
        {
            EnsureProjectOpen();
            var plcSoftwareId = GetRequiredString(parameters, "plc_software_id");
            var plcSoftware = ResolveObject(plcSoftwareId);
            var root = FindWatchTableRoot(plcSoftware);
            var watchTables = new List<Dictionary<string, object>>();
            var seen = new HashSet<string>(StringComparer.Ordinal);
            if (root != null)
            {
                WalkWatchTableGroup(root, watchTables, seen);
            }

            return new Dictionary<string, object>
            {
                { "plc_software_id", plcSoftwareId },
                { "watch_tables", watchTables },
            };
        }

        private object ListNetworks(Dictionary<string, object> parameters)
        {
            EnsureProjectOpen();
            var networks = EnumerateNetworkObjects()
                .Select(DescribeNetwork)
                .ToList();
            return new Dictionary<string, object>
            {
                { "networks", networks },
            };
        }

        private object ListHmiObjects(Dictionary<string, object> parameters)
        {
            EnsureProjectOpen();
            return new Dictionary<string, object>
            {
                { "hmi_objects", EnumerateHmiObjects().Select(DescribeHmiObject).ToList() },
            };
        }

        private object ListSafetyObjects(Dictionary<string, object> parameters)
        {
            EnsureProjectOpen();
            var plcSoftwareId = GetString(parameters, "plc_software_id");
            var safetyObjects = new List<Dictionary<string, object>>();
            if (!string.IsNullOrWhiteSpace(plcSoftwareId))
            {
                var plcSoftware = ResolveObject(plcSoftwareId);
                safetyObjects.AddRange(EnumerateSafetyObjects(plcSoftware).Select(DescribeSafetyObject));
            }
            else
            {
                foreach (var plcSoftware in EnumerateProjectPlcSoftware())
                {
                    safetyObjects.AddRange(EnumerateSafetyObjects(plcSoftware).Select(DescribeSafetyObject));
                }
            }

            return new Dictionary<string, object>
            {
                { "plc_software_id", plcSoftwareId },
                { "safety_objects", safetyObjects },
            };
        }

        private object ConsistencyCheck(Dictionary<string, object> parameters)
        {
            EnsureProjectOpen();
            var scope = GetDictionary(parameters, "scope", required: true);
            var target = ResolveConsistencyCheckTarget(scope);
            var compileService = GetService(target, "Siemens.Engineering.Compiler.ICompilable", required: false);
            if (compileService == null)
            {
                throw new AdapterException(
                    "unsupported_live_operation",
                    "The selected scope does not expose a direct Openness compile service for consistency checking.",
                    new Dictionary<string, object>
                    {
                        { "action", "consistency_check" },
                        { "scope", DescribeEngineeringObject(target) },
                    });
            }

            var compilerResult = InvokeMethod(compileService, "Compile");
            var issues = new List<Dictionary<string, object>>();
            CollectConsistencyIssues(compilerResult, issues);
            return new Dictionary<string, object>
            {
                { "scope", DescribeEngineeringObject(target) },
                { "issue_count", issues.Count },
                { "issues", issues },
            };
        }

        private object CrossReference(Dictionary<string, object> parameters)
        {
            EnsureProjectOpen();
            var target = ResolveObject(GetRequiredString(parameters, "object_id"));
            var service = GetServiceByAnyTypeName(
                target,
                required: false,
                "Siemens.Engineering.CrossReferenceService",
                "CrossReferenceService");
            if (service == null)
            {
                throw new AdapterException(
                    "unsupported_live_operation",
                    "The selected object does not expose the Openness CrossReferenceService.",
                    new Dictionary<string, object>
                    {
                        { "action", "cross_reference" },
                        { "target", DescribeEngineeringObject(target) },
                    });
            }

            var filterType = FindTypeByAnyName(
                "Siemens.Engineering.CrossReferenceFilter",
                "CrossReferenceFilter");
            var filterValue = Enum.Parse(filterType, "AllObjects");
            var rootResult = InvokeMethod(service, "GetCrossReferences", filterValue);
            var references = new List<Dictionary<string, object>>();
            var seen = new HashSet<string>(StringComparer.Ordinal);
            foreach (var source in EnumerateObjects(TryGetPropertyValue(rootResult, "Sources")))
            {
                CollectCrossReferenceHits(source, references, seen);
            }

            return new Dictionary<string, object>
            {
                { "target", DescribeEngineeringObject(target) },
                { "references", references },
            };
        }

        private object ExportObject(Dictionary<string, object> parameters)
        {
            EnsureProjectOpen();
            var target = ResolveObject(GetRequiredString(parameters, "object_id"));
            var destinationPath = GetString(parameters, "destination_path");
            var readMode = GetString(parameters, "read_mode") ?? "include_text";
            if (string.IsNullOrWhiteSpace(destinationPath))
            {
                destinationPath = BuildDefaultExportPath(target);
            }

            ExportToPath(target, destinationPath);
            if (!File.Exists(destinationPath))
            {
                throw new AdapterException(
                    "verification_failed",
                    "Export completed but no file was created.",
                    new Dictionary<string, object> { { "path", destinationPath } });
            }

            var sha = ComputeSha256(destinationPath);
            return new Dictionary<string, object>
            {
                { "object", DescribeEngineeringObject(target) },
                { "export_path", destinationPath },
                { "content_sha256", sha },
                {
                    "content_text",
                    string.Equals(readMode, "include_text", StringComparison.OrdinalIgnoreCase)
                        ? File.ReadAllText(destinationPath)
                        : null
                },
                {
                    "verification",
                    new Dictionary<string, object>
                    {
                        { "verified", true },
                        { "strategy", "post_export_file_check" },
                        { "checked_fields", new object[0] },
                        { "exported_sha256", sha },
                    }
                },
            };
        }

        private object ImportObject(Dictionary<string, object> parameters)
        {
            EnsureProjectOpen();
            var targetGroup = ResolveObject(GetRequiredString(parameters, "target_group_object_id"));
            var sourceFilePath = GetRequiredString(parameters, "source_file_path");
            if (!File.Exists(sourceFilePath))
            {
                throw new AdapterException(
                    "missing_file",
                    "Import source file was not found.",
                    new Dictionary<string, object> { { "path", sourceFilePath } });
            }

            var composition = ResolveImportComposition(targetGroup);
            var beforeHashes = BuildPreImportHashes(composition);
            var importedObjects = EnumerateObjects(
                InvokeImport(composition, sourceFilePath, GetString(parameters, "conflict_mode") ?? "none"))
                .ToList();
            var touched = new List<Dictionary<string, object>>();
            var verifiedFields = new List<Dictionary<string, object>>();

            foreach (var importedObject in importedObjects)
            {
                var name = ReadName(importedObject);
                string beforeHash;
                beforeHashes.TryGetValue(name, out beforeHash);
                var afterHash = TryComputeExportHash(importedObject);
                touched.Add(
                    new Dictionary<string, object>
                    {
                        { "object", DescribeEngineeringObject(importedObject) },
                        {
                            "changes",
                            new List<Dictionary<string, object>>
                            {
                                new Dictionary<string, object>
                                {
                                    { "field", "import_source_path" },
                                    { "before", null },
                                    { "after", sourceFilePath },
                                },
                                new Dictionary<string, object>
                                {
                                    { "field", "export_sha256" },
                                    { "before", beforeHash },
                                    { "after", afterHash },
                                },
                            }
                        },
                    });
                verifiedFields.Add(
                    new Dictionary<string, object>
                    {
                        { "field", string.Format(CultureInfo.InvariantCulture, "{0}.export_sha256", name) },
                        { "expected", afterHash },
                        { "actual", afterHash },
                    });
            }

            var verified = importedObjects.Count > 0 && verifiedFields.All(
                field => field["expected"] != null && Equals(field["expected"], field["actual"]));
            if (!verified)
            {
                throw new AdapterException(
                    "verification_failed",
                    "Import API call succeeded but read-back verification did not complete.",
                    new Dictionary<string, object>
                    {
                        { "source_file_path", sourceFilePath },
                        { "imported_object_count", importedObjects.Count },
                    });
            }

            return new Dictionary<string, object>
            {
                { "touched_objects", touched },
                {
                    "verification",
                    new Dictionary<string, object>
                    {
                        { "verified", true },
                        { "strategy", "post_import_export_hash" },
                        { "checked_fields", verifiedFields },
                        { "exported_sha256", null },
                    }
                },
            };
        }

        private object ApplyEdit(Dictionary<string, object> parameters)
        {
            EnsureProjectOpen();
            var target = ResolveObject(GetRequiredString(parameters, "object_id"));
            var operation = GetDictionary(parameters, "operation", required: true);
            var operationType = GetRequiredString(operation, "type");

            var changes = new List<Dictionary<string, object>>();
            var verifiedFields = new List<Dictionary<string, object>>();

            switch (operationType)
            {
                case "rename_object":
                    ApplyScalarEdit(target, "Name", GetRequiredString(operation, "new_name"), changes, verifiedFields);
                    break;
                case "set_block_header":
                    MaybeApplyScalarEdit(target, "HeaderAuthor", GetString(operation, "header_author"), changes, verifiedFields);
                    MaybeApplyScalarEdit(target, "HeaderFamily", GetString(operation, "header_family"), changes, verifiedFields);
                    MaybeApplyScalarEdit(target, "HeaderName", GetString(operation, "header_name"), changes, verifiedFields);
                    MaybeApplyScalarEdit(target, "HeaderVersion", GetString(operation, "header_version"), changes, verifiedFields);
                    break;
                case "set_plc_tag_properties":
                    MaybeApplyScalarEdit(target, "Name", GetString(operation, "name"), changes, verifiedFields);
                    MaybeApplyScalarEdit(target, "DataTypeName", GetString(operation, "data_type_name"), changes, verifiedFields);
                    MaybeApplyScalarEdit(target, "LogicalAddress", GetString(operation, "logical_address"), changes, verifiedFields);
                    MaybeApplyScalarEdit(target, "ExternalAccessible", GetNullableBoolean(operation, "external_accessible"), changes, verifiedFields);
                    MaybeApplyScalarEdit(target, "ExternalVisible", GetNullableBoolean(operation, "external_visible"), changes, verifiedFields);
                    MaybeApplyScalarEdit(target, "ExternalWritable", GetNullableBoolean(operation, "external_writable"), changes, verifiedFields);
                    MaybeApplyScalarEdit(target, "IsSafety", GetNullableBoolean(operation, "is_safety"), changes, verifiedFields);
                    break;
                default:
                    throw new AdapterException(
                        "unsupported_edit",
                        "Unsupported edit operation.",
                        new Dictionary<string, object> { { "type", operationType } });
            }

            var verified = verifiedFields.All(field => Equals(field["expected"], field["actual"]));
            if (!verified)
            {
                throw new AdapterException(
                    "verification_failed",
                    "Edit API call succeeded but one or more fields failed verification.",
                    new Dictionary<string, object> { { "object_id", GetRequiredString(parameters, "object_id") } });
            }

            return new Dictionary<string, object>
            {
                {
                    "touched_objects",
                    new List<Dictionary<string, object>>
                    {
                        new Dictionary<string, object>
                        {
                            { "object", DescribeEngineeringObject(target) },
                            { "changes", changes },
                        }
                    }
                },
                {
                    "verification",
                    new Dictionary<string, object>
                    {
                        { "verified", true },
                        { "strategy", "read_back" },
                        { "checked_fields", verifiedFields },
                        { "exported_sha256", null },
                    }
                },
            };
        }

        private object Compile(Dictionary<string, object> parameters)
        {
            EnsureProjectOpen();
            var scope = GetDictionary(parameters, "scope", required: true);
            var scopeType = GetRequiredString(scope, "type");
            object target;
            if (string.Equals(scopeType, "current_project", StringComparison.OrdinalIgnoreCase))
            {
                target = _project;
            }
            else if (string.Equals(scopeType, "object", StringComparison.OrdinalIgnoreCase))
            {
                target = ResolveObject(GetRequiredString(scope, "object_id"));
            }
            else
            {
                throw new AdapterException(
                    "unsupported_compile_scope",
                    "Unsupported compile scope.",
                    new Dictionary<string, object> { { "type", scopeType } });
            }

            var compileService = GetService(target, "Siemens.Engineering.Compiler.ICompilable", required: true);
            var compilerResult = InvokeMethod(compileService, "Compile");
            return new Dictionary<string, object>
            {
                { "scope", DescribeEngineeringObject(target) },
                { "result", DescribeCompilerResult(compilerResult) },
            };
        }

        private object UnsupportedLiveOperation(string action)
        {
            throw new AdapterException(
                "unsupported_live_operation",
                "The requested PLC engineering action is not yet implemented in the live TIA Openness adapter. Extend the adapter with a direct Openness implementation for this capability.",
                new Dictionary<string, object>
                {
                    { "action", action },
                    { "portal_version", _portalVersion },
                });
        }

        private InstalledPortal ResolveInstall(string requestedVersion)
        {
            if (!string.IsNullOrWhiteSpace(_publicApiDirectoryOverride))
            {
                return CreateInstallFromDirectory(_publicApiDirectoryOverride, requestedVersion ?? _portalVersionOverride);
            }

            var candidates = DiscoverInstalls();
            if (!string.IsNullOrWhiteSpace(requestedVersion))
            {
                var exact = candidates.FirstOrDefault(
                    candidate => candidate.PortalVersion.Equals(NormalizePortalVersion(requestedVersion), StringComparison.OrdinalIgnoreCase));
                if (exact != null)
                {
                    return exact;
                }
            }

            var latest = candidates
                .OrderByDescending(candidate => candidate.SortKey)
                .FirstOrDefault();
            if (latest == null)
            {
                throw new AdapterException(
                    "portal_not_found",
                    "No TIA Portal Openness installation could be resolved.",
                    new Dictionary<string, object>
                    {
                        { "portal_version", requestedVersion },
                        { "public_api_override", _publicApiDirectoryOverride },
                    });
            }

            return latest;
        }

        private void EnsureAssembliesLoaded(InstalledPortal install)
        {
            if (string.Equals(_assemblyDirectory, install.PublicApiDirectory, StringComparison.OrdinalIgnoreCase))
            {
                return;
            }

            _assemblyDirectory = install.PublicApiDirectory;
            _portalVersion = install.PortalVersion;
            _loadedAssemblies.Clear();

            if (!Directory.Exists(_assemblyDirectory))
            {
                throw new AdapterException(
                    "missing_public_api_dir",
                    "Resolved TIA PublicAPI directory does not exist.",
                    new Dictionary<string, object> { { "path", _assemblyDirectory } });
            }

            if (install.UsesModularAssemblies)
            {
                LoadAssemblyIfExists("Siemens.Engineering.Base.dll", required: true);
                LoadAssemblyIfExists("Siemens.Engineering.Step7.dll", required: true);
                LoadAssemblyIfExists("Siemens.Engineering.Safety.dll", required: false);
            }
            else
            {
                LoadAssemblyIfExists("Siemens.Engineering.dll", required: true);
            }
        }

        private void AttachOpenProjectIfPresent()
        {
            _project = null;
            var projects = TryGetPropertyValue(_tiaPortal, "Projects");
            if (projects != null)
            {
                _project = EnumerateObjects(projects).FirstOrDefault();
            }

            RefreshObjectIdentifierProvider();
        }

        private void RefreshObjectIdentifierProvider()
        {
            if (_project == null)
            {
                _objectIdentifierProvider = null;
                return;
            }

            _objectIdentifierProvider = GetService(
                _project,
                "Siemens.Engineering.ObjectIdentifierProvider",
                required: true);
        }

        private object LaunchPortal(Type tiaPortalType, string requestedUiMode)
        {
            var modeType = FindType("Siemens.Engineering.TiaPortalMode");
            var modeName = string.Equals(requestedUiMode, "without_ui", StringComparison.OrdinalIgnoreCase)
                ? "WithoutUserInterface"
                : "WithUserInterface";
            var modeValue = Enum.Parse(modeType, modeName);
            return Activator.CreateInstance(tiaPortalType, modeValue);
        }

        private IEnumerable<object> FlattenDeviceItems(object parent)
        {
            foreach (var child in EnumerateObjects(TryGetPropertyValue(parent, "DeviceItems")))
            {
                yield return child;
                foreach (var nested in FlattenDeviceItems(child))
                {
                    yield return nested;
                }
            }
        }

        private void WalkBlockGroup(object group, List<Dictionary<string, object>> blocks, string traversalMode)
        {
            foreach (var block in EnumerateObjects(TryGetPropertyValue(group, "Blocks")))
            {
                blocks.Add(DescribeBlock(block));
            }

            if (!string.Equals(traversalMode, "recursive", StringComparison.OrdinalIgnoreCase))
            {
                return;
            }

            foreach (var childGroup in EnumerateObjects(TryGetPropertyValue(group, "Groups")))
            {
                WalkBlockGroup(childGroup, blocks, traversalMode);
            }
        }

        private void WalkTagTableGroup(
            object group,
            List<Dictionary<string, object>> tagTables,
            string traversalMode,
            bool includeTags)
        {
            foreach (var table in EnumerateObjects(TryGetPropertyValue(group, "TagTables")))
            {
                tagTables.Add(DescribeTagTable(table, includeTags));
            }

            if (!string.Equals(traversalMode, "recursive", StringComparison.OrdinalIgnoreCase))
            {
                return;
            }

            foreach (var childGroup in EnumerateObjects(TryGetPropertyValue(group, "Groups")))
            {
                WalkTagTableGroup(childGroup, tagTables, traversalMode, includeTags);
            }
        }

        private Dictionary<string, object> DescribeProcess(object process)
        {
            return new Dictionary<string, object>
            {
                { "process_id", GetNullableInt(process, "Id") },
                { "mode", GetString(process, "Mode") },
                { "project_path", SafeToString(TryGetPropertyValue(process, "ProjectPath")) },
                { "executable_path", SafeToString(TryGetPropertyValue(process, "Path")) },
            };
        }

        private Dictionary<string, object> DescribeBlock(object block)
        {
            return new Dictionary<string, object>
            {
                { "object", DescribeEngineeringObject(block) },
                { "block_type", block.GetType().Name },
                { "group_path", BuildPath(TryGetPropertyValue(block, "Parent")) },
                { "number", GetNullableInt(block, "Number") ?? GetNullableIntAttribute(block, "Number") },
                { "header_author", GetString(block, "HeaderAuthor") },
                { "header_family", GetString(block, "HeaderFamily") },
                { "header_name", GetString(block, "HeaderName") },
                { "header_version", SafeToString(TryGetPropertyValue(block, "HeaderVersion")) },
            };
        }

        private Dictionary<string, object> DescribeTagTable(object table, bool includeTags)
        {
            return new Dictionary<string, object>
            {
                { "object", DescribeEngineeringObject(table) },
                { "group_path", BuildPath(TryGetPropertyValue(table, "Parent")) },
                {
                    "tags",
                    includeTags
                        ? EnumerateObjects(TryGetPropertyValue(table, "Tags")).Select(DescribeTag).ToList()
                        : null
                },
            };
        }

        private Dictionary<string, object> DescribeTag(object tag)
        {
            return new Dictionary<string, object>
            {
                { "object", DescribeEngineeringObject(tag) },
                { "data_type_name", GetString(tag, "DataTypeName") },
                { "logical_address", GetString(tag, "LogicalAddress") },
                { "external_accessible", GetNullableBoolean(tag, "ExternalAccessible") },
                { "external_visible", GetNullableBoolean(tag, "ExternalVisible") },
                { "external_writable", GetNullableBoolean(tag, "ExternalWritable") },
            };
        }

        private Dictionary<string, object> DescribeEngineeringObject(object target)
        {
            return new Dictionary<string, object>
            {
                { "object_id", GetObjectIdentifier(target) },
                { "kind", target.GetType().Name },
                { "name", ReadName(target) },
                { "path", BuildPath(target) },
            };
        }

        private Dictionary<string, object> DescribeCompilerResult(object compilerResult)
        {
            return new Dictionary<string, object>
            {
                { "state", SafeToString(TryGetPropertyValue(compilerResult, "State")) },
                { "warning_count", GetNullableInt(compilerResult, "WarningCount") ?? 0 },
                { "error_count", GetNullableInt(compilerResult, "ErrorCount") ?? 0 },
                {
                    "messages",
                    EnumerateObjects(TryGetPropertyValue(compilerResult, "Messages"))
                        .Select(
                            message => new Dictionary<string, object>
                            {
                                { "path", SafeToString(TryGetPropertyValue(message, "Path")) },
                                { "state", SafeToString(TryGetPropertyValue(message, "State")) },
                                { "description", SafeToString(TryGetPropertyValue(message, "Description")) },
                                { "warning_count", GetNullableInt(message, "WarningCount") ?? 0 },
                                { "error_count", GetNullableInt(message, "ErrorCount") ?? 0 },
                                { "messages", DescribeCompilerResult(message)["messages"] },
                            }).ToList()
                },
            };
        }

        private Dictionary<string, string> BuildPreImportHashes(object composition)
        {
            var hashes = new Dictionary<string, string>(StringComparer.OrdinalIgnoreCase);
            foreach (var item in EnumerateObjects(composition))
            {
                var name = ReadName(item);
                if (string.IsNullOrWhiteSpace(name))
                {
                    continue;
                }

                hashes[name] = TryComputeExportHash(item);
            }

            return hashes;
        }

        private void EnsureNamedObjectDoesNotExist(IEnumerable<object> candidates, string name, string objectKind)
        {
            if (candidates.Any(candidate => string.Equals(ReadName(candidate), name, StringComparison.OrdinalIgnoreCase)))
            {
                throw new AdapterException(
                    "name_conflict",
                    "An engineering object with the requested name already exists in the selected scope.",
                    new Dictionary<string, object>
                    {
                        { "name", name },
                        { "object_kind", objectKind },
                    });
            }
        }

        private object ResolveBlockComposition(object plcSoftware)
        {
            var blockGroup = GetPropertyValue(plcSoftware, "BlockGroup");
            return TryGetPropertyValue(blockGroup, "Blocks") ?? blockGroup;
        }

        private object ResolveProgrammingLanguage(string language)
        {
            var enumType = FindTypeByAnyName(
                "Siemens.Engineering.SW.Blocks.ProgrammingLanguage",
                "ProgrammingLanguage");
            string value;
            switch ((language ?? "scl").ToLowerInvariant())
            {
                case "lad":
                    value = "LAD";
                    break;
                case "fbd":
                    value = "FBD";
                    break;
                default:
                    value = "SCL";
                    break;
            }

            return Enum.Parse(enumType, value);
        }

        private bool ShouldUseDirectFbCreate(string blockKind, string language, string blockBody)
        {
            return string.Equals(blockKind, "fb", StringComparison.OrdinalIgnoreCase) &&
                string.IsNullOrWhiteSpace(blockBody) &&
                (string.Equals(language, "lad", StringComparison.OrdinalIgnoreCase) ||
                 string.Equals(language, "fbd", StringComparison.OrdinalIgnoreCase));
        }

        private object FindBlockByName(object plcSoftware, string name)
        {
            return EnumerateBlockGroupObjects(GetPropertyValue(plcSoftware, "BlockGroup"))
                .FirstOrDefault(candidate => string.Equals(ReadName(candidate), name, StringComparison.OrdinalIgnoreCase));
        }

        private object FindDataTypeByName(object plcSoftware, string name)
        {
            return EnumerateDataTypeObjects(FindDataTypeRoot(plcSoftware))
                .FirstOrDefault(candidate => string.Equals(ReadName(candidate), name, StringComparison.OrdinalIgnoreCase));
        }

        private bool IsDataBlock(object target)
        {
            var typeName = target.GetType().Name;
            return string.Equals(typeName, "DataBlock", StringComparison.OrdinalIgnoreCase) ||
                string.Equals(typeName, "GlobalDB", StringComparison.OrdinalIgnoreCase) ||
                string.Equals(typeName, "InstanceDB", StringComparison.OrdinalIgnoreCase);
        }

        private object ResolveSafetyAdministration(object plcSoftware)
        {
            var direct = TryGetFirstPropertyValue(plcSoftware, "SafetyAdministration", "SafetyAdmin");
            if (direct != null)
            {
                return direct;
            }

            direct = GetServiceByAnyTypeName(
                plcSoftware,
                false,
                "Siemens.Engineering.Safety.SafetyAdministration",
                "SafetyAdministration");
            if (direct != null)
            {
                return direct;
            }

            foreach (var property in plcSoftware.GetType().GetProperties(BindingFlags.Public | BindingFlags.Instance))
            {
                object value;
                try
                {
                    value = property.GetValue(plcSoftware, null);
                }
                catch
                {
                    continue;
                }

                if (value != null &&
                    value.GetType().Name.IndexOf("SafetyAdministration", StringComparison.OrdinalIgnoreCase) >= 0)
                {
                    return value;
                }
            }

            return null;
        }

        private object FindExternalSourceRoot(object plcSoftware)
        {
            return TryGetFirstPropertyValue(
                plcSoftware,
                "ExternalSourceGroup",
                "PlcExternalSourceSystemGroup",
                "ExternalSources") ??
                TryGetPropertyValueByFragments(plcSoftware, "External", "Source");
        }

        private object ResolveExternalSourceComposition(object plcSoftware)
        {
            var root = FindExternalSourceRoot(plcSoftware);
            var composition = TryGetFirstPropertyValue(root, "ExternalSources", "Sources");
            if (composition != null)
            {
                return composition;
            }

            foreach (var property in root.GetType().GetProperties(BindingFlags.Public | BindingFlags.Instance))
            {
                object value;
                try
                {
                    value = property.GetValue(root, null);
                }
                catch
                {
                    continue;
                }

                if (value != null &&
                    value.GetType().Name.IndexOf("PlcExternalSourceComposition", StringComparison.OrdinalIgnoreCase) >= 0)
                {
                    return value;
                }
            }

            throw new AdapterException(
                "unsupported_live_operation",
                "The selected PLC software does not expose a writable external-source composition.",
                DescribeEngineeringObject(plcSoftware));
        }

        private List<object> GenerateObjectsFromSourceText(object plcSoftware, string baseName, string extension, string sourceText)
        {
            var sourceFilePath = BuildTemporarySourceFilePath(baseName, extension);
            var externalSourceName = string.Format(
                CultureInfo.InvariantCulture,
                "AIPLC_{0}_{1}",
                SanitizeName(baseName),
                Guid.NewGuid().ToString("N").Substring(0, 8));
            object externalSource = null;
            try
            {
                File.WriteAllText(sourceFilePath, sourceText, new UTF8Encoding(true));
                externalSource = CreateExternalSourceFromFile(ResolveExternalSourceComposition(plcSoftware), externalSourceName, sourceFilePath);
                var generateOptionType = TryFindTypeByAnyName(
                    "Siemens.Engineering.SW.ExternalSources.GenerateBlockOption",
                    "GenerateBlockOption");
                var generated = generateOptionType != null
                    ? InvokeCompatibleMethod(externalSource, "GenerateBlocksFromSource", Enum.Parse(generateOptionType, "None"))
                    : InvokeCompatibleMethod(externalSource, "GenerateBlocksFromSource");
                return EnumerateObjects(generated).ToList();
            }
            finally
            {
                if (externalSource != null)
                {
                    DeleteEngineeringObject(externalSource);
                }

                if (File.Exists(sourceFilePath))
                {
                    File.Delete(sourceFilePath);
                }
            }
        }

        private object CreateExternalSourceFromFile(object composition, string name, string path)
        {
            return InvokeCompatibleMethod(composition, "CreateFromFile", name, path);
        }

        private string GenerateSourceText(object plcSoftware, IEnumerable<object> targets)
        {
            var targetList = targets.Where(item => item != null).ToList();
            if (targetList.Count == 0)
            {
                throw new AdapterException(
                    "missing_parameter",
                    "At least one source-generation target is required.",
                    new Dictionary<string, object> { { "parameter", "targets" } });
            }

            var generateSourceType = FindTypeByAnyName(
                "Siemens.Engineering.SW.ExternalSources.IGenerateSource",
                "IGenerateSource");
            var typedTargets = Array.CreateInstance(generateSourceType, targetList.Count);
            for (var index = 0; index < targetList.Count; index++)
            {
                typedTargets.SetValue(targetList[index], index);
            }

            var sourceFilePath = BuildTemporarySourceFilePath(
                ReadName(targetList[0]) ?? "read_back",
                InferSourceExtensionForTargets(targetList));
            try
            {
                var root = FindExternalSourceRoot(plcSoftware);
                var generateOptionsType = TryFindTypeByAnyName(
                    "Siemens.Engineering.SW.ExternalSources.GenerateOptions",
                    "GenerateOptions");
                if (generateOptionsType != null)
                {
                    InvokeCompatibleMethod(
                        root,
                        "GenerateSource",
                        typedTargets,
                        new FileInfo(sourceFilePath),
                        Enum.Parse(generateOptionsType, "None"));
                }
                else
                {
                    InvokeCompatibleMethod(root, "GenerateSource", typedTargets, new FileInfo(sourceFilePath));
                }

                return File.ReadAllText(sourceFilePath);
            }
            finally
            {
                if (File.Exists(sourceFilePath))
                {
                    File.Delete(sourceFilePath);
                }
            }
        }

        private string BuildTemporarySourceFilePath(string baseName, string extension)
        {
            var directory = Path.Combine(Path.GetTempPath(), "codex-tia", "sources");
            Directory.CreateDirectory(directory);
            var normalizedExtension = string.IsNullOrWhiteSpace(extension)
                ? ".scl"
                : (extension.StartsWith(".") ? extension : "." + extension);
            return Path.Combine(
                directory,
                string.Format(
                    CultureInfo.InvariantCulture,
                    "{0}_{1}{2}",
                    SanitizeName(baseName),
                    Guid.NewGuid().ToString("N"),
                    normalizedExtension));
        }

        private string InferSourceExtensionForBlockKind(string blockKind)
        {
            return string.Equals(blockKind, "global_db", StringComparison.OrdinalIgnoreCase) ? ".db" : ".scl";
        }

        private string InferSourceExtensionForTargets(List<object> targets)
        {
            if (targets == null || targets.Count == 0)
            {
                return ".scl";
            }

            var extensions = targets
                .Where(target => target != null)
                .Select(InferSourceExtensionForTarget)
                .Distinct(StringComparer.OrdinalIgnoreCase)
                .ToList();

            if (extensions.Count <= 1)
            {
                return extensions.Count == 0 ? ".scl" : extensions[0];
            }

            throw new AdapterException(
                "unsupported_live_operation",
                "Source generation for mixed engineering object kinds is not supported in one operation.",
                new Dictionary<string, object>
                {
                    { "extensions", extensions },
                    { "targets", targets.Select(DescribeEngineeringObject).ToList() },
                });
        }

        private string InferSourceExtensionForTarget(object target)
        {
            if (target == null)
            {
                return ".scl";
            }

            if (IsDataBlock(target))
            {
                return ".db";
            }

            return string.Equals(InferDataTypeKind(target), "udt", StringComparison.OrdinalIgnoreCase) ? ".udt" : ".scl";
        }

        private object InvokeCompatibleMethod(object target, string methodName, params object[] args)
        {
            Exception lastException = null;
            foreach (var method in target.GetType().GetMethods()
                .Where(candidate => candidate.Name == methodName &&
                    candidate.GetParameters().Length == args.Length))
            {
                object[] convertedArguments;
                if (!TryConvertArguments(method.GetParameters(), args, out convertedArguments))
                {
                    continue;
                }

                try
                {
                    return method.Invoke(target, convertedArguments);
                }
                catch (TargetInvocationException ex)
                {
                    lastException = ex.InnerException ?? ex;
                }
                catch (Exception ex)
                {
                    lastException = ex;
                }
            }

            if (lastException != null)
            {
                throw lastException;
            }

            throw new AdapterException(
                "missing_method",
                "A compatible method overload was not found on the selected Openness object.",
                new Dictionary<string, object>
                {
                    { "type", target.GetType().FullName },
                    { "method", methodName },
                    { "arg_count", args.Length },
                });
        }

        private object FindNamedGeneratedObject(IEnumerable<object> generatedObjects, string name)
        {
            return generatedObjects.FirstOrDefault(
                candidate => string.Equals(ReadName(candidate), name, StringComparison.OrdinalIgnoreCase));
        }

        private void DeleteEngineeringObject(object target)
        {
            var deleteMethod = target.GetType().GetMethods()
                .FirstOrDefault(method => method.Name == "Delete" && method.GetParameters().Length == 0);
            if (deleteMethod != null)
            {
                deleteMethod.Invoke(target, null);
            }
        }

        private string BuildUdtSource(string name, List<object> members)
        {
            var newline = Environment.NewLine;
            return string.Format(
                CultureInfo.InvariantCulture,
                "TYPE {0}{1}VERSION : 0.1{1}   STRUCT{1}{2}   END_STRUCT;{1}END_TYPE",
                QuotePlcIdentifier(name),
                newline,
                BuildMemberSourceLines(members.Select(ToMemberDictionary).ToList(), newline, "      "));
        }

        private string BuildBlockSource(string blockKind, string name, string blockBody)
        {
            var normalizedBody = PrepareBlockBodyForSource(blockBody);
            var newline = Environment.NewLine;
            if (string.Equals(blockKind, "fb", StringComparison.OrdinalIgnoreCase))
            {
                return string.Format(
                    CultureInfo.InvariantCulture,
                    "FUNCTION_BLOCK {0}{1}VERSION : 0.1{1}BEGIN{1}{2}{1}END_FUNCTION_BLOCK",
                    QuotePlcIdentifier(name),
                    newline,
                    IndentSourceLines(normalizedBody, "    "));
            }

            if (string.Equals(blockKind, "fc", StringComparison.OrdinalIgnoreCase))
            {
                return string.Format(
                    CultureInfo.InvariantCulture,
                    "FUNCTION {0} : Void{1}VERSION : 0.1{1}BEGIN{1}{2}{1}END_FUNCTION",
                    QuotePlcIdentifier(name),
                    newline,
                    IndentSourceLines(normalizedBody, "    "));
            }

            if (string.Equals(blockKind, "ob", StringComparison.OrdinalIgnoreCase))
            {
                return string.Format(
                    CultureInfo.InvariantCulture,
                    "ORGANIZATION_BLOCK {0}{1}VERSION : 0.1{1}BEGIN{1}{2}{1}END_ORGANIZATION_BLOCK",
                    QuotePlcIdentifier(name),
                    newline,
                    IndentSourceLines(normalizedBody, "    "));
            }

            if (string.Equals(blockKind, "global_db", StringComparison.OrdinalIgnoreCase))
            {
                return string.Format(
                    CultureInfo.InvariantCulture,
                    "DATA_BLOCK {0}{1}VERSION : 0.1{1}   STRUCT{1}   END_STRUCT;{1}BEGIN{1}END_DATA_BLOCK",
                    QuotePlcIdentifier(name),
                    newline);
            }

            throw new AdapterException(
                "unsupported_live_operation",
                "Live source-backed block creation only supports ob, fb, fc, and global_db.",
                new Dictionary<string, object> { { "block_kind", blockKind } });
        }

        private string ReplaceBlockBodyInSource(string sourceText, string requestedBody)
        {
            var normalizedSource = NormalizeLineEndings(sourceText);
            var beginIndex = normalizedSource.IndexOf("BEGIN", StringComparison.OrdinalIgnoreCase);
            if (beginIndex < 0)
            {
                throw new AdapterException(
                    "unsupported_live_operation",
                    "The generated source does not contain a BEGIN section that can be replaced.",
                    new Dictionary<string, object> { { "source_preview", normalizedSource } });
            }

            var bodyStart = normalizedSource.IndexOf('\n', beginIndex);
            if (bodyStart < 0)
            {
                throw new AdapterException(
                    "unsupported_live_operation",
                    "The generated source does not contain a writable block body section.",
                    new Dictionary<string, object> { { "source_preview", normalizedSource } });
            }

            bodyStart += 1;
            var endIndex = FindFirstSourceTokenIndex(
                normalizedSource,
                "END_FUNCTION_BLOCK",
                "END_FUNCTION",
                "END_ORGANIZATION_BLOCK");
            if (endIndex < 0 || endIndex < bodyStart)
            {
                throw new AdapterException(
                    "unsupported_live_operation",
                    "The generated source does not contain a supported executable-block terminator.",
                    new Dictionary<string, object> { { "source_preview", normalizedSource } });
            }

            var rebuilt = normalizedSource.Substring(0, bodyStart) +
                IndentSourceLines(PrepareBlockBodyForSource(requestedBody), "    ") + "\n" +
                normalizedSource.Substring(endIndex);
            return RestoreLineEndings(rebuilt, sourceText);
        }

        private string ReplaceDbMembersInSource(string sourceText, List<Dictionary<string, object>> members)
        {
            var normalizedSource = NormalizeLineEndings(sourceText);
            var structIndex = normalizedSource.IndexOf("STRUCT", StringComparison.OrdinalIgnoreCase);
            if (structIndex >= 0)
            {
                var membersStart = normalizedSource.IndexOf('\n', structIndex);
                if (membersStart < 0)
                {
                    throw new AdapterException(
                        "unsupported_live_operation",
                        "The generated source does not contain a writable STRUCT body.",
                        new Dictionary<string, object> { { "source_preview", normalizedSource } });
                }

                membersStart += 1;
                var endStructIndex = normalizedSource.IndexOf("END_STRUCT;", membersStart, StringComparison.OrdinalIgnoreCase);
                if (endStructIndex < 0)
                {
                    throw new AdapterException(
                        "unsupported_live_operation",
                        "The generated source does not contain an END_STRUCT marker.",
                        new Dictionary<string, object> { { "source_preview", normalizedSource } });
                }

                var rebuiltStruct = normalizedSource.Substring(0, membersStart) +
                    BuildMemberSourceLines(members, "\n", "      ") +
                    normalizedSource.Substring(endStructIndex);
                return RestoreLineEndings(rebuiltStruct, sourceText);
            }

            var varIndex = FindMemberSectionStartIndex(normalizedSource);
            if (varIndex >= 0)
            {
                var membersStart = normalizedSource.IndexOf('\n', varIndex);
                if (membersStart < 0)
                {
                    throw new AdapterException(
                        "unsupported_live_operation",
                        "The generated source does not contain a writable VAR body.",
                        new Dictionary<string, object> { { "source_preview", normalizedSource } });
                }

                membersStart += 1;
                var endVarIndex = normalizedSource.IndexOf("END_VAR", membersStart, StringComparison.OrdinalIgnoreCase);
                if (endVarIndex < 0)
                {
                    throw new AdapterException(
                        "unsupported_live_operation",
                        "The generated source does not contain an END_VAR marker.",
                        new Dictionary<string, object> { { "source_preview", normalizedSource } });
                }

                var rebuiltVar = normalizedSource.Substring(0, membersStart) +
                    BuildMemberSourceLines(members, "\n", "      ") +
                    normalizedSource.Substring(endVarIndex);
                return RestoreLineEndings(rebuiltVar, sourceText);
            }

            var beginIndex = normalizedSource.IndexOf("BEGIN", StringComparison.OrdinalIgnoreCase);
            if (beginIndex >= 0)
            {
                var insertedVarSection =
                    "   VAR\n" +
                    BuildMemberSourceLines(members, "\n", "      ") +
                    "   END_VAR\n";
                var rebuilt = normalizedSource.Substring(0, beginIndex) +
                    insertedVarSection +
                    normalizedSource.Substring(beginIndex);
                return RestoreLineEndings(rebuilt, sourceText);
            }

            throw new AdapterException(
                "unsupported_live_operation",
                "The generated source does not contain a supported DB declaration section that can be replaced.",
                new Dictionary<string, object> { { "source_preview", normalizedSource } });
        }

        private string ExtractBlockBodyFromSource(string sourceText)
        {
            var normalizedSource = NormalizeLineEndings(sourceText);
            var beginIndex = normalizedSource.IndexOf("BEGIN", StringComparison.OrdinalIgnoreCase);
            if (beginIndex < 0)
            {
                return string.Empty;
            }

            var bodyStart = normalizedSource.IndexOf('\n', beginIndex);
            if (bodyStart < 0)
            {
                return string.Empty;
            }

            bodyStart += 1;
            var endIndex = FindFirstSourceTokenIndex(
                normalizedSource,
                "END_FUNCTION_BLOCK",
                "END_FUNCTION",
                "END_ORGANIZATION_BLOCK");
            if (endIndex < 0 || endIndex < bodyStart)
            {
                return string.Empty;
            }

            return normalizedSource.Substring(bodyStart, endIndex - bodyStart).Trim();
        }

        private string PrepareBlockBodyForSource(string body)
        {
            var normalizedBody = NormalizeLineEndings(body ?? string.Empty);
            if (string.IsNullOrWhiteSpace(normalizedBody))
            {
                return "// Auto-generated by AIPLC";
            }

            return string.Join(
                "\n",
                normalizedBody.Split('\n')
                    .Select(line => line.TrimEnd())
                    .SkipWhile(string.IsNullOrWhiteSpace)
                    .Reverse()
                    .SkipWhile(string.IsNullOrWhiteSpace)
                    .Reverse());
        }

        private string NormalizeSourceBodyText(string text)
        {
            return string.Join(
                "\n",
                NormalizeLineEndings(text ?? string.Empty)
                    .Split('\n')
                    .Select(line => line.Trim())
                    .SkipWhile(string.IsNullOrWhiteSpace)
                    .Reverse()
                    .SkipWhile(string.IsNullOrWhiteSpace)
                    .Reverse());
        }

        private string IndentSourceLines(string text, string indent)
        {
            if (string.IsNullOrWhiteSpace(text))
            {
                return indent + "// Auto-generated by AIPLC";
            }

            return string.Join(
                "\n",
                NormalizeLineEndings(text)
                    .Split('\n')
                    .Select(line => string.IsNullOrWhiteSpace(line) ? string.Empty : indent + line.TrimEnd()));
        }

        private int FindFirstSourceTokenIndex(string sourceText, params string[] tokens)
        {
            var indexes = tokens
                .Select(token => sourceText.IndexOf(token, StringComparison.OrdinalIgnoreCase))
                .Where(index => index >= 0)
                .OrderBy(index => index)
                .ToList();
            return indexes.Count == 0 ? -1 : indexes[0];
        }

        private string NormalizeLineEndings(string text)
        {
            return (text ?? string.Empty).Replace("\r\n", "\n").Replace('\r', '\n');
        }

        private string RestoreLineEndings(string normalizedText, string originalText)
        {
            return normalizedText.Replace("\n", originalText != null && originalText.Contains("\r\n") ? "\r\n" : Environment.NewLine);
        }

        private string JoinRequestedMemberNames(List<object> members)
        {
            return string.Join(
                "|",
                members.Select(ToMemberDictionary)
                    .Select(member => GetRequiredString(member, "name")));
        }

        private string JoinReadBackMemberNames(List<Dictionary<string, object>> members)
        {
            return string.Join(
                "|",
                members.Select(member => Convert.ToString(member["name"], CultureInfo.InvariantCulture) ?? string.Empty));
        }

        private Dictionary<string, object> ToMemberDictionary(object source)
        {
            var dictionary = source as Dictionary<string, object>;
            if (dictionary == null)
            {
                throw new AdapterException(
                    "missing_parameter",
                    "A member definition is not a JSON object.",
                    new Dictionary<string, object> { { "value", source } });
            }

            return dictionary;
        }

        private string BuildMemberSourceLines(List<Dictionary<string, object>> members, string newline, string indent)
        {
            var builder = new StringBuilder();
            foreach (var member in members)
            {
                var comment = GetString(member, "comment");
                if (!string.IsNullOrWhiteSpace(comment))
                {
                    builder.Append(indent).Append("// ").Append(comment).Append(newline);
                }

                builder.Append(indent)
                    .Append(QuotePlcIdentifier(GetRequiredString(member, "name")))
                    .Append(" : ")
                    .Append(GetRequiredString(member, "data_type_name"));
                var initialValue = GetString(member, "initial_value");
                if (!string.IsNullOrWhiteSpace(initialValue))
                {
                    builder.Append(" := ").Append(initialValue);
                }

                builder.Append(';').Append(newline);
            }

            return builder.ToString();
        }

        private int FindMemberSectionStartIndex(string sourceText)
        {
            var normalizedSource = NormalizeLineEndings(sourceText);
            var searchIndex = 0;
            while (searchIndex < normalizedSource.Length)
            {
                var varIndex = normalizedSource.IndexOf("VAR", searchIndex, StringComparison.OrdinalIgnoreCase);
                if (varIndex < 0)
                {
                    return -1;
                }

                var lineStart = varIndex == 0 ? 0 : normalizedSource.LastIndexOf('\n', varIndex - 1) + 1;
                var lineEnd = normalizedSource.IndexOf('\n', varIndex);
                if (lineEnd < 0)
                {
                    lineEnd = normalizedSource.Length;
                }

                var line = normalizedSource.Substring(lineStart, lineEnd - lineStart).Trim();
                if (string.Equals(line, "VAR", StringComparison.OrdinalIgnoreCase) ||
                    string.Equals(line, "VAR RETAIN", StringComparison.OrdinalIgnoreCase) ||
                    string.Equals(line, "VAR_RETAIN", StringComparison.OrdinalIgnoreCase))
                {
                    return varIndex;
                }

                searchIndex = lineEnd;
            }

            return -1;
        }

        private List<Dictionary<string, object>> MergeDbMembers(
            List<Dictionary<string, object>> existingMembers,
            List<object> requestedMembers,
            bool replaceExisting)
        {
            if (replaceExisting)
            {
                return requestedMembers.Select(ToMemberDictionary).ToList();
            }

            var merged = new List<Dictionary<string, object>>(existingMembers);
            foreach (var requestedMember in requestedMembers.Select(ToMemberDictionary))
            {
                var requestedName = GetRequiredString(requestedMember, "name");
                var existingIndex = merged.FindIndex(
                    member => string.Equals(
                        Convert.ToString(member["name"], CultureInfo.InvariantCulture),
                        requestedName,
                        StringComparison.OrdinalIgnoreCase));
                if (existingIndex >= 0)
                {
                    merged[existingIndex] = requestedMember;
                }
                else
                {
                    merged.Add(requestedMember);
                }
            }

            return merged;
        }

        private List<Dictionary<string, object>> DescribeDbMembers(object dataBlock)
        {
            try
            {
                var plcSoftware = ResolveOwningPlcSoftware(dataBlock);
                var sourceMembers = plcSoftware == null
                    ? new List<Dictionary<string, object>>()
                    : ParseDbMembersFromSource(GenerateSourceText(plcSoftware, new[] { dataBlock }));
                if (sourceMembers.Count > 0)
                {
                    return sourceMembers;
                }
            }
            catch
            {
            }

            var blockInterface = TryGetPropertyValue(dataBlock, "Interface");
            return blockInterface == null
                ? new List<Dictionary<string, object>>()
                : DescribeDirectInterfaceMembers(blockInterface);
        }

        private List<Dictionary<string, object>> DescribeDirectInterfaceMembers(object container)
        {
            var members = new List<Dictionary<string, object>>();
            var seen = new HashSet<string>(StringComparer.Ordinal);
            CollectDirectInterfaceMembers(container, members, seen, 0);
            return members;
        }

        private void CollectDirectInterfaceMembers(
            object container,
            List<Dictionary<string, object>> members,
            HashSet<string> seen,
            int depth)
        {
            if (container == null || depth > 6)
            {
                return;
            }

            foreach (var member in EnumerateObjects(
                TryGetFirstPropertyValue(
                    container,
                    "Members",
                    "MemberDefinitions",
                    "Elements")))
            {
                var key = GetStableIdentityKey(member);
                if (seen.Add(key))
                {
                    members.Add(DescribeDataTypeMember(member));
                }
            }

            foreach (var child in EnumerateObjects(
                TryGetFirstPropertyValue(
                    container,
                    "Sections",
                    "Groups",
                    "Subgroups")))
            {
                CollectDirectInterfaceMembers(child, members, seen, depth + 1);
            }
        }

        private List<Dictionary<string, object>> ParseDbMembersFromSource(string sourceText)
        {
            var normalized = NormalizeLineEndings(sourceText);
            var members = new List<Dictionary<string, object>>();
            if (string.IsNullOrWhiteSpace(normalized))
            {
                return members;
            }

            var inMembers = false;
            foreach (var rawLine in normalized.Split('\n'))
            {
                var line = rawLine.Trim();
                if (string.IsNullOrWhiteSpace(line) ||
                    line.StartsWith("//", StringComparison.Ordinal) ||
                    line.StartsWith("(*", StringComparison.Ordinal))
                {
                    continue;
                }

                if (!inMembers)
                {
                    if (string.Equals(line, "STRUCT", StringComparison.OrdinalIgnoreCase) ||
                        string.Equals(line, "VAR", StringComparison.OrdinalIgnoreCase) ||
                        string.Equals(line, "VAR RETAIN", StringComparison.OrdinalIgnoreCase) ||
                        string.Equals(line, "VAR_RETAIN", StringComparison.OrdinalIgnoreCase))
                    {
                        inMembers = true;
                    }

                    continue;
                }

                if (line.StartsWith("END_STRUCT", StringComparison.OrdinalIgnoreCase) ||
                    line.StartsWith("END_VAR", StringComparison.OrdinalIgnoreCase))
                {
                    break;
                }

                var colonIndex = line.IndexOf(':');
                if (colonIndex <= 0)
                {
                    continue;
                }

                var name = UnquotePlcIdentifier(line.Substring(0, colonIndex).Trim());
                if (string.IsNullOrWhiteSpace(name))
                {
                    continue;
                }

                var rightSide = line.Substring(colonIndex + 1).Trim();
                if (rightSide.EndsWith(";", StringComparison.Ordinal))
                {
                    rightSide = rightSide.Substring(0, rightSide.Length - 1).Trim();
                }

                string initialValue = null;
                var assignmentIndex = rightSide.IndexOf(":=", StringComparison.Ordinal);
                if (assignmentIndex >= 0)
                {
                    initialValue = rightSide.Substring(assignmentIndex + 2).Trim();
                    rightSide = rightSide.Substring(0, assignmentIndex).Trim();
                }

                members.Add(
                    new Dictionary<string, object>
                    {
                        { "name", name },
                        { "data_type_name", rightSide },
                        { "comment", null },
                        { "initial_value", string.IsNullOrWhiteSpace(initialValue) ? null : initialValue },
                    });
            }

            return members;
        }

        private string BuildBlockCallStatement(
            string calleeName,
            string instanceDbName,
            string comment,
            List<object> parameterBindings)
        {
            var targetName = !string.IsNullOrWhiteSpace(instanceDbName)
                ? QuotePlcIdentifier(instanceDbName)
                : QuotePlcIdentifier(calleeName);
            var lines = new List<string>();
            if (!string.IsNullOrWhiteSpace(comment))
            {
                lines.Add("// " + comment);
            }

            if (parameterBindings.Count == 0)
            {
                lines.Add(targetName + "();");
                return string.Join("\n", lines);
            }

            lines.Add(targetName + "(");
            for (var index = 0; index < parameterBindings.Count; index++)
            {
                var binding = ToMemberDictionary(parameterBindings[index]);
                lines.Add(
                    string.Format(
                        CultureInfo.InvariantCulture,
                        "    {0} := {1}{2}",
                        GetRequiredString(binding, "parameter"),
                        GetRequiredString(binding, "expression"),
                        index == parameterBindings.Count - 1 ? string.Empty : ","));
            }

            lines.Add(");");
            return string.Join("\n", lines);
        }

        private string AppendBlockCallToBody(string existingBody, string callStatement)
        {
            var normalizedExisting = NormalizeLineEndings(existingBody ?? string.Empty).Trim();
            return string.IsNullOrWhiteSpace(normalizedExisting)
                ? callStatement
                : normalizedExisting + "\n\n" + callStatement;
        }

        private object EnsureInstanceDbExists(object plcSoftware, string name, string instanceOfName)
        {
            var existing = FindBlockByName(plcSoftware, name);
            if (existing != null)
            {
                return existing;
            }

            var composition = ResolveBlockComposition(plcSoftware);
            var blockNumber = NextAvailableDbNumber(plcSoftware);
            Exception lastException = null;
            foreach (var arguments in new[]
            {
                new object[] { name, true, blockNumber, instanceOfName },
                new object[] { name, false, blockNumber, instanceOfName },
                new object[] { name, blockNumber, instanceOfName },
            })
            {
                try
                {
                    return InvokeCompatibleMethod(composition, "CreateInstanceDB", arguments);
                }
                catch (TargetInvocationException ex)
                {
                    lastException = ex.InnerException ?? ex;
                }
                catch (Exception ex)
                {
                    lastException = ex;
                }
            }

            if (lastException != null)
            {
                throw lastException;
            }

            throw new AdapterException(
                "unsupported_live_operation",
                "The selected PLC software does not expose a compatible CreateInstanceDB overload.",
                new Dictionary<string, object>
                {
                    { "plc_software", DescribeEngineeringObject(plcSoftware) },
                    { "instance_db_name", name },
                    { "instance_of_name", instanceOfName },
                });
        }

        private int NextAvailableDbNumber(object plcSoftware)
        {
            var usedNumbers = EnumerateBlockGroupObjects(GetPropertyValue(plcSoftware, "BlockGroup"))
                .Where(IsDataBlock)
                .Select(block => GetNullableInt(block, "Number") ?? GetNullableIntAttribute(block, "Number") ?? 0)
                .Where(number => number > 0)
                .ToHashSet();

            var candidate = 1;
            while (usedNumbers.Contains(candidate))
            {
                candidate++;
            }

            return candidate;
        }

        private string QuotePlcIdentifier(string value)
        {
            return "\"" + (value ?? string.Empty).Replace("\"", "\"\"") + "\"";
        }

        private string UnquotePlcIdentifier(string value)
        {
            var trimmed = (value ?? string.Empty).Trim();
            if (trimmed.Length >= 2 &&
                trimmed.StartsWith("\"", StringComparison.Ordinal) &&
                trimmed.EndsWith("\"", StringComparison.Ordinal))
            {
                return trimmed.Substring(1, trimmed.Length - 2).Replace("\"\"", "\"");
            }

            return trimmed;
        }

        private string SanitizeName(string value)
        {
            var builder = new StringBuilder();
            foreach (var character in value ?? string.Empty)
            {
                builder.Append(char.IsLetterOrDigit(character) ? character : '_');
            }

            return builder.Length == 0 ? "AIPLC" : builder.ToString();
        }

        private string TryComputeExportHash(object target)
        {
            var tempPath = Path.Combine(
                Path.GetTempPath(),
                "codex-tia",
                Guid.NewGuid().ToString("N") + ".xml");
            Directory.CreateDirectory(Path.GetDirectoryName(tempPath));
            try
            {
                ExportToPath(target, tempPath);
                return File.Exists(tempPath) ? ComputeSha256(tempPath) : null;
            }
            catch
            {
                return null;
            }
            finally
            {
                if (File.Exists(tempPath))
                {
                    File.Delete(tempPath);
                }
            }
        }

        private object ResolveImportComposition(object targetGroup)
        {
            var blocks = TryGetPropertyValue(targetGroup, "Blocks");
            if (blocks != null)
            {
                return blocks;
            }

            var tagTables = TryGetPropertyValue(targetGroup, "TagTables");
            if (tagTables != null)
            {
                return tagTables;
            }

            throw new AdapterException(
                "unsupported_import_target",
                "The selected object does not expose an import composition.",
                DescribeEngineeringObject(targetGroup));
        }

        private object InvokeImport(object composition, string sourceFilePath, string conflictMode)
        {
            var fileInfo = new FileInfo(sourceFilePath);
            var importOptionsType = FindType("Siemens.Engineering.ImportOptions");
            var importOption = Enum.Parse(
                importOptionsType,
                string.Equals(conflictMode, "override", StringComparison.OrdinalIgnoreCase)
                    ? "Override"
                    : "None");
            var importMethod = composition.GetType().GetMethods()
                .FirstOrDefault(
                    method => method.Name == "Import" &&
                              method.GetParameters().Length == 2);
            if (importMethod == null)
            {
                throw new AdapterException(
                    "unsupported_import",
                    "Import is not supported for the selected composition.",
                    new Dictionary<string, object> { { "type", composition.GetType().FullName } });
            }

            return importMethod.Invoke(composition, new object[] { fileInfo, importOption });
        }

        private void ExportToPath(object target, string destinationPath)
        {
            var fileInfo = new FileInfo(destinationPath);
            Directory.CreateDirectory(fileInfo.DirectoryName);
            var exportOptionsType = TryFindType("Siemens.Engineering.ExportOptions");
            var exportOption = exportOptionsType != null ? Enum.Parse(exportOptionsType, "WithDefaults") : null;

            var exportMethod = target.GetType().GetMethods()
                .FirstOrDefault(
                    method => method.Name == "Export" &&
                              method.GetParameters().Length == 2);
            if (exportMethod != null)
            {
                exportMethod.Invoke(target, new object[] { fileInfo, exportOption });
                return;
            }

            exportMethod = target.GetType().GetMethods()
                .FirstOrDefault(
                    method => method.Name == "Export" &&
                              method.GetParameters().Length == 1);
            if (exportMethod != null)
            {
                exportMethod.Invoke(target, new object[] { fileInfo });
                return;
            }

            throw new AdapterException(
                "unsupported_export",
                "The selected object does not expose an Export method.",
                DescribeEngineeringObject(target));
        }

        private object ResolveObject(string objectId)
        {
            EnsureProjectOpen();
            if (_objectIdentifierProvider != null &&
                !IsFallbackIdentifier(objectId))
            {
                var resolved = InvokeMethod(_objectIdentifierProvider, "Find", objectId);
                if (resolved != null)
                {
                    return resolved;
                }
            }

            var fallbackResolved = ResolveFallbackObject(objectId);
            if (fallbackResolved != null)
            {
                return fallbackResolved;
            }

            if (_objectIdentifierProvider == null)
            {
                throw new AdapterException(
                    "object_identifier_provider_unavailable",
                    "Object identifier provider is unavailable for the current project.",
                    null);
            }

            throw new AdapterException(
                "object_not_found",
                "Object id could not be resolved in the current project.",
                new Dictionary<string, object> { { "object_id", objectId } });
        }

        private string GetObjectIdentifier(object target)
        {
            string identifier = null;
            if (_objectIdentifierProvider != null)
            {
                try
                {
                    identifier = SafeToString(InvokeMethod(_objectIdentifierProvider, "GetIdentifier", target));
                }
                catch
                {
                    identifier = null;
                }
            }

            if (!string.IsNullOrWhiteSpace(identifier))
            {
                return identifier;
            }

            return BuildFallbackIdentifier(target);
        }

        private object GetService(object target, string serviceTypeName, bool required)
        {
            var serviceType = FindType(serviceTypeName);
            var getServiceMethod = target.GetType().GetMethods()
                .FirstOrDefault(
                    method => method.Name == "GetService" &&
                              method.IsGenericMethodDefinition &&
                              method.GetParameters().Length == 0);
            if (getServiceMethod == null)
            {
                if (!required)
                {
                    return null;
                }

                throw new AdapterException(
                    "unsupported_service_lookup",
                    "GetService<T>() was not found on the target type.",
                    new Dictionary<string, object> { { "type", target.GetType().FullName } });
            }

            var generic = getServiceMethod.MakeGenericMethod(serviceType);
            var service = generic.Invoke(target, null);
            if (service == null && required)
            {
                throw new AdapterException(
                    "missing_service",
                    "Requested TIA service is not available on the target object.",
                    new Dictionary<string, object>
                    {
                        { "service_type", serviceTypeName },
                        { "target_type", target.GetType().FullName },
                    });
            }

            return service;
        }

        private object TryGetSoftware(object deviceItem)
        {
            try
            {
                var container = GetService(
                    deviceItem,
                    "Siemens.Engineering.HW.Features.SoftwareContainer",
                    required: false);
                return container == null ? null : TryGetPropertyValue(container, "Software");
            }
            catch
            {
                return null;
            }
        }

        private object TryGetPlcSoftware(object deviceItem)
        {
            var software = TryGetSoftware(deviceItem);
            return IsPlcSoftware(software) ? software : null;
        }

        private object TryGetHmiSoftware(object deviceItem)
        {
            var software = TryGetSoftware(deviceItem);
            return IsHmiSoftware(software) ? software : null;
        }

        private bool IsPlcSoftware(object software)
        {
            return software != null &&
                TypeNameContains(software, "PlcSoftware");
        }

        private bool IsHmiSoftware(object software)
        {
            return software != null &&
                (TypeNameContains(software, "Hmi") || TypeNameContains(software, "WinCC"));
        }

        private string ClassifyDeviceItem(object deviceItem)
        {
            if (TryGetPlcSoftware(deviceItem) != null)
            {
                return "plc";
            }

            if (TryGetHmiSoftware(deviceItem) != null)
            {
                return "hmi";
            }

            return deviceItem.GetType().Name;
        }

        private object BuildVerifiedMutationResponse(
            object target,
            List<Dictionary<string, object>> changes,
            List<Dictionary<string, object>> verifiedFields)
        {
            var verified = verifiedFields.All(field => Equals(field["expected"], field["actual"]));
            if (!verified)
            {
                throw new AdapterException(
                    "verification_failed",
                    "The live Openness call completed but read-back verification did not match the requested values.",
                    DescribeEngineeringObject(target));
            }

            return new Dictionary<string, object>
            {
                {
                    "touched_objects",
                    new List<Dictionary<string, object>>
                    {
                        new Dictionary<string, object>
                        {
                            { "object", DescribeEngineeringObject(target) },
                            { "changes", changes },
                        }
                    }
                },
                {
                    "verification",
                    new Dictionary<string, object>
                    {
                        { "verified", true },
                        { "strategy", "read_back" },
                        { "checked_fields", verifiedFields },
                        { "exported_sha256", null },
                    }
                },
            };
        }

        private void EnsureFieldValue(
            object target,
            string fieldName,
            object expectedValue,
            List<Dictionary<string, object>> changes,
            List<Dictionary<string, object>> verifiedFields)
        {
            if (expectedValue == null)
            {
                return;
            }

            var normalizedExpected = NormalizeFieldValue(expectedValue);
            var before = ReadFieldValue(target, fieldName);
            if (!Equals(before, normalizedExpected))
            {
                SetFieldValue(target, fieldName, expectedValue);
            }

            var after = ReadFieldValue(target, fieldName);
            if (!Equals(before, after))
            {
                changes.Add(
                    new Dictionary<string, object>
                    {
                        { "field", ToSnakeCase(fieldName) },
                        { "before", before },
                        { "after", after },
                    });
            }

            verifiedFields.Add(
                new Dictionary<string, object>
                {
                    { "field", ToSnakeCase(fieldName) },
                    { "expected", normalizedExpected },
                    { "actual", after },
                });
        }

        private object FindDataTypeRoot(object plcSoftware)
        {
            return TryGetFirstPropertyValue(
                plcSoftware,
                "TypeGroup",
                "PlcTypeGroup",
                "PlcTypeSystemGroup",
                "DataTypeGroup",
                "PlcTypes") ??
                TryGetPropertyValueByFragments(plcSoftware, "Type", "Group");
        }

        private void WalkDataTypeGroup(
            object group,
            List<Dictionary<string, object>> dataTypes,
            HashSet<string> seen)
        {
            if (group == null)
            {
                return;
            }

            foreach (var dataType in EnumerateObjects(
                TryGetFirstPropertyValue(
                    group,
                    "Types",
                    "DataTypes",
                    "PlcTypes",
                    "SystemTypes")))
            {
                var key = GetStableIdentityKey(dataType);
                if (seen.Add(key))
                {
                    dataTypes.Add(DescribeDataType(dataType));
                }
            }

            foreach (var dataType in EnumerateObjects(group))
            {
                if (!LooksLikeDataTypeObject(dataType))
                {
                    continue;
                }

                var key = GetStableIdentityKey(dataType);
                if (seen.Add(key))
                {
                    dataTypes.Add(DescribeDataType(dataType));
                }
            }

            foreach (var childGroup in EnumerateObjects(
                TryGetFirstPropertyValue(
                    group,
                    "Groups",
                    "TypeGroups",
                    "Subgroups",
                    "SystemTypeGroups")))
            {
                WalkDataTypeGroup(childGroup, dataTypes, seen);
            }
        }

        private Dictionary<string, object> DescribeDataType(object dataType)
        {
            return new Dictionary<string, object>
            {
                { "object", DescribeEngineeringObject(dataType) },
                { "data_type_kind", InferDataTypeKind(dataType) },
                {
                    "comment",
                    GetString(dataType, "Comment") ??
                        SafeToString(TryGetAttributeValue(dataType, "Comment"))
                },
                { "members", DescribeDataTypeMembers(dataType) },
            };
        }

        private List<Dictionary<string, object>> DescribeDataTypeMembers(object dataType)
        {
            var members = new List<Dictionary<string, object>>();
            var seen = new HashSet<string>(StringComparer.Ordinal);
            CollectDataTypeMembers(dataType, members, seen, 0);
            if (members.Count == 0 &&
                string.Equals(InferDataTypeKind(dataType), "udt", StringComparison.OrdinalIgnoreCase))
            {
                return DescribeDataTypeMembersFromSource(dataType);
            }

            return members;
        }

        private void CollectDataTypeMembers(
            object container,
            List<Dictionary<string, object>> members,
            HashSet<string> seen,
            int depth)
        {
            if (container == null || depth > 6)
            {
                return;
            }

            if (LooksLikeDataTypeMember(container))
            {
                var key = GetStableIdentityKey(container);
                if (seen.Add(key))
                {
                    members.Add(DescribeDataTypeMember(container));
                }
            }

            foreach (var member in EnumerateObjects(
                TryGetFirstPropertyValue(
                    container,
                    "Members",
                    "MemberDefinitions",
                    "Elements")))
            {
                var key = GetStableIdentityKey(member);
                if (seen.Add(key))
                {
                    members.Add(DescribeDataTypeMember(member));
                }
            }

            foreach (var child in EnumerateObjects(
                TryGetFirstPropertyValue(
                    container,
                    "Sections",
                    "Groups",
                    "Subgroups")))
            {
                CollectDataTypeMembers(child, members, seen, depth + 1);
            }

            CollectDataTypeMembers(TryGetFirstPropertyValue(container, "Interface", "Structure"), members, seen, depth + 1);
        }

        private bool LooksLikeDataTypeMember(object target)
        {
            return target != null &&
                !TypeNameContains(target, "TypeGroup") &&
                (TryGetPropertyValue(target, "DataTypeName") != null ||
                 TryGetPropertyValue(target, "DataType") != null);
        }

        private bool LooksLikeDataTypeObject(object target)
        {
            return target != null &&
                !TypeNameContains(target, "Group") &&
                !TypeNameContains(target, "Composition") &&
                (TypeNameContains(target, "Type") ||
                 TypeNameContains(target, "Enum") ||
                 TypeNameContains(target, "Array"));
        }

        private Dictionary<string, object> DescribeDataTypeMember(object member)
        {
            return new Dictionary<string, object>
            {
                { "name", ReadName(member) },
                {
                    "data_type_name",
                    GetString(member, "DataTypeName") ??
                        GetString(member, "TypeName") ??
                        ReadNameOrNull(TryGetPropertyValue(member, "DataType"))
                },
                {
                    "comment",
                    TryGetOptionalString(member, "Comment") ??
                        SafeToString(TryGetOptionalAttributeValue(member, "Comment"))
                },
                {
                    "initial_value",
                    TryGetOptionalString(member, "InitialValue") ??
                        TryGetOptionalString(member, "StartValue") ??
                        SafeToString(TryGetOptionalAttributeValue(member, "InitialValue"))
                },
            };
        }

        private List<Dictionary<string, object>> DescribeDataTypeMembersFromSource(object dataType)
        {
            try
            {
                var plcSoftware = ResolveOwningPlcSoftware(dataType);
                if (plcSoftware == null)
                {
                    return new List<Dictionary<string, object>>();
                }

                return ParseUdtMembersFromSource(GenerateSourceText(plcSoftware, new[] { dataType }));
            }
            catch
            {
                return new List<Dictionary<string, object>>();
            }
        }

        private List<Dictionary<string, object>> ParseUdtMembersFromSource(string sourceText)
        {
            var members = new List<Dictionary<string, object>>();
            var normalized = NormalizeLineEndings(sourceText);
            if (string.IsNullOrWhiteSpace(normalized))
            {
                return members;
            }

            var structDepth = 0;
            foreach (var rawLine in normalized.Split('\n'))
            {
                var line = rawLine.Trim();
                if (string.IsNullOrWhiteSpace(line) ||
                    line.StartsWith("//", StringComparison.Ordinal) ||
                    line.StartsWith("(*", StringComparison.Ordinal))
                {
                    continue;
                }

                if (string.Equals(line, "STRUCT", StringComparison.OrdinalIgnoreCase))
                {
                    structDepth++;
                    continue;
                }

                if (line.StartsWith("END_STRUCT", StringComparison.OrdinalIgnoreCase))
                {
                    structDepth--;
                    if (structDepth <= 0)
                    {
                        break;
                    }

                    continue;
                }

                if (structDepth != 1)
                {
                    continue;
                }

                var colonIndex = line.IndexOf(':');
                if (colonIndex <= 0)
                {
                    continue;
                }

                var name = UnquotePlcIdentifier(line.Substring(0, colonIndex).Trim());
                if (string.IsNullOrWhiteSpace(name))
                {
                    continue;
                }

                var rightSide = line.Substring(colonIndex + 1).Trim();
                if (rightSide.EndsWith(";", StringComparison.Ordinal))
                {
                    rightSide = rightSide.Substring(0, rightSide.Length - 1).Trim();
                }

                string initialValue = null;
                var assignmentIndex = rightSide.IndexOf(":=", StringComparison.Ordinal);
                if (assignmentIndex >= 0)
                {
                    initialValue = rightSide.Substring(assignmentIndex + 2).Trim();
                    rightSide = rightSide.Substring(0, assignmentIndex).Trim();
                }

                members.Add(
                    new Dictionary<string, object>
                    {
                        { "name", name },
                        { "data_type_name", rightSide },
                        { "comment", null },
                        { "initial_value", string.IsNullOrWhiteSpace(initialValue) ? null : initialValue },
                    });
            }

            return members;
        }

        private string InferDataTypeKind(object dataType)
        {
            if (TypeNameContains(dataType, "Enum"))
            {
                return "enum";
            }

            if (TypeNameContains(dataType, "Array"))
            {
                return "array";
            }

            if (TypeNameContains(dataType, "Struct") ||
                TypeNameContains(dataType, "Udt") ||
                TypeNameContains(dataType, "Type"))
            {
                return "udt";
            }

            return dataType.GetType().Name;
        }

        private object FindTechnologyObjectRoot(object plcSoftware)
        {
            return TryGetFirstPropertyValue(
                plcSoftware,
                "TechnologicalObjectGroup",
                "TechnologicalObjectsGroup",
                "TechnologyObjectGroup",
                "TechnologyObjects") ??
                TryGetPropertyValueByFragments(plcSoftware, "Technology");
        }

        private void WalkTechnologyObjectGroup(
            object group,
            List<Dictionary<string, object>> technologyObjects,
            HashSet<string> seen)
        {
            if (group == null)
            {
                return;
            }

            if (TypeNameContains(group, "Technology") &&
                !TypeNameContains(group, "Group") &&
                !TypeNameContains(group, "Composition"))
            {
                var key = GetStableIdentityKey(group);
                if (seen.Add(key))
                {
                    technologyObjects.Add(DescribeTechnologyObject(group));
                }
            }

            foreach (var item in EnumerateObjects(
                TryGetFirstPropertyValue(
                    group,
                    "TechnologicalObjects",
                    "TechnologyObjects",
                    "Objects")))
            {
                var key = GetStableIdentityKey(item);
                if (seen.Add(key))
                {
                    technologyObjects.Add(DescribeTechnologyObject(item));
                }
            }

            foreach (var item in EnumerateObjects(group))
            {
                if (!TypeNameContains(item, "Technology") ||
                    TypeNameContains(item, "Group") ||
                    TypeNameContains(item, "Composition"))
                {
                    continue;
                }

                var key = GetStableIdentityKey(item);
                if (seen.Add(key))
                {
                    technologyObjects.Add(DescribeTechnologyObject(item));
                }
            }

            foreach (var childGroup in EnumerateObjects(
                TryGetFirstPropertyValue(
                    group,
                    "Groups",
                    "Subgroups",
                    "TechnologicalObjectGroups",
                    "TechnologyObjectGroups")))
            {
                WalkTechnologyObjectGroup(childGroup, technologyObjects, seen);
            }
        }

        private Dictionary<string, object> DescribeTechnologyObject(object technologyObject)
        {
            return new Dictionary<string, object>
            {
                { "object", DescribeEngineeringObject(technologyObject) },
                {
                    "technology_type",
                    GetString(technologyObject, "TechnologyType") ??
                        GetString(technologyObject, "TypeIdentifier") ??
                        technologyObject.GetType().Name
                },
                {
                    "bound_axis",
                    ReadNameOrNull(TryGetFirstPropertyValue(technologyObject, "Axis", "BoundAxis")) ??
                        GetString(technologyObject, "AxisName")
                },
            };
        }

        private object FindWatchTableRoot(object plcSoftware)
        {
            return TryGetFirstPropertyValue(
                plcSoftware,
                "WatchAndForceTableSystemGroup",
                "WatchTableGroup",
                "WatchTables") ??
                TryGetPropertyValueByFragments(plcSoftware, "Watch");
        }

        private void WalkWatchTableGroup(
            object group,
            List<Dictionary<string, object>> watchTables,
            HashSet<string> seen)
        {
            if (group == null)
            {
                return;
            }

            if (TypeNameContains(group, "Watch") &&
                !TypeNameContains(group, "Group") &&
                !TypeNameContains(group, "Composition"))
            {
                var key = GetStableIdentityKey(group);
                if (seen.Add(key))
                {
                    watchTables.Add(DescribeWatchTable(group));
                }
            }

            foreach (var table in EnumerateObjects(
                TryGetFirstPropertyValue(
                    group,
                    "WatchTables",
                    "Tables")))
            {
                var key = GetStableIdentityKey(table);
                if (seen.Add(key))
                {
                    watchTables.Add(DescribeWatchTable(table));
                }
            }

            foreach (var table in EnumerateObjects(group))
            {
                if (!TypeNameContains(table, "Watch") ||
                    TypeNameContains(table, "Group") ||
                    TypeNameContains(table, "Composition") ||
                    TypeNameContains(table, "Expression"))
                {
                    continue;
                }

                var key = GetStableIdentityKey(table);
                if (seen.Add(key))
                {
                    watchTables.Add(DescribeWatchTable(table));
                }
            }

            foreach (var childGroup in EnumerateObjects(
                TryGetFirstPropertyValue(
                    group,
                    "Groups",
                    "Subgroups",
                    "Folders")))
            {
                WalkWatchTableGroup(childGroup, watchTables, seen);
            }
        }

        private Dictionary<string, object> DescribeWatchTable(object watchTable)
        {
            var exportedExpressions = TryDescribeWatchTableExpressionsFromExport(watchTable);
            if (exportedExpressions != null)
            {
                return new Dictionary<string, object>
                {
                    { "object", DescribeEngineeringObject(watchTable) },
                    { "expressions", exportedExpressions },
                };
            }

            var expressions = new List<Dictionary<string, object>>();
            var seen = new HashSet<string>(StringComparer.Ordinal);
            foreach (var composition in EnumerateWatchExpressionCompositions(watchTable))
            {
                foreach (var expression in EnumerateObjects(composition))
                {
                    var key = GetStableIdentityKey(expression);
                    if (!seen.Add(key))
                    {
                        continue;
                    }

                    var described = DescribeWatchTableExpression(expression);
                    if (described != null)
                    {
                        expressions.Add(described);
                    }
                }
            }

            return new Dictionary<string, object>
            {
                { "object", DescribeEngineeringObject(watchTable) },
                { "expressions", expressions },
            };
        }

        private List<Dictionary<string, object>> TryDescribeWatchTableExpressionsFromExport(object watchTable)
        {
            var exportPath = Path.Combine(
                Path.GetTempPath(),
                "codex-tia-watch-exports",
                Guid.NewGuid().ToString("N") + ".xml");
            Directory.CreateDirectory(Path.GetDirectoryName(exportPath));
            try
            {
                ExportToPath(watchTable, exportPath);
                var document = XDocument.Load(exportPath);
                var watchTableElement = document.Descendants()
                    .FirstOrDefault(
                        element => string.Equals(
                            element.Name.LocalName,
                            "SW.WatchAndForceTables.PlcWatchTable",
                            StringComparison.Ordinal));
                if (watchTableElement == null)
                {
                    return null;
                }

                var objectList = watchTableElement.Element("ObjectList");
                if (objectList == null)
                {
                    return new List<Dictionary<string, object>>();
                }

                return objectList.Elements()
                    .Where(
                        element => string.Equals(
                            element.Name.LocalName,
                            "SW.WatchAndForceTables.PlcWatchTableEntry",
                            StringComparison.Ordinal))
                    .Select(
                        element =>
                        {
                            var attributeList = element.Element("AttributeList");
                            var expressionText =
                                attributeList == null
                                    ? null
                                    : attributeList.Element("Address")?.Value ??
                                        attributeList.Element("Name")?.Value;
                            if (string.IsNullOrWhiteSpace(expressionText))
                            {
                                return null;
                            }

                            return new Dictionary<string, object>
                            {
                                { "expression", expressionText },
                                { "comment", null },
                            };
                        })
                    .Where(expression => expression != null)
                    .ToList();
            }
            catch
            {
                return null;
            }
            finally
            {
                if (File.Exists(exportPath))
                {
                    File.Delete(exportPath);
                }
            }
        }

        private Dictionary<string, object> DescribeWatchTableExpression(object expression)
        {
            if (TypeNameContains(expression, "CommentEntry"))
            {
                return null;
            }

            var expressionText =
                TryGetOptionalString(expression, "Expression") ??
                TryGetOptionalString(expression, "Operand") ??
                TryGetOptionalString(expression, "Address") ??
                ReadNameOrNull(expression);
            if (string.IsNullOrWhiteSpace(expressionText) ||
                string.Equals(expressionText, "PlcTableCommentEntry", StringComparison.OrdinalIgnoreCase))
            {
                return null;
            }

            return new Dictionary<string, object>
            {
                {
                    "expression",
                    expressionText
                },
                {
                    "comment",
                    TryGetOptionalString(expression, "Comment") ??
                        TryGetOptionalString(expression, "DisplayComment") ??
                        SafeToString(TryGetOptionalAttributeValue(expression, "Comment"))
                },
            };
        }

        private IEnumerable<object> EnumerateNetworkObjects()
        {
            var seen = new HashSet<string>(StringComparer.Ordinal);
            foreach (var network in EnumerateObjects(
                TryGetFirstPropertyValue(
                    _project,
                    "Subnets",
                    "Subnetworks",
                    "Networks",
                    "IoSystems")))
            {
                var key = GetStableIdentityKey(network);
                if (seen.Add(key))
                {
                    yield return network;
                }
            }
        }

        private Dictionary<string, object> DescribeNetwork(object network)
        {
            return new Dictionary<string, object>
            {
                { "object", DescribeEngineeringObject(network) },
                { "connected_object_ids", EnumerateConnectedObjectIdentifiers(network) },
            };
        }

        private List<string> EnumerateConnectedObjectIdentifiers(object network)
        {
            var connectedIds = new List<string>();
            var seen = new HashSet<string>(StringComparer.Ordinal);
            foreach (var participant in EnumerateObjects(
                TryGetFirstPropertyValue(
                    network,
                    "Nodes",
                    "Participants",
                    "Interfaces")))
            {
                var owner = TryGetFirstPropertyValue(
                    participant,
                    "Owner",
                    "DeviceItem",
                    "Parent");
                if (owner == null)
                {
                    continue;
                }

                var id = GetObjectIdentifier(owner);
                if (!string.IsNullOrWhiteSpace(id) && seen.Add(id))
                {
                    connectedIds.Add(id);
                }
            }

            return connectedIds;
        }

        private IEnumerable<object> EnumerateHmiObjects()
        {
            var seen = new HashSet<string>(StringComparer.Ordinal);
            foreach (var device in EnumerateObjects(GetPropertyValue(_project, "Devices")))
            {
                foreach (var deviceItem in FlattenDeviceItems(device))
                {
                    var hmiSoftware = TryGetHmiSoftware(deviceItem);
                    if (hmiSoftware == null)
                    {
                        continue;
                    }

                    var key = GetStableIdentityKey(hmiSoftware);
                    if (seen.Add(key))
                    {
                        yield return hmiSoftware;
                    }
                }
            }
        }

        private Dictionary<string, object> DescribeHmiObject(object hmiObject)
        {
            return new Dictionary<string, object>
            {
                { "object", DescribeEngineeringObject(hmiObject) },
                { "hmi_type", hmiObject.GetType().Name },
            };
        }

        private IEnumerable<object> EnumerateProjectPlcSoftware()
        {
            foreach (var device in EnumerateObjects(GetPropertyValue(_project, "Devices")))
            {
                foreach (var deviceItem in FlattenDeviceItems(device))
                {
                    var plcSoftware = TryGetPlcSoftware(deviceItem);
                    if (plcSoftware != null)
                    {
                        yield return plcSoftware;
                    }
                }
            }
        }

        private IEnumerable<object> EnumerateSafetyObjects(object plcSoftware)
        {
            var seen = new HashSet<string>(StringComparer.Ordinal);
            foreach (var candidate in EnumerateBlockGroupObjects(TryGetPropertyValue(plcSoftware, "BlockGroup")))
            {
                if (!TypeNameContains(candidate, "Safety"))
                {
                    continue;
                }

                var key = GetStableIdentityKey(candidate);
                if (seen.Add(key))
                {
                    yield return candidate;
                }
            }

            foreach (var property in plcSoftware.GetType().GetProperties(BindingFlags.Public | BindingFlags.Instance))
            {
                if (property.Name.IndexOf("Safety", StringComparison.OrdinalIgnoreCase) < 0)
                {
                    continue;
                }

                var value = property.GetValue(plcSoftware, null);
                if (value != null &&
                    !(value is string) &&
                    !value.GetType().IsValueType &&
                    !typeof(IEnumerable).IsAssignableFrom(value.GetType()))
                {
                    var valueKey = GetStableIdentityKey(value);
                    if (seen.Add(valueKey))
                    {
                        yield return value;
                    }
                }

                foreach (var candidate in EnumerateObjects(value))
                {
                    var key = GetStableIdentityKey(candidate);
                    if (seen.Add(key))
                    {
                        yield return candidate;
                    }
                }
            }
        }

        private Dictionary<string, object> DescribeSafetyObject(object safetyObject)
        {
            return new Dictionary<string, object>
            {
                { "object", DescribeEngineeringObject(safetyObject) },
                { "safety_type", safetyObject.GetType().Name },
            };
        }

        private object ResolveConsistencyCheckTarget(Dictionary<string, object> scope)
        {
            var scopeType = GetRequiredString(scope, "type");
            if (string.Equals(scopeType, "current_project", StringComparison.OrdinalIgnoreCase))
            {
                return _project;
            }

            if (string.Equals(scopeType, "plc_software", StringComparison.OrdinalIgnoreCase))
            {
                return ResolveObject(GetRequiredString(scope, "plc_software_id"));
            }

            if (string.Equals(scopeType, "object", StringComparison.OrdinalIgnoreCase))
            {
                return ResolveObject(GetRequiredString(scope, "object_id"));
            }

            throw new AdapterException(
                "unsupported_scope",
                "Unsupported consistency-check scope.",
                new Dictionary<string, object> { { "scope_type", scopeType } });
        }

        private void CollectConsistencyIssues(object compilerResult, List<Dictionary<string, object>> issues)
        {
            foreach (var message in EnumerateObjects(TryGetPropertyValue(compilerResult, "Messages")))
            {
                var state = SafeToString(TryGetPropertyValue(message, "State")) ?? "Unknown";
                if (string.Equals(state, "Error", StringComparison.OrdinalIgnoreCase) ||
                    string.Equals(state, "Warning", StringComparison.OrdinalIgnoreCase))
                {
                    issues.Add(BuildConsistencyIssue(message, state));
                }

                CollectConsistencyIssues(message, issues);
            }
        }

        private Dictionary<string, object> BuildConsistencyIssue(object compilerMessage, string state)
        {
            var path = SafeToString(TryGetPropertyValue(compilerMessage, "Path"));
            var resolvedObject = TryResolveObjectByCompilerPath(path);
            return new Dictionary<string, object>
            {
                { "severity", state.ToLowerInvariant() },
                {
                    "code",
                    SafeToString(TryGetPropertyValue(compilerMessage, "Code")) ??
                        SafeToString(TryGetPropertyValue(compilerMessage, "Identifier")) ??
                        "compile_message"
                },
                {
                    "message",
                    SafeToString(TryGetPropertyValue(compilerMessage, "Description")) ??
                        path ??
                        "Compiler reported an issue."
                },
                { "object", resolvedObject == null ? null : DescribeEngineeringObject(resolvedObject) },
            };
        }

        private void CollectCrossReferenceHits(
            object sourceObject,
            List<Dictionary<string, object>> references,
            HashSet<string> seen)
        {
            foreach (var referenceObject in EnumerateObjects(TryGetPropertyValue(sourceObject, "References")))
            {
                var locations = EnumerateObjects(TryGetPropertyValue(referenceObject, "Locations")).ToList();
                if (locations.Count == 0)
                {
                    var hit = DescribeCrossReferenceHit(referenceObject, null);
                    AddCrossReferenceHit(hit, references, seen);
                    continue;
                }

                foreach (var location in locations)
                {
                    var hit = DescribeCrossReferenceHit(referenceObject, location);
                    AddCrossReferenceHit(hit, references, seen);
                }
            }

            foreach (var childSource in EnumerateObjects(TryGetPropertyValue(sourceObject, "Children")))
            {
                CollectCrossReferenceHits(childSource, references, seen);
            }
        }

        private void AddCrossReferenceHit(
            Dictionary<string, object> hit,
            List<Dictionary<string, object>> references,
            HashSet<string> seen)
        {
            var hitObject = hit["object"] as Dictionary<string, object>;
            var objectId = hitObject == null ? null : GetString(hitObject, "object_id");
            var relation = GetString(hit, "relation") ?? string.Empty;
            var detail = GetString(hit, "detail") ?? string.Empty;
            var key = string.Format(
                CultureInfo.InvariantCulture,
                "{0}|{1}|{2}",
                objectId ?? string.Empty,
                relation,
                detail);
            if (seen.Add(key))
            {
                references.Add(hit);
            }
        }

        private Dictionary<string, object> DescribeCrossReferenceHit(object referenceObject, object location)
        {
            var underlyingObject = TryGetPropertyValue(referenceObject, "UnderlyingObject") ??
                TryGetPropertyValue(location, "ReferencedAs");
            var relation = SafeToString(TryGetPropertyValue(location, "ReferenceType")) ??
                SafeToString(TryGetPropertyValue(location, "Access")) ??
                "reference";
            var detailParts = new List<string>();
            var referenceLocation = GetString(location, "ReferenceLocation");
            if (!string.IsNullOrWhiteSpace(referenceLocation))
            {
                detailParts.Add(referenceLocation);
            }

            var referencedAsName = GetString(location, "ReferncedAsName") ??
                GetString(location, "ReferencedAsName");
            if (!string.IsNullOrWhiteSpace(referencedAsName))
            {
                detailParts.Add(referencedAsName);
            }

            var access = SafeToString(TryGetPropertyValue(location, "Access"));
            if (!string.IsNullOrWhiteSpace(access))
            {
                detailParts.Add("access=" + access);
            }

            return new Dictionary<string, object>
            {
                { "object", DescribeCrossReferenceObject(referenceObject, underlyingObject) },
                { "relation", relation },
                { "detail", detailParts.Count == 0 ? null : string.Join("; ", detailParts) },
            };
        }

        private Dictionary<string, object> DescribeCrossReferenceObject(object referenceObject, object underlyingObject)
        {
            if (underlyingObject != null)
            {
                return DescribeEngineeringObject(underlyingObject);
            }

            var name = GetString(referenceObject, "Name") ?? "CrossReferenceObject";
            var path = GetString(referenceObject, "Path") ?? name;
            return new Dictionary<string, object>
            {
                { "object_id", BuildSyntheticIdentifier("cross_reference", path) },
                { "kind", "cross_reference_text" },
                { "name", name },
                { "path", path },
            };
        }

        private object TryResolveObjectByCompilerPath(string path)
        {
            if (string.IsNullOrWhiteSpace(path))
            {
                return null;
            }

            return EnumerateResolvableObjects().FirstOrDefault(
                candidate => string.Equals(BuildPath(candidate), path, StringComparison.OrdinalIgnoreCase) ||
                    string.Equals(ReadName(candidate), path, StringComparison.OrdinalIgnoreCase) ||
                    string.Equals(
                        BuildPath(candidate),
                        string.Format(CultureInfo.InvariantCulture, "{0}/{1}", BuildPath(_project), path),
                        StringComparison.OrdinalIgnoreCase));
        }

        private object CreateObjectInComposition(
            object composition,
            string expectedName,
            params object[][] candidateArgumentSets)
        {
            if (composition == null)
            {
                throw new AdapterException(
                    "unsupported_live_operation",
                    "The selected Openness target does not expose a compatible child composition for this request.",
                    new Dictionary<string, object> { { "name", expectedName } });
            }

            Exception lastException = null;
            foreach (var arguments in candidateArgumentSets)
            {
                try
                {
                    var created = TryCreateObjectInComposition(composition, expectedName, arguments);
                    if (created != null)
                    {
                        return created;
                    }
                }
                catch (TargetInvocationException ex)
                {
                    lastException = ex.InnerException ?? ex;
                }
                catch (Exception ex)
                {
                    lastException = ex;
                }
            }

            if (lastException != null)
            {
                throw lastException;
            }

            throw new AdapterException(
                "unsupported_live_operation",
                "The selected Openness composition does not expose a compatible Create overload for this request.",
                new Dictionary<string, object>
                {
                    { "name", expectedName },
                    { "composition_type", composition.GetType().FullName },
                });
        }

        private object TryCreateObjectInComposition(object composition, string expectedName, object[] arguments)
        {
            foreach (var method in composition.GetType().GetMethods()
                .Where(
                    candidate => candidate.Name == "Create" &&
                        candidate.GetParameters().Length == arguments.Length))
            {
                object[] convertedArguments;
                if (!TryConvertArguments(method.GetParameters(), arguments, out convertedArguments))
                {
                    continue;
                }

                var created = method.Invoke(composition, convertedArguments);
                if (created != null)
                {
                    return created;
                }

                var createdByName = EnumerateObjects(composition).FirstOrDefault(
                    item => string.Equals(ReadName(item), expectedName, StringComparison.OrdinalIgnoreCase));
                if (createdByName != null)
                {
                    return createdByName;
                }
            }

            return null;
        }

        private bool TryConvertArguments(
            ParameterInfo[] parameters,
            object[] values,
            out object[] convertedArguments)
        {
            convertedArguments = new object[parameters.Length];
            for (var index = 0; index < parameters.Length; index++)
            {
                try
                {
                    convertedArguments[index] = ConvertArgument(values[index], parameters[index].ParameterType);
                }
                catch
                {
                    convertedArguments = null;
                    return false;
                }
            }

            return true;
        }

        private object ConvertArgument(object value, Type targetType)
        {
            if (value == null)
            {
                if (!targetType.IsValueType || Nullable.GetUnderlyingType(targetType) != null)
                {
                    return null;
                }

                throw new InvalidCastException();
            }

            if (targetType.IsInstanceOfType(value))
            {
                return value;
            }

            if (targetType.IsEnum)
            {
                return Enum.Parse(targetType, Convert.ToString(value, CultureInfo.InvariantCulture));
            }

            return ConvertValue(value, targetType);
        }

        private object TryGetFirstPropertyValue(object target, params string[] propertyNames)
        {
            foreach (var propertyName in propertyNames)
            {
                var value = TryGetPropertyValue(target, propertyName);
                if (value != null)
                {
                    return value;
                }
            }

            return null;
        }

        private string BuildSyntheticIdentifier(string kind, string rawValue)
        {
            var encoded = Convert.ToBase64String(Encoding.UTF8.GetBytes(rawValue ?? string.Empty));
            return string.Format(
                CultureInfo.InvariantCulture,
                "{0}{1}:{2}",
                FallbackIdentifierPrefix,
                kind,
                encoded);
        }

        private object TryGetPropertyValueByFragments(object target, params string[] fragments)
        {
            if (target == null)
            {
                return null;
            }

            foreach (var property in target.GetType().GetProperties(BindingFlags.Public | BindingFlags.Instance))
            {
                if (fragments.All(
                    fragment => property.Name.IndexOf(fragment, StringComparison.OrdinalIgnoreCase) >= 0))
                {
                    return property.GetValue(target, null);
                }
            }

            return null;
        }

        private bool TypeNameContains(object target, string fragment)
        {
            return target != null &&
                target.GetType().Name.IndexOf(fragment, StringComparison.OrdinalIgnoreCase) >= 0;
        }

        private Type FindTypeByAnyName(params string[] typeNames)
        {
            var type = TryFindTypeByAnyName(typeNames);
            if (type == null)
            {
                throw new AdapterException(
                    "missing_type",
                    "Required TIA Portal Openness type was not found after loading assemblies.",
                    new Dictionary<string, object> { { "type_names", typeNames } });
            }

            return type;
        }

        private Type TryFindTypeByAnyName(params string[] typeNames)
        {
            foreach (var typeName in typeNames)
            {
                var fullType = TryFindType(typeName);
                if (fullType != null)
                {
                    return fullType;
                }

                var shortType = TryFindTypeByShortName(typeName);
                if (shortType != null)
                {
                    return shortType;
                }
            }

            return null;
        }

        private Type TryFindTypeByShortName(string typeName)
        {
            var shortName = typeName.Split('.').LastOrDefault();
            if (string.IsNullOrWhiteSpace(shortName))
            {
                return null;
            }

            foreach (var assembly in _loadedAssemblies.Values)
            {
                Type[] types;
                try
                {
                    types = assembly.GetTypes();
                }
                catch (ReflectionTypeLoadException ex)
                {
                    types = ex.Types;
                }

                foreach (var type in types)
                {
                    if (type != null &&
                        string.Equals(type.Name, shortName, StringComparison.Ordinal))
                    {
                        return type;
                    }
                }
            }

            return null;
        }

        private object GetServiceByAnyTypeName(object target, bool required, params string[] typeNames)
        {
            var serviceType = TryFindTypeByAnyName(typeNames);
            if (serviceType == null)
            {
                if (!required)
                {
                    return null;
                }

                throw new AdapterException(
                    "missing_type",
                    "Required TIA Portal Openness type was not found after loading assemblies.",
                    new Dictionary<string, object> { { "type_names", typeNames } });
            }

            var getServiceMethod = target.GetType().GetMethods()
                .FirstOrDefault(
                    method => method.Name == "GetService" &&
                              method.IsGenericMethodDefinition &&
                              method.GetParameters().Length == 0);
            if (getServiceMethod == null)
            {
                if (!required)
                {
                    return null;
                }

                throw new AdapterException(
                    "unsupported_service_lookup",
                    "GetService<T>() was not found on the target type.",
                    new Dictionary<string, object> { { "type", target.GetType().FullName } });
            }

            var generic = getServiceMethod.MakeGenericMethod(serviceType);
            var service = generic.Invoke(target, null);
            if (service == null && required)
            {
                throw new AdapterException(
                    "missing_service",
                    "Requested TIA service is not available on the target object.",
                    new Dictionary<string, object>
                    {
                        { "service_types", typeNames },
                        { "target_type", target.GetType().FullName },
                    });
            }

            return service;
        }

        private string GetStableIdentityKey(object target)
        {
            return GetObjectIdentifier(target) ??
                BuildPath(target) ??
                string.Format(CultureInfo.InvariantCulture, "{0}:{1}", target.GetType().FullName, ReadName(target));
        }

        private void ApplyScalarEdit(
            object target,
            string fieldName,
            object value,
            List<Dictionary<string, object>> changes,
            List<Dictionary<string, object>> verifiedFields)
        {
            var before = ReadFieldValue(target, fieldName);
            SetFieldValue(target, fieldName, value);
            var after = ReadFieldValue(target, fieldName);

            changes.Add(
                new Dictionary<string, object>
                {
                    { "field", ToSnakeCase(fieldName) },
                    { "before", before },
                    { "after", after },
                });
            verifiedFields.Add(
                new Dictionary<string, object>
                {
                    { "field", ToSnakeCase(fieldName) },
                    { "expected", NormalizeFieldValue(value) },
                    { "actual", after },
                });
        }

        private void MaybeApplyScalarEdit(
            object target,
            string fieldName,
            object value,
            List<Dictionary<string, object>> changes,
            List<Dictionary<string, object>> verifiedFields)
        {
            if (value == null)
            {
                return;
            }

            ApplyScalarEdit(target, fieldName, value, changes, verifiedFields);
        }

        private void ApplyScalarEditWithCandidates(
            object target,
            object value,
            List<Dictionary<string, object>> changes,
            List<Dictionary<string, object>> verifiedFields,
            params string[] candidateFields)
        {
            Exception lastException = null;
            foreach (var candidateField in candidateFields)
            {
                try
                {
                    ApplyScalarEdit(target, candidateField, value, changes, verifiedFields);
                    return;
                }
                catch (TargetInvocationException ex)
                {
                    lastException = ex.InnerException ?? ex;
                }
                catch (Exception ex)
                {
                    lastException = ex;
                }
            }

            if (lastException != null)
            {
                throw lastException;
            }

            throw new AdapterException(
                "unsupported_edit",
                "Requested field cannot be written on the selected object using the supported live field candidates.",
                new Dictionary<string, object>
                {
                    { "type", target.GetType().FullName },
                    { "fields", candidateFields },
                });
        }

        private object ReadFieldValue(object target, string fieldName)
        {
            var value = TryGetPropertyValue(target, fieldName);
            if (value != null)
            {
                return NormalizeFieldValue(value);
            }

            var attributeValue = TryGetAttributeValue(target, fieldName);
            return NormalizeFieldValue(attributeValue);
        }

        private void SetFieldValue(object target, string fieldName, object value)
        {
            var property = target.GetType().GetProperty(fieldName, BindingFlags.Public | BindingFlags.Instance);
            if (property != null && property.CanWrite)
            {
                property.SetValue(target, ConvertValue(value, property.PropertyType), null);
                return;
            }

            var setAttribute = target.GetType().GetMethods()
                .FirstOrDefault(
                    method => method.Name == "SetAttribute" &&
                              method.GetParameters().Length == 2);
            if (setAttribute != null)
            {
                setAttribute.Invoke(target, new[] { fieldName, value });
                return;
            }

            throw new AdapterException(
                "unsupported_edit",
                "Requested field cannot be written on the selected object.",
                new Dictionary<string, object>
                {
                    { "field", fieldName },
                    { "type", target.GetType().FullName },
                });
        }

        private object NormalizeFieldValue(object value)
        {
            if (value == null)
            {
                return null;
            }

            if (value is Version)
            {
                return value.ToString();
            }

            if (value is Enum)
            {
                return value.ToString();
            }

            return value;
        }

        private object ConvertValue(object value, Type targetType)
        {
            if (value == null)
            {
                return null;
            }

            if (targetType == typeof(string))
            {
                return Convert.ToString(value, CultureInfo.InvariantCulture);
            }

            if (targetType == typeof(bool) || targetType == typeof(bool?))
            {
                return Convert.ToBoolean(value, CultureInfo.InvariantCulture);
            }

            if (targetType == typeof(Version))
            {
                return new Version(Convert.ToString(value, CultureInfo.InvariantCulture));
            }

            return Convert.ChangeType(value, targetType, CultureInfo.InvariantCulture);
        }

        private void SetMultilingualTextField(
            object target,
            string fieldName,
            string value,
            List<Dictionary<string, object>> changes,
            List<Dictionary<string, object>> verifiedFields)
        {
            var multilingualText = TryGetPropertyValue(target, fieldName);
            if (multilingualText == null)
            {
                throw new AdapterException(
                    "unsupported_edit",
                    "Requested multilingual field was not found on the selected object.",
                    new Dictionary<string, object>
                    {
                        { "field", fieldName },
                        { "type", target.GetType().FullName },
                    });
            }

            var before = ReadMultilingualTextValue(multilingualText);
            var items = TryGetPropertyValue(multilingualText, "Items");
            var item = EnumerateObjects(items).FirstOrDefault();
            if (item == null)
            {
                throw new AdapterException(
                    "unsupported_edit",
                    "Requested multilingual field does not expose a writable text item.",
                    new Dictionary<string, object>
                    {
                        { "field", fieldName },
                        { "type", target.GetType().FullName },
                    });
            }

            SetFieldValue(item, "Text", value);
            var after = ReadMultilingualTextValue(multilingualText);
            changes.Add(
                new Dictionary<string, object>
                {
                    { "field", ToSnakeCase(fieldName) },
                    { "before", before },
                    { "after", after },
                });
            verifiedFields.Add(
                new Dictionary<string, object>
                {
                    { "field", ToSnakeCase(fieldName) },
                    { "expected", value },
                    { "actual", after },
                });
        }

        private string ReadMultilingualTextValue(object multilingualText)
        {
            if (multilingualText == null)
            {
                return null;
            }

            if (multilingualText is string)
            {
                return Convert.ToString(multilingualText, CultureInfo.InvariantCulture);
            }

            foreach (var item in EnumerateObjects(TryGetPropertyValue(multilingualText, "Items")))
            {
                var text = GetString(item, "Text");
                if (text != null)
                {
                    return text;
                }
            }

            return SafeToString(TryGetAttributeValue(multilingualText, "Text"));
        }

        private object ResolveUnifiedHmiSoftware(object target)
        {
            if (target != null &&
                string.Equals(target.GetType().FullName, "Siemens.Engineering.HmiUnified.HmiSoftware", StringComparison.Ordinal))
            {
                return target;
            }

            var hmiSoftware = TryGetHmiSoftware(target);
            if (hmiSoftware != null &&
                string.Equals(hmiSoftware.GetType().FullName, "Siemens.Engineering.HmiUnified.HmiSoftware", StringComparison.Ordinal))
            {
                return hmiSoftware;
            }

            throw new AdapterException(
                "unsupported_live_operation",
                "Live HMI alarm creation currently supports only WinCC Unified HMI software objects.",
                new Dictionary<string, object>
                {
                    { "target", target == null ? null : DescribeEngineeringObject(target) },
                });
        }

        private object EnsureUnifiedAlarmClass(
            object hmiSoftware,
            string severity,
            List<Dictionary<string, object>> touchedObjects)
        {
            var classes = GetPropertyValue(hmiSoftware, "AlarmClasses");
            var className = string.Format(
                CultureInfo.InvariantCulture,
                "AIPLC_{0}",
                severity.ToUpperInvariant());
            var existing = EnumerateObjects(classes).FirstOrDefault(
                candidate => string.Equals(ReadName(candidate), className, StringComparison.OrdinalIgnoreCase));
            if (existing != null)
            {
                return existing;
            }

            var created = CreateObjectInComposition(classes, className, new object[] { className });
            var changes = new List<Dictionary<string, object>>
            {
                new Dictionary<string, object>
                {
                    { "field", "created" },
                    { "before", null },
                    { "after", true },
                },
            };
            var verifiedFields = new List<Dictionary<string, object>>
            {
                new Dictionary<string, object>
                {
                    { "field", "name" },
                    { "expected", className },
                    { "actual", ReadName(created) },
                },
            };
            EnsureFieldValue(created, "Priority", MapSeverityToPriority(severity), changes, verifiedFields);
            if (!verifiedFields.All(field => Equals(field["expected"], field["actual"])))
            {
                throw new AdapterException(
                    "verification_failed",
                    "The live WinCC Unified alarm-class creation call completed but read-back verification did not match the requested values.",
                    DescribeEngineeringObject(created));
            }

            touchedObjects.Add(
                new Dictionary<string, object>
                {
                    { "object", DescribeEngineeringObject(created) },
                    { "changes", changes },
                });
            return created;
        }

        private byte MapSeverityToPriority(string severity)
        {
            switch ((severity ?? string.Empty).Trim().ToLowerInvariant())
            {
                case "info":
                    return 10;
                case "warning":
                    return 50;
                case "error":
                    return 100;
                case "critical":
                    return 200;
                default:
                    throw new AdapterException(
                        "unsupported_live_operation",
                        "Unsupported HMI alarm severity.",
                        new Dictionary<string, object> { { "severity", severity } });
            }
        }

        private void ParseTechnologyType(string rawTechnologyType, out string technologyType, out Version technologyVersion)
        {
            technologyType = rawTechnologyType;
            technologyVersion = new Version(1, 0);

            if (string.IsNullOrWhiteSpace(rawTechnologyType))
            {
                return;
            }

            var separatorIndex = rawTechnologyType.LastIndexOf('@');
            if (separatorIndex <= 0 || separatorIndex >= rawTechnologyType.Length - 1)
            {
                return;
            }

            Version parsedVersion;
            if (Version.TryParse(rawTechnologyType.Substring(separatorIndex + 1), out parsedVersion))
            {
                technologyType = rawTechnologyType.Substring(0, separatorIndex);
                technologyVersion = parsedVersion;
            }
        }

        private object ResolveOwningPlcSoftware(object target)
        {
            var current = target;
            while (current != null)
            {
                if (IsPlcSoftware(current))
                {
                    return current;
                }

                var fromDeviceItem = TryGetPlcSoftware(current);
                if (fromDeviceItem != null)
                {
                    return fromDeviceItem;
                }

                current = TryGetPropertyValue(current, "Parent");
            }

            return null;
        }

        private object ResolveTargetWithService(object target, params string[] serviceTypeNames)
        {
            foreach (var candidate in EnumerateServiceCandidates(target))
            {
                var service = GetServiceByAnyTypeName(candidate, false, serviceTypeNames);
                if (service != null)
                {
                    return candidate;
                }
            }

            throw new AdapterException(
                "missing_service",
                "Requested TIA service is not available on the selected scope or its nested hardware items.",
                new Dictionary<string, object>
                {
                    { "service_types", serviceTypeNames },
                    { "target", target == null ? null : DescribeEngineeringObject(target) },
                });
        }

        private IEnumerable<object> EnumerateServiceCandidates(object target)
        {
            var seen = new HashSet<string>(StringComparer.Ordinal);
            var current = target;
            while (current != null)
            {
                var key = GetStableIdentityKey(current);
                if (seen.Add(key))
                {
                    yield return current;
                }

                current = TryGetPropertyValue(current, "Parent");
            }

            var deviceItems = TryGetPropertyValue(target, "DeviceItems");
            if (deviceItems != null)
            {
                foreach (var child in FlattenDeviceItems(target))
                {
                    var key = GetStableIdentityKey(child);
                    if (seen.Add(key))
                    {
                        yield return child;
                    }
                }
            }
        }

        private void CollectCompareDifferences(object compareTarget, List<Dictionary<string, object>> differences)
        {
            object compareResult;
            try
            {
                compareResult = InvokeMethod(compareTarget, "CompareToOnline");
            }
            catch (TargetInvocationException ex)
            {
                throw ex.InnerException ?? ex;
            }

            CollectCompareDifferenceElements(TryGetPropertyValue(compareResult, "RootElement"), string.Empty, differences);
        }

        private void CollectCompareDifferenceElements(
            object element,
            string parentPath,
            List<Dictionary<string, object>> differences)
        {
            if (element == null)
            {
                return;
            }

            var leftName = SafeToString(TryGetPropertyValue(element, "LeftName"));
            var rightName = SafeToString(TryGetPropertyValue(element, "RightName"));
            var state = SafeToString(TryGetPropertyValue(element, "ComparisonResult"));
            var segment = !string.IsNullOrWhiteSpace(leftName) ? leftName : rightName;
            if (string.IsNullOrWhiteSpace(segment))
            {
                segment = "compare";
            }

            var currentPath = string.IsNullOrWhiteSpace(parentPath)
                ? segment
                : string.Format(CultureInfo.InvariantCulture, "{0}/{1}", parentPath, segment);
            if (IsDifferenceState(state))
            {
                differences.Add(
                    new Dictionary<string, object>
                    {
                        { "path", currentPath },
                        { "difference_type", state ?? "ObjectsDifferent" },
                        {
                            "description",
                            SafeToString(TryGetPropertyValue(element, "DetailedInformation")) ??
                                string.Format(CultureInfo.InvariantCulture, "{0} != {1}", leftName ?? "<missing>", rightName ?? "<missing>")
                        },
                    });
            }

            foreach (var child in EnumerateObjects(TryGetPropertyValue(element, "Elements")))
            {
                CollectCompareDifferenceElements(child, currentPath, differences);
            }
        }

        private bool IsDifferenceState(string state)
        {
            if (string.IsNullOrWhiteSpace(state))
            {
                return false;
            }

            return !string.Equals(state, "ObjectsIdentical", StringComparison.OrdinalIgnoreCase) &&
                !string.Equals(state, "FolderContentsIdentical", StringComparison.OrdinalIgnoreCase) &&
                !string.Equals(state, "CompareIrrelevant", StringComparison.OrdinalIgnoreCase);
        }

        private object SelectFirstConfigurationMode(object configuration)
        {
            var modes = TryGetPropertyValue(configuration, "Modes");
            var mode = EnumerateObjects(modes).FirstOrDefault();
            if (mode == null)
            {
                throw new AdapterException(
                    "missing_service",
                    "The selected download/online provider does not expose a usable connection configuration mode.",
                    new Dictionary<string, object>
                    {
                        { "configuration_type", configuration == null ? null : configuration.GetType().FullName },
                    });
            }

            return mode;
        }

        private object ResolveDownloadOptions(string downloadMode)
        {
            var downloadOptionsType = FindTypeByAnyName(
                "Siemens.Engineering.Download.DownloadOptions",
                "DownloadOptions");
            if (string.Equals(downloadMode, "software_only", StringComparison.OrdinalIgnoreCase))
            {
                return Enum.Parse(downloadOptionsType, "Software");
            }

            var hardware = Convert.ToInt32(Enum.Parse(downloadOptionsType, "Hardware"), CultureInfo.InvariantCulture);
            var software = Convert.ToInt32(Enum.Parse(downloadOptionsType, "Software"), CultureInfo.InvariantCulture);
            return Enum.ToObject(downloadOptionsType, hardware | software);
        }

        private Dictionary<string, object> DescribeDownloadResult(object downloadResult)
        {
            return new Dictionary<string, object>
            {
                { "state", SafeToString(TryGetPropertyValue(downloadResult, "State")) },
                { "warning_count", GetNullableInt(downloadResult, "WarningCount") ?? 0 },
                { "error_count", GetNullableInt(downloadResult, "ErrorCount") ?? 0 },
                {
                    "messages",
                    EnumerateObjects(TryGetPropertyValue(downloadResult, "Messages"))
                        .Select(DescribeDownloadResultMessage)
                        .ToList()
                },
            };
        }

        private Dictionary<string, object> DescribeDownloadResultMessage(object message)
        {
            return new Dictionary<string, object>
            {
                { "path", SafeToString(TryGetPropertyValue(message, "Path")) },
                { "state", SafeToString(TryGetPropertyValue(message, "State")) },
                {
                    "description",
                    SafeToString(TryGetPropertyValue(message, "Description")) ??
                        SafeToString(TryGetPropertyValue(message, "Message"))
                },
                { "warning_count", GetNullableInt(message, "WarningCount") ?? 0 },
                { "error_count", GetNullableInt(message, "ErrorCount") ?? 0 },
                {
                    "messages",
                    EnumerateObjects(TryGetPropertyValue(message, "Messages"))
                        .Select(DescribeDownloadResultMessage)
                        .ToList()
                },
            };
        }

        private string BuildDefaultExportPath(object target)
        {
            var safeName = string.Join(
                "_",
                ReadName(target).Split(Path.GetInvalidFileNameChars(), StringSplitOptions.RemoveEmptyEntries));
            return Path.Combine(Path.GetTempPath(), "codex-tia-exports", safeName + ".xml");
        }

        private string BuildPath(object target)
        {
            if (target == null)
            {
                return null;
            }

            var segments = new List<string>();
            var current = target;
            while (current != null)
            {
                var name = ReadName(current);
                if (!string.IsNullOrWhiteSpace(name))
                {
                    segments.Add(name);
                }

                current = TryGetPropertyValue(current, "Parent");
            }

            segments.Reverse();
            return string.Join("/", segments);
        }

        private string BuildFallbackIdentifier(object target)
        {
            var path = BuildPath(target);
            if (string.IsNullOrWhiteSpace(path))
            {
                return null;
            }

            var encodedPath = Convert.ToBase64String(Encoding.UTF8.GetBytes(path));
            return string.Format(
                CultureInfo.InvariantCulture,
                "{0}{1}:{2}",
                FallbackIdentifierPrefix,
                target.GetType().Name,
                encodedPath);
        }

        private string ReadName(object target)
        {
            return GetString(target, "Name") ??
                GetString(target, "ServerName") ??
                target.GetType().Name;
        }

        private string ReadNameOrNull(object target)
        {
            if (target == null)
            {
                return null;
            }

            if (target is string)
            {
                return Convert.ToString(target, CultureInfo.InvariantCulture);
            }

            return target.GetType().IsValueType ? SafeToString(target) : ReadName(target);
        }

        private static bool IsFallbackIdentifier(string objectId)
        {
            return !string.IsNullOrWhiteSpace(objectId) &&
                objectId.StartsWith(FallbackIdentifierPrefix, StringComparison.Ordinal);
        }

        private object ResolveFallbackObject(string objectId)
        {
            string kind;
            string path;
            if (!TryParseFallbackIdentifier(objectId, out kind, out path))
            {
                return null;
            }

            return EnumerateResolvableObjects().FirstOrDefault(
                candidate => string.Equals(candidate.GetType().Name, kind, StringComparison.Ordinal) &&
                    string.Equals(BuildPath(candidate), path, StringComparison.Ordinal));
        }

        private bool TryParseFallbackIdentifier(string objectId, out string kind, out string path)
        {
            kind = null;
            path = null;
            if (!IsFallbackIdentifier(objectId))
            {
                return false;
            }

            var remainder = objectId.Substring(FallbackIdentifierPrefix.Length);
            var separatorIndex = remainder.IndexOf(':');
            if (separatorIndex <= 0 || separatorIndex == remainder.Length - 1)
            {
                return false;
            }

            kind = remainder.Substring(0, separatorIndex);
            var encodedPath = remainder.Substring(separatorIndex + 1);
            try
            {
                path = Encoding.UTF8.GetString(Convert.FromBase64String(encodedPath));
                return !string.IsNullOrWhiteSpace(path);
            }
            catch
            {
                kind = null;
                path = null;
                return false;
            }
        }

        private IEnumerable<object> EnumerateResolvableObjects()
        {
            yield return _project;

            foreach (var network in EnumerateNetworkObjects())
            {
                yield return network;
            }

            foreach (var device in EnumerateObjects(GetPropertyValue(_project, "Devices")))
            {
                yield return device;

                foreach (var deviceItem in FlattenDeviceItems(device))
                {
                    yield return deviceItem;

                    var hmiSoftware = TryGetHmiSoftware(deviceItem);
                    if (hmiSoftware != null)
                    {
                        yield return hmiSoftware;
                    }

                    var plcSoftware = TryGetPlcSoftware(deviceItem);
                    if (plcSoftware == null)
                    {
                        continue;
                    }

                    foreach (var plcObject in EnumeratePlcSoftwareObjects(plcSoftware))
                    {
                        yield return plcObject;
                    }
                }
            }
        }

        private IEnumerable<object> EnumeratePlcSoftwareObjects(object plcSoftware)
        {
            yield return plcSoftware;

            foreach (var blockObject in EnumerateBlockGroupObjects(TryGetPropertyValue(plcSoftware, "BlockGroup")))
            {
                yield return blockObject;
            }

            foreach (var tagObject in EnumerateTagTableGroupObjects(TryGetPropertyValue(plcSoftware, "TagTableGroup")))
            {
                yield return tagObject;
            }

            foreach (var dataType in EnumerateDataTypeObjects(FindDataTypeRoot(plcSoftware)))
            {
                yield return dataType;
            }

            foreach (var technologyObject in EnumerateTechnologyObjectItems(FindTechnologyObjectRoot(plcSoftware)))
            {
                yield return technologyObject;
            }

            foreach (var watchTable in EnumerateWatchTableObjects(FindWatchTableRoot(plcSoftware)))
            {
                yield return watchTable;
            }

            foreach (var safetyObject in EnumerateSafetyObjects(plcSoftware))
            {
                yield return safetyObject;
            }
        }

        private IEnumerable<object> EnumerateDataTypeObjects(object group)
        {
            if (group == null)
            {
                yield break;
            }

            foreach (var dataType in EnumerateObjects(
                TryGetFirstPropertyValue(
                    group,
                    "Types",
                    "DataTypes",
                    "PlcTypes",
                    "SystemTypes")))
            {
                yield return dataType;
            }

            foreach (var dataType in EnumerateObjects(group))
            {
                if (LooksLikeDataTypeObject(dataType))
                {
                    yield return dataType;
                }
            }

            foreach (var childGroup in EnumerateObjects(
                TryGetFirstPropertyValue(
                    group,
                    "Groups",
                    "TypeGroups",
                    "Subgroups",
                    "SystemTypeGroups")))
            {
                foreach (var child in EnumerateDataTypeObjects(childGroup))
                {
                    yield return child;
                }
            }
        }

        private IEnumerable<object> EnumerateTechnologyObjectItems(object group)
        {
            if (group == null)
            {
                yield break;
            }

            if (TypeNameContains(group, "Technology") &&
                !TypeNameContains(group, "Group") &&
                !TypeNameContains(group, "Composition"))
            {
                yield return group;
            }

            foreach (var item in EnumerateObjects(
                TryGetFirstPropertyValue(
                    group,
                    "TechnologicalObjects",
                    "TechnologyObjects",
                    "Objects")))
            {
                yield return item;
            }

            foreach (var item in EnumerateObjects(group))
            {
                if (TypeNameContains(item, "Technology") &&
                    !TypeNameContains(item, "Group") &&
                    !TypeNameContains(item, "Composition"))
                {
                    yield return item;
                }
            }

            foreach (var childGroup in EnumerateObjects(
                TryGetFirstPropertyValue(
                    group,
                    "Groups",
                    "Subgroups",
                    "TechnologicalObjectGroups",
                    "TechnologyObjectGroups")))
            {
                foreach (var child in EnumerateTechnologyObjectItems(childGroup))
                {
                    yield return child;
                }
            }
        }

        private IEnumerable<object> EnumerateWatchTableObjects(object group)
        {
            if (group == null)
            {
                yield break;
            }

            if (TypeNameContains(group, "Watch") &&
                !TypeNameContains(group, "Group") &&
                !TypeNameContains(group, "Composition"))
            {
                yield return group;
            }

            foreach (var table in EnumerateObjects(
                TryGetFirstPropertyValue(
                    group,
                    "WatchTables",
                    "Tables")))
            {
                yield return table;
            }

            foreach (var table in EnumerateObjects(group))
            {
                if (TypeNameContains(table, "Watch") &&
                    !TypeNameContains(table, "Group") &&
                    !TypeNameContains(table, "Composition") &&
                    !TypeNameContains(table, "Expression"))
                {
                    yield return table;
                }
            }

            foreach (var childGroup in EnumerateObjects(
                TryGetFirstPropertyValue(
                    group,
                    "Groups",
                    "Subgroups",
                    "Folders")))
            {
                foreach (var child in EnumerateWatchTableObjects(childGroup))
                {
                    yield return child;
                }
            }
        }

        private IEnumerable<object> EnumerateBlockGroupObjects(object group)
        {
            if (group == null)
            {
                yield break;
            }

            yield return group;

            foreach (var block in EnumerateObjects(TryGetPropertyValue(group, "Blocks")))
            {
                yield return block;
            }

            foreach (var childGroup in EnumerateObjects(TryGetPropertyValue(group, "Groups")))
            {
                foreach (var child in EnumerateBlockGroupObjects(childGroup))
                {
                    yield return child;
                }
            }
        }

        private IEnumerable<object> EnumerateTagTableGroupObjects(object group)
        {
            if (group == null)
            {
                yield break;
            }

            yield return group;

            foreach (var table in EnumerateObjects(TryGetPropertyValue(group, "TagTables")))
            {
                yield return table;

                foreach (var tag in EnumerateObjects(TryGetPropertyValue(table, "Tags")))
                {
                    yield return tag;
                }
            }

            foreach (var childGroup in EnumerateObjects(TryGetPropertyValue(group, "Groups")))
            {
                foreach (var child in EnumerateTagTableGroupObjects(childGroup))
                {
                    yield return child;
                }
            }
        }

        private void EnsureConnected()
        {
            if (_tiaPortal == null)
            {
                throw new AdapterException(
                    "not_connected",
                    "Connect to TIA Portal before issuing project operations.",
                    null);
            }
        }

        private void EnsureProjectOpen()
        {
            EnsureConnected();
            if (_project == null)
            {
                throw new AdapterException(
                    "project_not_open",
                    "Open a TIA project before using this operation.",
                    null);
            }
        }

        private Assembly ResolveAssembly(object sender, ResolveEventArgs args)
        {
            if (string.IsNullOrWhiteSpace(_assemblyDirectory))
            {
                return null;
            }

            var fileName = new AssemblyName(args.Name).Name + ".dll";
            var candidate = Path.Combine(_assemblyDirectory, fileName);
            if (!File.Exists(candidate))
            {
                return null;
            }

            Assembly existing;
            if (_loadedAssemblies.TryGetValue(candidate, out existing))
            {
                return existing;
            }

            var loaded = Assembly.LoadFrom(candidate);
            _loadedAssemblies[candidate] = loaded;
            return loaded;
        }

        private void LoadAssemblyIfExists(string fileName, bool required)
        {
            var candidate = Path.Combine(_assemblyDirectory, fileName);
            if (!File.Exists(candidate))
            {
                if (required)
                {
                    throw new AdapterException(
                        "missing_assembly",
                        "Required Siemens.Engineering assembly was not found.",
                        new Dictionary<string, object> { { "path", candidate } });
                }

                return;
            }

            if (_loadedAssemblies.ContainsKey(candidate))
            {
                return;
            }

            _loadedAssemblies[candidate] = Assembly.LoadFrom(candidate);
        }

        private Type FindType(string fullName)
        {
            var type = TryFindType(fullName);
            if (type == null)
            {
                throw new AdapterException(
                    "missing_type",
                    "Required TIA Portal Openness type was not found after loading assemblies.",
                    new Dictionary<string, object> { { "type_name", fullName } });
            }

            return type;
        }

        private Type TryFindType(string fullName)
        {
            foreach (var assembly in _loadedAssemblies.Values)
            {
                var type = assembly.GetType(fullName, throwOnError: false, ignoreCase: false);
                if (type != null)
                {
                    return type;
                }
            }

            return null;
        }

        private static object InvokeStatic(Type targetType, string methodName)
        {
            var method = targetType.GetMethod(methodName, BindingFlags.Public | BindingFlags.Static);
            if (method == null)
            {
                throw new AdapterException(
                    "missing_method",
                    "Required static method was not found.",
                    new Dictionary<string, object>
                    {
                        { "type", targetType.FullName },
                        { "method", methodName },
                    });
            }

            return method.Invoke(null, null);
        }

        private static object InvokeMethod(object target, string methodName, params object[] args)
        {
            var method = target.GetType().GetMethods()
                .FirstOrDefault(
                    item => item.Name == methodName &&
                            item.GetParameters().Length == args.Length);
            if (method == null)
            {
                throw new AdapterException(
                    "missing_method",
                    "Required instance method was not found.",
                    new Dictionary<string, object>
                    {
                        { "type", target.GetType().FullName },
                        { "method", methodName },
                    });
            }

            return method.Invoke(target, args);
        }

        private static IEnumerable<object> EnumerateObjects(object sequence)
        {
            var enumerable = sequence as IEnumerable;
            if (enumerable == null)
            {
                yield break;
            }

            foreach (var item in enumerable)
            {
                if (item != null)
                {
                    yield return item;
                }
            }
        }

        private static object TryGetPropertyValue(object target, string propertyName)
        {
            if (target == null)
            {
                return null;
            }

            var property = target.GetType().GetProperty(propertyName, BindingFlags.Public | BindingFlags.Instance);
            return property != null ? property.GetValue(target, null) : null;
        }

        private static object TryGetAttributeValue(object target, string attributeName)
        {
            var method = target.GetType().GetMethods()
                .FirstOrDefault(
                    item => item.Name == "GetAttribute" &&
                            item.GetParameters().Length >= 1 &&
                            item.GetParameters()[0].ParameterType == typeof(string));
            if (method == null)
            {
                return null;
            }

            return method.GetParameters().Length == 2
                ? method.Invoke(target, new object[] { attributeName, false })
                : method.Invoke(target, new object[] { attributeName });
        }

        private static object TryGetOptionalAttributeValue(object target, string attributeName)
        {
            try
            {
                return TryGetAttributeValue(target, attributeName);
            }
            catch (TargetInvocationException ex) when (IsUnsupportedOptionalFieldAccess(ex.InnerException ?? ex))
            {
                return null;
            }
            catch (Exception ex) when (IsUnsupportedOptionalFieldAccess(ex))
            {
                return null;
            }
        }

        private static object GetPropertyValue(object target, string propertyName)
        {
            var value = TryGetPropertyValue(target, propertyName);
            if (value == null)
            {
                throw new AdapterException(
                    "missing_property",
                    "Required property was not found.",
                    new Dictionary<string, object>
                    {
                        { "type", target.GetType().FullName },
                        { "property", propertyName },
                    });
            }

            return value;
        }

        private static bool IsUnsupportedOptionalFieldAccess(Exception ex)
        {
            if (ex == null)
            {
                return false;
            }

            var fullName = ex.GetType().FullName ?? ex.GetType().Name;
            return ex is NotSupportedException ||
                string.Equals(fullName, "Siemens.Engineering.EngineeringNotSupportedException", StringComparison.Ordinal) ||
                fullName.EndsWith(".EngineeringNotSupportedException", false, CultureInfo.InvariantCulture);
        }

        private static string SafeToString(object value)
        {
            return value == null ? null : Convert.ToString(value, CultureInfo.InvariantCulture);
        }

        private static string GetString(object source, string propertyName)
        {
            var dictionary = source as Dictionary<string, object>;
            if (dictionary != null)
            {
                object value;
                if (!dictionary.TryGetValue(propertyName, out value) || value == null)
                {
                    return null;
                }

                return Convert.ToString(value, CultureInfo.InvariantCulture);
            }

            return SafeToString(TryGetPropertyValue(source, propertyName));
        }

        private static string TryGetOptionalString(object source, string propertyName)
        {
            try
            {
                return GetString(source, propertyName);
            }
            catch (TargetInvocationException ex) when (IsUnsupportedOptionalFieldAccess(ex.InnerException ?? ex))
            {
                return null;
            }
            catch (Exception ex) when (IsUnsupportedOptionalFieldAccess(ex))
            {
                return null;
            }
        }

        private static string GetRequiredString(Dictionary<string, object> source, string key)
        {
            var value = GetString(source, key);
            if (string.IsNullOrWhiteSpace(value))
            {
                throw new AdapterException(
                    "missing_parameter",
                    "Required request parameter is missing.",
                    new Dictionary<string, object> { { "parameter", key } });
            }

            return value;
        }

        private static int? GetNullableInt(object source, string propertyName)
        {
            object value;
            var dictionary = source as Dictionary<string, object>;
            if (dictionary != null)
            {
                if (!dictionary.TryGetValue(propertyName, out value) || value == null)
                {
                    return null;
                }
            }
            else
            {
                value = TryGetPropertyValue(source, propertyName);
            }

            return value == null ? (int?)null : Convert.ToInt32(value, CultureInfo.InvariantCulture);
        }

        private static bool? GetNullableBoolean(object source, string propertyName)
        {
            object value;
            var dictionary = source as Dictionary<string, object>;
            if (dictionary != null)
            {
                if (!dictionary.TryGetValue(propertyName, out value) || value == null)
                {
                    return null;
                }
            }
            else
            {
                value = TryGetPropertyValue(source, propertyName);
            }

            return value == null ? (bool?)null : Convert.ToBoolean(value, CultureInfo.InvariantCulture);
        }

        private static int? GetNullableIntAttribute(object target, string attributeName)
        {
            var value = TryGetAttributeValue(target, attributeName);
            return value == null ? (int?)null : Convert.ToInt32(value, CultureInfo.InvariantCulture);
        }

        private static Dictionary<string, object> GetDictionary(
            Dictionary<string, object> source,
            string key,
            bool required)
        {
            object value;
            if (source.TryGetValue(key, out value))
            {
                var dictionary = value as Dictionary<string, object>;
                if (dictionary != null)
                {
                    return dictionary;
                }
            }

            if (!required)
            {
                return null;
            }

            throw new AdapterException(
                "missing_parameter",
                "Required object parameter is missing.",
                new Dictionary<string, object> { { "parameter", key } });
        }

        private static List<object> GetList(
            Dictionary<string, object> source,
            string key,
            bool required)
        {
            object value;
            if (source.TryGetValue(key, out value))
            {
                var array = value as ArrayList;
                if (array != null)
                {
                    return array.Cast<object>().ToList();
                }

                var enumerable = value as IEnumerable;
                if (enumerable != null && !(value is string))
                {
                    return enumerable.Cast<object>().ToList();
                }
            }

            if (!required)
            {
                return null;
            }

            throw new AdapterException(
                "missing_parameter",
                "Required array parameter is missing.",
                new Dictionary<string, object> { { "parameter", key } });
        }

        private static string ReadOption(string[] args, string optionName)
        {
            for (var index = 0; index < args.Length - 1; index++)
            {
                if (string.Equals(args[index], optionName, StringComparison.OrdinalIgnoreCase))
                {
                    return args[index + 1];
                }
            }

            return null;
        }

        private static string ComputeSha256(string path)
        {
            using (var stream = File.OpenRead(path))
            using (var sha = SHA256.Create())
            {
                return BitConverter.ToString(sha.ComputeHash(stream)).Replace("-", string.Empty).ToLowerInvariant();
            }
        }

        private static InstalledPortal CreateInstallFromDirectory(string publicApiDirectory, string requestedVersion)
        {
            var modular = File.Exists(Path.Combine(publicApiDirectory, "Siemens.Engineering.Base.dll"));
            var portalVersion = !string.IsNullOrWhiteSpace(requestedVersion)
                ? NormalizePortalVersion(requestedVersion)
                : GuessPortalVersion(publicApiDirectory);
            return new InstalledPortal
            {
                PortalVersion = portalVersion,
                PublicApiDirectory = publicApiDirectory,
                UsesModularAssemblies = modular,
                SortKey = ParseMajorVersion(portalVersion),
            };
        }

        private static List<InstalledPortal> DiscoverInstalls()
        {
            var installs = new Dictionary<string, InstalledPortal>(StringComparer.OrdinalIgnoreCase);
            foreach (var install in DiscoverInstallsFromRegistry())
            {
                installs[install.PortalVersion] = install;
            }

            foreach (var install in DiscoverInstallsFromFilesystem())
            {
                installs[install.PortalVersion] = install;
            }

            return installs.Values.ToList();
        }

        private static IEnumerable<InstalledPortal> DiscoverInstallsFromRegistry()
        {
            using (var baseKey = RegistryKey.OpenBaseKey(RegistryHive.LocalMachine, RegistryView.Registry64))
            using (var openness = baseKey.OpenSubKey(@"SOFTWARE\Siemens\Automation\Openness"))
            {
                if (openness == null)
                {
                    yield break;
                }

                foreach (var versionKeyName in openness.GetSubKeyNames())
                {
                    if (ParseMajorVersion(versionKeyName) == 0)
                    {
                        continue;
                    }

                    foreach (var path in DiscoverPublicApiDirectories(openness.OpenSubKey(versionKeyName)))
                    {
                        if (Directory.Exists(path))
                        {
                            yield return CreateInstallFromDirectory(path, "V" + ParseMajorVersion(versionKeyName));
                        }
                    }
                }
            }
        }

        private static IEnumerable<InstalledPortal> DiscoverInstallsFromFilesystem()
        {
            var portalRoot = Path.Combine(
                Environment.GetFolderPath(Environment.SpecialFolder.ProgramFiles),
                "Siemens",
                "Automation");
            if (!Directory.Exists(portalRoot))
            {
                yield break;
            }

            foreach (var directory in Directory.GetDirectories(portalRoot, "Portal V*"))
            {
                var name = Path.GetFileName(directory);
                var majorVersion = ParseMajorVersion(name);
                if (majorVersion == 0)
                {
                    continue;
                }

                var modularDirectory = Path.Combine(directory, "PublicAPI", "V" + majorVersion, "net48");
                if (Directory.Exists(modularDirectory))
                {
                    yield return CreateInstallFromDirectory(modularDirectory, "V" + majorVersion);
                    continue;
                }

                var legacyDirectory = Path.Combine(directory, "PublicAPI", "V" + majorVersion);
                if (Directory.Exists(legacyDirectory))
                {
                    yield return CreateInstallFromDirectory(legacyDirectory, "V" + majorVersion);
                }
            }
        }

        private static IEnumerable<string> DiscoverPublicApiDirectories(RegistryKey versionKey)
        {
            if (versionKey == null)
            {
                yield break;
            }

            using (var publicApiKey = versionKey.OpenSubKey("PublicAPI"))
            {
                if (publicApiKey == null)
                {
                    yield break;
                }

                foreach (var candidate in WalkRegistryValues(publicApiKey))
                {
                    if (Directory.Exists(candidate))
                    {
                        yield return candidate;
                    }
                    else if (File.Exists(candidate))
                    {
                        yield return Path.GetDirectoryName(candidate);
                    }
                }
            }
        }

        private static IEnumerable<string> WalkRegistryValues(RegistryKey key)
        {
            foreach (var valueName in key.GetValueNames())
            {
                var value = key.GetValue(valueName) as string;
                if (!string.IsNullOrWhiteSpace(value))
                {
                    yield return value;
                }
            }

            foreach (var subKeyName in key.GetSubKeyNames())
            {
                using (var subKey = key.OpenSubKey(subKeyName))
                {
                    if (subKey == null)
                    {
                        continue;
                    }

                    foreach (var value in WalkRegistryValues(subKey))
                    {
                        yield return value;
                    }
                }
            }
        }

        private static string GuessPortalVersion(string publicApiDirectory)
        {
            var majorVersion = ParseMajorVersion(publicApiDirectory);
            return majorVersion == 0 ? "V21" : "V" + majorVersion;
        }

        private static string NormalizePortalVersion(string raw)
        {
            if (string.IsNullOrWhiteSpace(raw))
            {
                return "V21";
            }

            var major = ParseMajorVersion(raw);
            return major == 0 ? raw : "V" + major;
        }

        private static int ParseMajorVersion(string raw)
        {
            if (string.IsNullOrWhiteSpace(raw))
            {
                return 0;
            }

            var digits = new string(raw.Where(char.IsDigit).ToArray());
            if (digits.Length == 0)
            {
                return 0;
            }

            int major;
            return int.TryParse(digits.Substring(0, Math.Min(2, digits.Length)), NumberStyles.Integer, CultureInfo.InvariantCulture, out major)
                ? major
                : 0;
        }

        private static string ToSnakeCase(string value)
        {
            if (string.IsNullOrWhiteSpace(value))
            {
                return value;
            }

            var buffer = new List<char>();
            for (var index = 0; index < value.Length; index++)
            {
                var current = value[index];
                if (char.IsUpper(current) && index > 0)
                {
                    buffer.Add('_');
                }

                buffer.Add(char.ToLowerInvariant(current));
            }

            return new string(buffer.ToArray());
        }

        private sealed class InstalledPortal
        {
            public string PortalVersion { get; set; }

            public string PublicApiDirectory { get; set; }

            public bool UsesModularAssemblies { get; set; }

            public int SortKey { get; set; }
        }

        private sealed class AdapterException : Exception
        {
            internal AdapterException(string code, string message, object details)
                : base(message)
            {
                Code = code;
                Details = details;
            }

            internal string Code { get; private set; }

            internal object Details { get; private set; }
        }
    }
}
