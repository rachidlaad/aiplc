$ErrorActionPreference = 'Stop'
$outDir = 'C:\codex\adapters\siemens-tia-openness\Aiplc.TiaOpenness.Adapter\bin\Release\net48'
$outFile = 'C:\codex\adapters\siemens-tia-openness\Aiplc.TiaOpenness.Adapter\bin\Release\net48\Aiplc.TiaOpenness.Adapter.exe'
New-Item -ItemType Directory -Force -Path $outDir | Out-Null
$csc = 'C:\Windows\Microsoft.NET\Framework64\v4.0.30319\csc.exe'
$args = @(
  '/nologo',
  '/target:exe',
  ('/out:' + $outFile),
  '/reference:C:\Windows\Microsoft.NET\Framework64\v4.0.30319\System.Web.Extensions.dll',
  '/reference:C:\Windows\Microsoft.NET\Framework64\v4.0.30319\Microsoft.CSharp.dll',
  'C:\codex\adapters\siemens-tia-openness\Aiplc.TiaOpenness.Adapter\Program.cs',
  'C:\codex\adapters\siemens-tia-openness\Aiplc.TiaOpenness.Adapter\TiaOpennessBridge.cs'
)
$args | ForEach-Object { Write-Output $_ }
