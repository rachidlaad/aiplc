$ErrorActionPreference = 'Stop'
$log = 'C:\codex\.tmp\add_openness_user.log'
New-Item -ItemType File -Force -Path $log | Out-Null
Add-Content $log "start $(Get-Date -Format o)"
& 'C:\Windows\System32\net.exe' localgroup 'Siemens TIA Openness' 'DESKTOP-8DACNP1\intel' /add 2>&1 | Out-File -FilePath $log -Append -Encoding utf8
Add-Content $log "end $(Get-Date -Format o)"
