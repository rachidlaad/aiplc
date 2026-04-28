$ErrorActionPreference = 'SilentlyContinue'
$roots = @(
  'C:\\Program Files\\Microsoft Visual Studio',
  'C:\\Program Files (x86)\\Microsoft Visual Studio',
  'C:\\Program Files\\dotnet',
  'C:\\Program Files (x86)\\dotnet'
)
foreach ($root in $roots) {
  if (Test-Path $root) {
    Write-Output "=== $root ==="
    Get-ChildItem $root -Recurse -ErrorAction SilentlyContinue |
      Where-Object { $_.PSIsContainer -eq $false -and ($_.Name -ieq 'MSBuild.exe' -or $_.Name -ieq 'dotnet.exe') } |
      Select-Object -First 60 FullName |
      Format-Table -AutoSize
  }
}
