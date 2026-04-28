$ErrorActionPreference = 'Continue'
Write-Output '=== Openness group members ==='
try { Get-LocalGroupMember -Group 'Siemens TIA Openness' | Select-Object Name,ObjectClass,PrincipalSource | Format-Table -AutoSize } catch { $_.Exception.Message }
Write-Output '=== Siemens Portal roots ==='
Get-ChildItem 'C:\Program Files\Siemens\Automation' -Directory -ErrorAction SilentlyContinue | Select-Object FullName,Name | Format-Table -AutoSize
Write-Output '=== PublicAPI candidates ==='
Get-ChildItem 'C:\Program Files\Siemens\Automation' -Recurse -Directory -ErrorAction SilentlyContinue | Where-Object { $_.FullName -match 'PublicAPI|Openness|net48' } | Select-Object -First 80 FullName | Format-Table -AutoSize
Write-Output '=== Registry Openness keys ==='
$keys = @(
  'HKLM:\SOFTWARE\Siemens\Automation\Openness',
  'HKLM:\SOFTWARE\WOW6432Node\Siemens\Automation\Openness'
)
foreach ($k in $keys) {
  Write-Output "-- $k"
  if (Test-Path $k) {
    Get-ChildItem $k | Select-Object Name | Format-Table -AutoSize
    Get-ItemProperty $k | Format-List *
  } else {
    Write-Output 'missing'
  }
}
