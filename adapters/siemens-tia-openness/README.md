# Siemens TIA Openness Adapter

This folder contains the Windows-local companion adapter for `codex-plc-mcp-server`.

Key points:

- Target framework: `.NET Framework 4.8`
- Runtime model: local subprocess launched by the Rust MCP server
- Siemens integration: runtime reflection over installed Openness assemblies
- Primary target: TIA Portal Openness `V21`
- Legacy path: best-effort support for `V20`-style single-assembly installs

Build on a Windows machine with TIA Portal installed:

```powershell
msbuild adapters\siemens-tia-openness\Aiplc.TiaOpenness.Adapter\Aiplc.TiaOpenness.Adapter.csproj /p:Configuration=Release
```

Runtime assembly discovery:

- `--public-api-dir`
- `CODEX_TIA_PUBLICAPI_DIR`
- Siemens Openness registry keys
- `C:\Program Files\Siemens\Automation\Portal V*\PublicAPI\...`

The adapter intentionally avoids compile-time Siemens references so the Rust MCP server and automated tests can run without TIA installed.

Use the main repository document for the full operator workflow, safety boundaries, and manual Windows acceptance flow:

- [docs/plc-tia-portal.md](/mnt/c/codex/docs/plc-tia-portal.md)
