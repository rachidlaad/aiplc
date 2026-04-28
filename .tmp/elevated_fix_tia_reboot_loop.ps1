$ErrorActionPreference = 'Stop'
$log = 'C:\codex\.tmp\elevated_fix_tia_reboot_loop.log'
function Log($msg) { Add-Content -Path $log -Value $msg }
New-Item -ItemType File -Force -Path $log | Out-Null
Log "start $(Get-Date -Format o)"

$sessionKey = 'HKLM\SYSTEM\CurrentControlSet\Control\Session Manager'
try {
  Get-Process -ErrorAction SilentlyContinue |
    Where-Object { $_.ProcessName -ieq 'Start' -or $_.ProcessName -ieq 'Start.exe' } |
    ForEach-Object {
      Log "stopping Start.exe pid=$($_.Id)"
      Stop-Process -Id $_.Id -Force
    }
} catch {
  Log "stop-process error: $($_.Exception.Message)"
}

Start-Sleep -Seconds 1

try {
  Log 'deleting PendingFileRenameOperations'
  & 'C:\Windows\System32\reg.exe' delete $sessionKey /v PendingFileRenameOperations /f | Out-Null
  Log 'deleted PendingFileRenameOperations'
} catch {
  Log "reg delete error: $($_.Exception.Message)"
}

try {
  $after = (& 'C:\Windows\System32\reg.exe' query $sessionKey /v PendingFileRenameOperations) 2>&1
  $after | Out-File -FilePath 'C:\codex\.tmp\pending-file-rename-after.txt' -Encoding utf8
  Log 'queried PendingFileRenameOperations after delete'
} catch {
  Log 'PendingFileRenameOperations absent after delete'
  'VALUE_ABSENT' | Out-File -FilePath 'C:\codex\.tmp\pending-file-rename-after.txt' -Encoding utf8
}

try {
  Log 'launching local TIA setup elevated'
  Start-Process -FilePath 'C:\TIA_V21\DVD1\Start.exe' -Verb RunAs
  Log 'launch requested'
} catch {
  Log "launch error: $($_.Exception.Message)"
}

Start-Sleep -Seconds 25
try {
  Get-Content 'C:\ProgramData\Siemens\Automation\Logfiles\Setup\SIA_Starter.log' -Tail 120 | Out-File -FilePath 'C:\codex\.tmp\sia-starter-after-relaunch.txt' -Encoding utf8
  Log 'captured SIA_Starter.log tail after relaunch'
} catch {
  Log "log capture error: $($_.Exception.Message)"
}

try {
  Get-CimInstance Win32_Process | Where-Object { $_.Name -ieq 'Start.exe' -or $_.Name -ieq 'start.exe' } |
    Select-Object Name,ProcessId,CreationDate,ExecutablePath,CommandLine |
    Format-List | Out-File -FilePath 'C:\codex\.tmp\start-processes-after.txt' -Encoding utf8
  Log 'captured running Start.exe instances'
} catch {
  Log "process capture error: $($_.Exception.Message)"
}

Log "end $(Get-Date -Format o)"
