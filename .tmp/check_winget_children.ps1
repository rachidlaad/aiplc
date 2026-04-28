$ErrorActionPreference = 'Continue'
$winget = Get-CimInstance Win32_Process | Where-Object { $_.Name -ieq 'winget.exe' }
$winget | Select-Object Name,ProcessId,ParentProcessId,CreationDate,ExecutablePath,CommandLine | Format-List
if ($winget) {
  Get-CimInstance Win32_Process | Where-Object { $_.ParentProcessId -eq $winget.ProcessId } |
    Select-Object Name,ProcessId,ParentProcessId,CreationDate,ExecutablePath,CommandLine | Format-List
}
