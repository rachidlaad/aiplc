$ErrorActionPreference = 'Continue'
$key = 'HKLM:\SOFTWARE\Siemens\Automation\Openness\21.0'
if (Test-Path $key) {
  Get-ItemProperty $key | Format-List *
} else {
  'missing'
}
