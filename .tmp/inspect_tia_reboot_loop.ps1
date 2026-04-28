$ErrorActionPreference = 'Continue'

function Section($title) {
  Write-Output "`n=== $title ==="
}

Section 'Timestamp'
Get-Date -Format o

Section 'Siemens Scheduled Task'
$taskName = 'ContinueAfterReboot_SIAStarter'
$taskOutput = & "$env:SystemRoot\System32\schtasks.exe" /Query /TN $taskName /FO LIST /V 2>&1
$taskOutput

Section 'Siemens Start.bat'
$startBat = 'C:\Program Files (x86)\Common Files\Siemens\Automation\Siemens Installer Assistant\600\Start.bat'
if (Test-Path $startBat) {
  Get-Item $startBat | Select-Object FullName,Length,LastWriteTime | Format-List
  Get-Content $startBat
} else {
  Write-Output "Missing: $startBat"
}

Section 'Siemens Run/RunOnce Entries'
$runKeys = @(
  'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Run',
  'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnce',
  'HKCU:\SOFTWARE\Microsoft\Windows\CurrentVersion\Run',
  'HKCU:\SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnce'
)
foreach ($key in $runKeys) {
  Write-Output "-- $key"
  if (Test-Path $key) {
    $item = Get-ItemProperty -Path $key
    $props = $item.PSObject.Properties |
      Where-Object { $_.Name -notmatch '^PS' } |
      Select-Object Name, Value
    if ($props) {
      $props | Where-Object { $_.Name -match 'Siemens|SIA|TIA|Installer|Setup' -or $_.Value -match 'Siemens|SIA|TIA|Installer|Setup|Start\.bat' } | Format-Table -AutoSize
    } else {
      Write-Output '(no values)'
    }
  } else {
    Write-Output '(key missing)'
  }
}

Section 'Windows Pending Reboot Indicators'
$checks = @(
  @{ Name = 'CBS RebootPending'; Path = 'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Component Based Servicing\RebootPending' },
  @{ Name = 'WindowsUpdate RebootRequired'; Path = 'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\WindowsUpdate\Auto Update\RebootRequired' }
)
foreach ($check in $checks) {
  [pscustomobject]@{ Indicator = $check.Name; Exists = (Test-Path $check.Path); Path = $check.Path }
}

$sessionManager = 'HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager'
$pendingValue = $null
try {
  $pendingValue = (Get-ItemProperty -Path $sessionManager -Name PendingFileRenameOperations -ErrorAction Stop).PendingFileRenameOperations
} catch {
}
[pscustomobject]@{
  Indicator = 'Session Manager PendingFileRenameOperations'
  Exists = [bool]$pendingValue
  Path = "$sessionManager\\PendingFileRenameOperations"
  EntryCount = if ($pendingValue) { @($pendingValue).Count } else { 0 }
}
if ($pendingValue) {
  Write-Output '-- PendingFileRenameOperations values --'
  @($pendingValue)
}

Section 'Siemens Log Directory'
$logDir = 'C:\ProgramData\Siemens\Automation\Logfiles\Setup'
if (Test-Path $logDir) {
  Get-ChildItem $logDir | Sort-Object LastWriteTime -Descending | Select-Object Name,Length,LastWriteTime | Format-Table -AutoSize
} else {
  Write-Output "Missing: $logDir"
}

Section 'Latest SIA_Starter.log tail'
$starterLog = Join-Path $logDir 'SIA_Starter.log'
if (Test-Path $starterLog) {
  Get-Content $starterLog -Tail 120
} else {
  Write-Output 'SIA_Starter.log missing'
}

Section 'Any newer Siemens setup logs in common locations'
$scanPaths = @(
  'C:\ProgramData\Siemens',
  'C:\Users\intel\AppData\Local\Temp',
  'C:\Windows\Temp',
  'C:\Program Files (x86)\Common Files\Siemens\Automation\Siemens Installer Assistant\600'
)
foreach ($p in $scanPaths) {
  if (Test-Path $p) {
    Write-Output "-- $p"
    Get-ChildItem -Path $p -Recurse -ErrorAction SilentlyContinue |
      Where-Object { $_.PSIsContainer -eq $false -and $_.Name -match 'setup|siemens|tia|starter|bootstrap|install|msi|prereq' } |
      Sort-Object LastWriteTime -Descending |
      Select-Object -First 40 FullName,LastWriteTime,Length |
      Format-Table -AutoSize
  }
}

Section 'Running Siemens/Installer Processes'
Get-Process -ErrorAction SilentlyContinue |
  Where-Object {
    $_.ProcessName -match 'Siemens|TIA|Setup|Installer|SIA|Start' -or
    $_.Path -match 'Siemens|TIA|Setup|Installer'
  } |
  Select-Object ProcessName,Id,MainWindowTitle,Path,StartTime |
  Sort-Object StartTime |
  Format-Table -AutoSize

Section 'Primary inspection complete'
Write-Output 'Done.'
