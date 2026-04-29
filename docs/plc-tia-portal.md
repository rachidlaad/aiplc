# Siemens TIA Portal PLC Agent

This repository now supports Siemens TIA Portal engineering through an adapter layer instead of an AIPLC core rewrite:

- AIPLC CLI/TUI and the normal session loop stay unchanged.
- `codex-plc-mcp-server` exposes PLC engineering capabilities as MCP tools.
- A Windows-local `.NET Framework 4.8` companion adapter talks to TIA Portal Openness and returns structured results.
- The boundary is adapter-based, so future industrial backends can replace the Windows adapter without changing the main AIPLC loop.

## Architecture

```text
AIPLC CLI/TUI
  -> MCP client
  -> codex-plc-mcp-server (Rust, stdio MCP server)
  -> backend adapter
     -> subprocess backend for Windows-local TIA Portal Openness
     -> internal simulator backend for tests/CI only
  -> Siemens TIA Portal
```

The Rust MCP server owns the tool contract and approval boundary. The Windows adapter owns TIA-specific assembly loading, object lookup, Openness calls, read-back verification, and compile reporting.

## Exposed tools

The product tool list is live-first. The default backend is `subprocess`, which
uses the Windows-local TIA Portal Openness adapter and exposes the full tool
contract directly through the live adapter.

Live `subprocess` tools:

- `tia_portal_connect`
- `tia_portal_open_project`
- `tia_portal_project_overview`
- `tia_portal_list_blocks`
- `tia_portal_list_tag_tables`
- `tia_portal_list_data_types`
- `tia_portal_export_object`
- `tia_portal_import_object`
- `tia_portal_rename_object`
- `tia_portal_set_block_header`
- `tia_portal_set_plc_tag_properties`
- `tia_portal_apply_edit`
- `tia_portal_create_udt`
- `tia_portal_edit_udt`
- `tia_portal_create_block`
- `tia_portal_edit_block_body`
- `tia_portal_create_block_call`
- `tia_portal_edit_db_members`
- `tia_portal_create_plc_tag`
- `tia_portal_create_tag_table`
- `tia_portal_list_technology_objects`
- `tia_portal_list_watch_tables`
- `tia_portal_list_networks`
- `tia_portal_list_hmi_objects`
- `tia_portal_list_safety_objects`
- `tia_portal_cross_reference`
- `tia_portal_consistency_check`
- `tia_portal_create_watch_table`
- `tia_portal_write_hardware_config`
- `tia_portal_write_network_config`
- `tia_portal_create_hmi_alarm`
- `tia_portal_create_technology_object`
- `tia_portal_create_safety_object`
- `tia_portal_compare_online_offline`
- `tia_portal_run_simulation`
- `tia_portal_go_online`
- `tia_portal_download_to_device`
- `tia_portal_compile`

## Capability matrix

Live TIA Openness adapter exposes:

- connect / attach / launch
- open project
- project overview
- list blocks
- list tag tables
- list PLC data types with best-effort member inventory
- list technology objects with best-effort Openness discovery
- list watch tables with best-effort expression inventory
- list networks with best-effort participant discovery
- list HMI software objects
- list safety-related objects exposed by the PLC software object model
- cross-reference inspection via the documented Openness `CrossReferenceService`
- consistency check through direct `ICompilable` execution and flattened issue reporting
- export supported objects
- import supported blocks and PLC tag tables
- UDT creation and editing with read-back verification
- block creation, block-body edits, and block-call insertion with read-back verification
- DB member editing with read-back verification
- create PLC tag tables
- create PLC tags with read-back verification of requested scalar properties
- watch-table creation
- hardware and network configuration writes
- HMI alarm authoring
- technology-object and safety-object authoring
- online/offline compare
- simulation
- go-online
- download-to-device
- direct safe edits for supported object renames, selected block header fields,
  and selected PLC tag properties
- compile project or object scope

Live actions are not masked by a second product mode:

- all MCP tools are exposed through the live `subprocess` backend
- any operation that the real adapter or current TIA project cannot complete
  fails with the adapter/runtime error from the live path
- the server does not maintain a separate product-only feature bucket

## Safety boundaries

- Inspect first. Always connect, open, and enumerate before mutating.
- Prefer direct Openness object-model edits over export/import when both are available.
- Treat read tools as low-risk and mutating tools as approval-gated.
- Never report a change as successful unless the API call succeeded and the post-change verification read also succeeded.
- Always report touched objects, field-level changes, and compile outcomes.
- Stop on ambiguity, unsupported object kinds, unresolved references, or failed verification.
- Use a disposable project copy for manual testing. Do not point this workflow at a production master project.

