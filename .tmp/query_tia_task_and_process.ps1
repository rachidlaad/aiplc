$ErrorActionPreference = 'Continue'

Write-Output '=== Scheduled Task via Get-ScheduledTask ==='
try {
  $task = Get-ScheduledTask -TaskName 'ContinueAfterReboot_SIAStarter' -ErrorAction Stop
  $task | Select-Object TaskName,State,Author,@{N='Execute';E={$_.Actions.Execute}},@{N='Arguments';E={$_.Actions.Arguments}} | Format-List
} catch {
  Write-Output $_.Exception.Message
}

Write-Output '=== Scheduled Task via schtasks.exe ==='
& 'C:\Windows\System32\schtasks.exe' /Query /TN 'ContinueAfterReboot_SIAStarter' /FO LIST /V 2>&1

Write-Output '=== Running Start.exe instances ==='
Get-CimInstance Win32_Process | Where-Object { $_.Name -ieq 'Start.exe' -or $_.Name -ieq 'start.exe' } |
  Select-Object Name,ProcessId,ParentProcessId,CreationDate,ExecutablePath,CommandLine |
  Format-List
