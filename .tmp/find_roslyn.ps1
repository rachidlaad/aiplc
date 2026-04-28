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
      Where-Object { $_.PSIsContainer -eq $false -and ($_.Name -ieq 'csc.exe' -or $_.Name -ieq 'vbcscompiler.exe' -or $_.Name -ieq 'MSBuild.exe') } |
      Select-Object FullName |
      Format-Table -AutoSize
  }
}