## Supported versions

Current intent:

- Primary target: TIA Portal Openness `V21`
- Legacy compatibility path: `V20`-style single-assembly installs where the legacy `Siemens.Engineering.dll` layout is still present

Why the adapter is version-aware:

- Siemens documents that V21 moved Openness to modular assemblies, changed installation and registry layout, and requires rebuilding against the V21 libraries.
- Siemens documents that V21 Openness supports only `.NET Framework 4.8`.

Official references:

- V21 modular assembly changes: <https://docs.tia.siemens.cloud/r/en-us/v21/readme-tia-portal-openness/major-changes-for-long-term-stability-in-tia-portal-openness-v21>
- Object support matrix: <https://docs.tia.siemens.cloud/r/en-us/v21/tia-portal-openness-api-for-automation-of-engineering-workflows/tia-portal-openness-api/tia-portal-openness-object/object-list?contentId=6KC67KZF3fLSGEA_9eMd1Q>

## Windows setup

1. Install TIA Portal with Openness on Windows.
2. Ensure your Windows user is in the local `Siemens TIA Openness` group.
3. Build the Windows adapter on that machine:

```powershell
msbuild adapters\siemens-tia-openness\Aiplc.TiaOpenness.Adapter\Aiplc.TiaOpenness.Adapter.csproj /p:Configuration=Release
```

4. Build the Rust MCP server:

```powershell
cd codex-rs
cargo build -p codex-plc-mcp-server
```

5. Configure AIPLC to launch the MCP server and gate mutations with prompts:

```toml
[mcp_servers.tia_portal]
command = "C:\\path\\to\\codex-plc-mcp-server.exe"
args = [
  "--adapter-command",
  "C:\\path\\to\\Aiplc.TiaOpenness.Adapter.exe",
]
default_tools_approval_mode = "prompt"

[mcp_servers.tia_portal.tools.tia_portal_connect]
approval_mode = "approve"
[mcp_servers.tia_portal.tools.tia_portal_open_project]
approval_mode = "approve"
[mcp_servers.tia_portal.tools.tia_portal_project_overview]
approval_mode = "approve"
[mcp_servers.tia_portal.tools.tia_portal_list_blocks]
approval_mode = "approve"
[mcp_servers.tia_portal.tools.tia_portal_list_tag_tables]
approval_mode = "approve"
[mcp_servers.tia_portal.tools.tia_portal_export_object]
approval_mode = "approve"
```

## DLL resolution

The Windows adapter does not statically reference Siemens assemblies. It resolves them at runtime so one adapter binary can target different local TIA installs.

Resolution order:

1. `--public-api-dir`
2. `CODEX_TIA_PUBLICAPI_DIR`
3. Siemens Openness registry discovery
4. Filesystem discovery under `C:\Program Files\Siemens\Automation\Portal V*\PublicAPI\...`

Important version notes:

- For V21, the expected shared folder is typically `C:\Program Files\Siemens\Automation\Portal V21\PublicAPI\V21\net48`.
- For older layouts, the adapter falls back to a legacy single-assembly folder such as `...\PublicAPI\V20`.
- Do not copy Siemens assemblies into the adapter output folder. Siemens documents that `Copy Local` is not supported for Openness assemblies in V21.

## Direct API vs export/import

The agent is designed to prefer direct Openness object-model operations first and only fall back to export/import when needed.

That matters because Siemens documents two important constraints for export/import:

- the export/import file format is XML for most supported objects
- the export format is internal and valid only for the current TIA Portal Openness version

Official reference:

- <https://docs.tia.siemens.cloud/r/en-us/v21/tia-portal-openness-api-for-automation-of-engineering-workflows/export/import/overview/basic-principles-of-importing/exporting>

In practice:

- use `tia_portal_apply_edit` for safe, deterministic field edits when the object model exposes them
- prefer `tia_portal_rename_object`, `tia_portal_set_block_header`, and `tia_portal_set_plc_tag_properties` over `tia_portal_apply_edit` for those common edits
- if `tia_portal_apply_edit` is used directly, its `operation` field must be a structured object, never free text
- use `tia_portal_export_object` and `tia_portal_import_object` for controlled round-trips when direct writes are unavailable
- never build a version-agnostic XML editing system on top of TIA export files
- advanced authoring tools should follow the same rule when live implementations are added later: direct Openness first, export/import only as a fallback

## Compile behavior

The compile tool resolves `ICompilable` from the selected target and returns the result tree with message counts and nested messages.

Current compile scopes:

- current project
- single engineering object

