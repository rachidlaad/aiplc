$ErrorActionPreference = 'Stop'

$backupDir = 'C:\codex\.tmp\tia_reboot_fix_backup'
New-Item -ItemType Directory -Force -Path $backupDir | Out-Null

$sessionKey = 'HKLM\SYSTEM\CurrentControlSet\Control\Session Manager'
$backupReg = Join-Path $backupDir 'session-manager-before.reg'
$backupTxt = Join-Path $backupDir 'pending-file-rename-before.txt'
$afterTxt = Join-Path $backupDir 'pending-file-rename-after.txt'
$logSnapshot = Join-Path $backupDir 'sia-starter-before-tail.txt'

# Backup current registry state and recent Siemens log tail.
& 'C:\Windows\System32\reg.exe' export $sessionKey $backupReg /y | Out-Null
$pending = (& 'C:\Windows\System32\reg.exe' query $sessionKey /v PendingFileRenameOperations) 2>&1
$pending | Out-File -FilePath $backupTxt -Encoding utf8
if (Test-Path 'C:\ProgramData\Siemens\Automation\Logfiles\Setup\SIA_Starter.log') {
  Get-Content 'C:\ProgramData\Siemens\Automation\Logfiles\Setup\SIA_Starter.log' -Tail 80 | Out-File -FilePath $logSnapshot -Encoding utf8
}

# Close the currently displayed Siemens starter prompt if it is still open.
Get-Process -ErrorAction SilentlyContinue |
  Where-Object { $_.ProcessName -ieq 'Start' -or $_.ProcessName -ieq 'Start.exe' } |
  ForEach-Object { Stop-Process -Id $_.Id -Force }

Start-Sleep -Seconds 1

# Remove only the single reboot gate Siemens is tripping on.
& 'C:\Windows\System32\reg.exe' delete $sessionKey /v PendingFileRenameOperations /f | Out-Null

# Verify removal.
$after = (& 'C:\Windows\System32\reg.exe' query $sessionKey /v PendingFileRenameOperations) 2>&1
$after | Out-File -FilePath $afterTxt -Encoding utf8

# Relaunch local installer elevated.
Start-Process -FilePath 'C:\TIA_V21\DVD1\Start.exe' -Verb RunAs

# Wait a bit for log activity.
Start-Sleep -Seconds 20

Write-Output '=== Backup Files ==='
Get-ChildItem $backupDir | Select-Object FullName,Length,LastWriteTime | Format-Table -AutoSize

Write-Output '=== PendingFileRenameOperations After ==='
Get-Content $afterTxt

Write-Output '=== Start.exe Processes After Relaunch ==='
Get-CimInstance Win32_Process | Where-Object { $_.Name -ieq 'Start.exe' -or $_.Name -ieq 'start.exe' } |
  Select-Object Name,ProcessId,CreationDate,ExecutablePath,CommandLine | Format-List

Write-Output '=== Latest Siemens Log Tail After Relaunch ==='
if (Test-Path 'C:\ProgramData\Siemens\Automation\Logfiles\Setup\SIA_Starter.log') {
  Get-Content 'C:\ProgramData\Siemens\Automation\Logfiles\Setup\SIA_Starter.log' -Tail 120
}
