$ErrorActionPreference = 'Continue'
$task = 'ContinueAfterReboot_SIAStarter'
try {
  Export-ScheduledTask -TaskName $task | Out-String
} catch {
  $_ | Format-List * -Force
}