Official reference:

- <https://docs.tia.siemens.cloud/r/en-us/v21/tia-portal-openness-api-for-automation-of-engineering-workflows/tia-portal-openness-api/functions-for-projects-and-project-data/compiling-a-project?contentId=Lh19wmmzQOEBZSyi3sS~RQ>

Operational note from Siemens:

- all devices should be offline before compilation

## Internal test simulator

The product backend is live `subprocess`. The simulator backend is internal
test infrastructure for CI and local Rust tests without TIA installed. It is not
a product mode, and release builds reject it.

The simulator provides:

- deterministic project/device/PLC/block/tag-table inspection
- deterministic UDT, DB, and block-authoring state
- deterministic technology-object, watch-table, network, HMI, and safety inventories
- deterministic hardware and network write workflows with read-back verification
- deterministic HMI alarm, technology-object, and safety-object authoring
- deterministic cross-reference, consistency-check, compare, and simulation results
- deterministic go-online and download-to-device state transitions
- export/import round-trips against JSON fixtures
- safe edit verification
- compile success and compile failure paths

## Automated validation

Rust-side validation added in this change set:

- tool schema tests in `plc-mcp-server/src/tooling.rs`
- simulator backend unit tests in the PLC MCP server backend
- subprocess adapter protocol tests in `plc-mcp-server/src/backend/subprocess.rs`
- simulator-backed MCP end-to-end flows in the PLC MCP server tests
  includes a full authoring flow covering UDTs, blocks, block calls, DB edits, tags, watch tables, diagnostics, validation, simulation, and compile

Run them with:

```powershell
cd codex-rs
cargo test -p codex-plc-mcp-server
```

## Manual Windows acceptance test

An ignored Windows-only integration test exercises the full path against a real TIA install while restoring the edited block afterward.

Environment:

```powershell
$env:AIPLC_TIA_ADAPTER_COMMAND = "C:\path\to\Aiplc.TiaOpenness.Adapter.exe"
$env:CODEX_TIA_PROJECT_PATH = "C:\Samples\ProjectCopy\ProjectCopy.ap21"
$env:CODEX_TIA_BLOCK_NAME = "MotorFB"

# Optional when the project has multiple PLC software roots.
$env:CODEX_TIA_PLC_SOFTWARE_NAME = "PLC_1"

# Optional when auto-discovery is not enough.
$env:CODEX_TIA_PORTAL_VERSION = "V21"
$env:CODEX_TIA_PUBLICAPI_DIR = "C:\Program Files\Siemens\Automation\Portal V21\PublicAPI\V21\net48"
```

Run:

```powershell
cd codex-rs
cargo test -p codex-plc-mcp-server --test manual_windows_acceptance -- --ignored --nocapture
```

What it proves:

1. connect to a local TIA Portal instance or launch one
2. open the sample project
3. inspect PLC software and blocks
4. export the target block
5. apply a controlled block-header change
6. compile the changed block
7. export again and verify the marker came back from TIA
8. restore the original block by importing the backup export
9. compile again
10. export again and verify the marker is gone

Current scope of the manual live acceptance path:

- it validates the live Openness operations that are implemented today
- advanced authoring and commissioning tools should get their own Windows acceptance scenarios as live Openness implementations are added

## Example transcript

Prompt:

```text
Connect to local TIA Portal V21, open C:\Samples\Packaging\Packaging.ap21, inspect the PLC, and tell me which block you would change to update the conveyor start logic.
```

Expected agent behavior:

```text
1. tia_portal_connect(connection_mode=auto, ui_mode=with_ui, portal_version=V21)
2. tia_portal_open_project(project_path=C:\Samples\Packaging\Packaging.ap21)
3. tia_portal_project_overview()
4. tia_portal_list_blocks(plc_software_id=...)
5. tia_portal_export_object(object_id=block/ConveyorStart, read_mode=include_text)
```

Follow-up prompt:

```text
Update only the block header author to "Controls Engineering", compile just that block, and confirm the result.
```

Expected agent response pattern:

```text
Target object: ConveyorStart (CodeBlock)
Planned change: header_author -> Controls Engineering
Verification: read back exported block text contains the updated header author
Compile scope: that block only
Compile result: success, 0 errors, 0 warnings
Touched objects:
- ConveyorStart: header_author "Old Author" -> "Controls Engineering"
```

## Future backends

To add another industrial backend without touching the AIPLC session loop:

- keep the Rust MCP tool contract stable
- implement another `PlcBackend` in Rust, or another subprocess adapter that speaks the existing JSON request/response contract
- preserve the same verification and compile reporting rules
