$ErrorActionPreference = 'Continue'
$paths = @(
  'C:\\Program Files\\Microsoft Visual Studio\\2022\\BuildTools\\MSBuild\\Current\\Bin\\MSBuild.exe',
  'C:\\Program Files\\Microsoft Visual Studio\\2022\\Community\\MSBuild\\Current\\Bin\\MSBuild.exe',
  'C:\\Program Files (x86)\\Microsoft Visual Studio\\2019\\BuildTools\\MSBuild\\Current\\Bin\\MSBuild.exe',
  'C:\\Windows\\Microsoft.NET\\Framework64\\v4.0.30319\\MSBuild.exe'
)
foreach ($p in $paths) {
  if (Test-Path $p) { Write-Output $p }
}
Write-Output '=== dotnet ==='
try { & 'C:\\Program Files\\dotnet\\dotnet.exe' --list-sdks } catch { 'dotnet missing' }
